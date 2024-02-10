use alloc::vec::Vec;

use core::fmt::Debug;
use p3_commit::Mmcs;
use p3_field::Field;
use proptest::prelude::Arbitrary;
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Arbitrary)]
#[serde(bound(
    serialize = "Witness: Serialize",
    deserialize = "Witness: Deserialize<'de>"
))]
pub struct FriProof<F: Field + Arbitrary + Debug, M: Mmcs<F>, Witness> {
    pub(crate) commit_phase_commits: Vec<M::Commitment>,
    pub(crate) query_proofs: Vec<QueryProof<F, M>>,
    // This could become Vec<FC::Challenge> if this library was generalized to support non-constant
    // final polynomials.
    pub(crate) final_poly: F,
    pub(crate) pow_witness: Witness,
}

#[derive(Serialize, Deserialize, Debug, Arbitrary)]
#[serde(bound = "")]
pub struct QueryProof<F: Field + Arbitrary + Debug, M: Mmcs<F>> {
    /// For each commit phase commitment, this contains openings of a commit phase codeword at the
    /// queried location, along with an opening proof.
    pub(crate) commit_phase_openings: Vec<CommitPhaseProofStep<F, M>>,
}

#[derive(Serialize, Deserialize, Arbitrary, Debug)]
// #[serde(bound(serialize = "F: Serialize"))]
#[serde(bound = "")]
pub struct CommitPhaseProofStep<F: Field + Arbitrary + Debug, M: Mmcs<F>> {
    /// The opening of the commit phase codeword at the sibling location.
    // This may change to Vec<FC::Challenge> if the library is generalized to support other FRI
    // folding arities besides 2, meaning that there can be multiple siblings.
    pub(crate) sibling_value: F,

    pub(crate) opening_proof: M::Proof,
}
