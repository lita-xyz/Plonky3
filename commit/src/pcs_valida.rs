//! Extension traits for the PCS used in Valida.
//!
//! https://github.com/Plonky3/Plonky3/pull/253
//! deleted the UnivariatePcsWithLde trait.
//! This reexposes the minimal functionality so that
//! valida-vm is compatible with latest Plonky3.
//!
//! In particular we need public non-bit-reversed LDEs.
use p3_field::ExtensionField;
use p3_matrix::dense::RowMajorMatrix;

use crate::pcs::{Pcs, Val};

pub trait PcsValidaExt<Challenge, Challenger>: Pcs<Challenge, Challenger>
where
    Challenge: ExtensionField<Val<<Self as Pcs<Challenge, Challenger>>::Domain>>,
{
    fn compute_lde_batch(
        &self,
        polynomials: RowMajorMatrix<Val<<Self as Pcs<Challenge, Challenger>>::Domain>>,
    ) -> RowMajorMatrix<Val<<Self as Pcs<Challenge, Challenger>>::Domain>>;
}
