use alloc::vec;
use alloc::vec::Vec;

use itertools::{izip, Itertools};
use p3_challenger::{CanObserve, FieldChallenger, GrindingChallenger};
use p3_commit::Mmcs;
use p3_dft::{Radix2Dit, TwoAdicSubgroupDft};
use p3_field::{ExtensionField, Field, TwoAdicField};
use p3_matrix::dense::{DenseMatrix, RowMajorMatrix};
use p3_maybe_rayon::prelude::*;
use p3_util::{log2_strict_usize, reverse_slice_index_bits};
use tracing::{debug_span, info_span, instrument};

use crate::{CommitPhaseProofStep, FriConfig, FriGenericConfig, FriProof, QueryProof};

#[instrument(name = "FRI prover", skip_all)]
pub fn prove<G, Val, Challenge, M, Challenger>(
    g: &G,
    config: &FriConfig<M>,
    inputs: Vec<Vec<Challenge>>,
    challenger: &mut Challenger,
    open_input: impl Fn(usize) -> G::InputProof,
) -> FriProof<Challenge, M, Challenger::Witness, G::InputProof>
where
    Val: Field,
    Challenge: ExtensionField<Val> + TwoAdicField,
    M: Mmcs<Challenge> + Sync,
    <M as Mmcs<Challenge>>::Proof: Send,
    <M as Mmcs<Challenge>>::ProverData<DenseMatrix<Challenge>>: Sync,
    Challenger: FieldChallenger<Val> + GrindingChallenger + CanObserve<M::Commitment>,
    G: FriGenericConfig<Challenge>,
{
    assert!(!inputs.is_empty());
    assert!(
        inputs
            .iter()
            .tuple_windows()
            .all(|(l, r)| l.len() >= r.len()),
        "Inputs are not sorted in descending order of length."
    );

    let log_max_height = log2_strict_usize(inputs[0].len());
    let log_min_height = log2_strict_usize(inputs.last().unwrap().len());
    if config.log_final_poly_len > 0 {
        assert!(log_min_height > config.log_final_poly_len + config.log_blowup);
    }

    let commit_phase_result = commit_phase(g, config, inputs, challenger);

    let pow_witness = challenger.grind(config.proof_of_work_bits);

    let query_proofs = info_span!("query phase").in_scope(|| {
        // LITA: undo https://github.com/Plonky3/Plonky3/pull/333/commits/a202042a0d7f102830f548751555560e48677148#diff-28b0919b8d62afd4eb098b9d57ad393727c0dd95138fab75b1bef5244eb45e71R42
        let query_indices: Vec<usize> = (0..config.num_queries)
            .map(|_| challenger.sample_bits(log_max_height + g.extra_query_index_bits()))
            .collect();

        // LITA: to avoid requiring Dft: Sync, we compute `open_input` separately from `answer_query`
        // The Radix2DIT DFT memoizes twiddle factors and that state makes it non-trivial to call in parallel.
        let input_proofs: Vec<_> = query_indices
            .iter()
            .map(|index| open_input(*index))
            .collect();

        let extra_bits = g.extra_query_index_bits();

        let commit_phase_openings: Vec<_> = query_indices
            .into_par_iter()
            .map(|index| answer_query(config, &commit_phase_result.data, index >> extra_bits))
            .collect();

        // LITA TODO: parallelize this
        input_proofs.into_iter().zip(commit_phase_openings.into_iter())
            .map(|(input_proof, commit_phase_openings)| QueryProof {
                input_proof,
                commit_phase_openings,
            })
            .collect()
    });

    FriProof {
        commit_phase_commits: commit_phase_result.commits,
        query_proofs,
        final_poly: commit_phase_result.final_poly,
        pow_witness,
    }
}

struct CommitPhaseResult<F: Field, M: Mmcs<F>> {
    commits: Vec<M::Commitment>,
    data: Vec<M::ProverData<RowMajorMatrix<F>>>,
    final_poly: Vec<F>,
}

#[instrument(name = "commit phase", skip_all)]
fn commit_phase<G, Val, Challenge, M, Challenger>(
    g: &G,
    config: &FriConfig<M>,
    inputs: Vec<Vec<Challenge>>,
    challenger: &mut Challenger,
) -> CommitPhaseResult<Challenge, M>
where
    Val: Field,
    Challenge: ExtensionField<Val> + TwoAdicField,
    M: Mmcs<Challenge>,
    Challenger: FieldChallenger<Val> + CanObserve<M::Commitment>,
    G: FriGenericConfig<Challenge>,
{
    let mut inputs_iter = inputs.into_iter().peekable();
    let mut folded = inputs_iter.next().unwrap();
    let mut commits = vec![];
    let mut data = vec![];

    while folded.len() > config.blowup() * config.final_poly_len() {
        let leaves = RowMajorMatrix::new(folded, 2);
        let (commit, prover_data) = config.mmcs.commit_matrix(leaves);
        challenger.observe(commit.clone());

        let beta: Challenge = challenger.sample_ext_element();
        // We passed ownership of `current` to the MMCS, so get a reference to it
        let leaves = config.mmcs.get_matrices(&prover_data).pop().unwrap();
        folded = g.fold_matrix(beta, leaves.as_view());

        commits.push(commit);
        data.push(prover_data);

        if let Some(v) = inputs_iter.next_if(|v| v.len() == folded.len()) {
            izip!(&mut folded, v).for_each(|(c, x)| *c += x);
        }
    }

    // After repeated folding steps, we end up working over a coset hJ instead of the original
    // domain. The IDFT we apply operates over a subgroup J, not hJ. This means the polynomial we
    // recover is G(x), where G(x) = F(hx), and F is the polynomial whose evaluations we actually
    // observed. For our current construction, this does not cause issues since degree properties
    // and zero-checks remain valid. If we changed our domain construction (e.g., using multiple
    // cosets), we would need to carefully reconsider these assumptions.

    reverse_slice_index_bits(&mut folded);
    // TODO: For better performance, we could run the IDFT on only the first half
    //       (or less, depending on `log_blowup`) of `final_poly`.
    let final_poly = debug_span!("idft final poly").in_scope(|| Radix2Dit::default().idft(folded));

    // The evaluation domain is "blown-up" relative to the polynomial degree of `final_poly`,
    // so all coefficients after the first final_poly_len should be zero.
    debug_assert!(
        final_poly
            .iter()
            .skip(1 << config.log_final_poly_len)
            .all(|x| x.is_zero()),
        "All coefficients beyond final_poly_len must be zero"
    );

    // Observe all coefficients of the final polynomial.
    for &x in &final_poly {
        challenger.observe_ext_element(x);
    }

    CommitPhaseResult {
        commits,
        data,
        final_poly,
    }
}

fn answer_query<F, M>(
    config: &FriConfig<M>,
    commit_phase_commits: &[M::ProverData<RowMajorMatrix<F>>],
    index: usize,
) -> Vec<CommitPhaseProofStep<F, M>>
where
    F: Field,
    M: Mmcs<F>,
{
    commit_phase_commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let index_i = index >> i;
            let index_i_sibling = index_i ^ 1;
            let index_pair = index_i >> 1;

            let (mut opened_rows, opening_proof) = config.mmcs.open_batch(index_pair, commit);
            assert_eq!(opened_rows.len(), 1);
            let opened_row = opened_rows.pop().unwrap();
            assert_eq!(opened_row.len(), 2, "Committed data should be in pairs");
            let sibling_value = opened_row[index_i_sibling % 2];

            CommitPhaseProofStep {
                sibling_value,
                opening_proof,
            }
        })
        .collect()
}
