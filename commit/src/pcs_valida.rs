//! Extension traits for the PCS used in Valida.
//!
//! https://github.com/Plonky3/Plonky3/pull/253
//! deleted the UnivariatePcsWithLde trait.
//! This reexposes the minimal functionality so that
//! valida-vm is compatible with latest Plonky3.
//!
//! In particular we need public non-bit-reversed LDEs.
use alloc::vec::Vec;
use p3_field::ExtensionField;
use p3_matrix::{dense::RowMajorMatrix, Matrix};

use crate::pcs::{Pcs, Val};

pub trait PcsValidaExt<Challenge, Challenger>: Pcs<Challenge, Challenger>
where
    Challenge: ExtensionField<Val<Self::Domain>>,
{
    // Should be the same as
    // ```
    //    let(_commit, data) = self.commit(vec![evaluations]);
    //    self.get_evaluations_on_domain(&data, 0, domain)
    // ```
    // but faster, without the need to compute the actual commitment
    fn domain_extend_evaluations<'a>(
        &self,
        evaluations: RowMajorMatrix<Val<Self::Domain>>,
        evaluation_domain: Self::Domain,
        extension_domain: Self::Domain,
    ) -> impl Matrix<Val<Self::Domain>> + 'a;

    // For each point in `points`, return the evaluations
    // of the polynomial encoded by each column of `evaluations`
    // using `evaluation domain` at that point.
    fn evaluate_at_points<M>(
        &self,
        evaluations: &M,
        evaluation_domain: Self::Domain,
        points: &[Challenge],
    ) -> Vec<Vec<Challenge>>
    where
        M: Matrix<Val<Self::Domain>> + Clone;
}
