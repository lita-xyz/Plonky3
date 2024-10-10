use alloc::slice;
use alloc::vec::Vec;
use core::iter;

use p3_commit::{Pcs, PcsValidaExt, PolynomialSpace};
use p3_field::{ExtensionField, TwoAdicField};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_util::log2_strict_usize;

use crate::StarkGenericConfig;

pub trait PublicValues<F, E>: Matrix<F> + Sized
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, zeta: E, offset: usize) -> Vec<E>
    where
        Self: core::marker::Sized,
    {
        let height = self.height();
        let log_height = log2_strict_usize(height);
        let g = F::two_adic_generator(log_height);
        let shift = g.powers().nth(offset).unwrap();

        p3_interpolation::interpolate_coset::<F, E, _>(self, shift, zeta)
    }

    fn get_ldes<SC>(&self, config: &SC) -> Self
    where
        SC: StarkGenericConfig<Challenge = E>,
        <SC::Pcs as Pcs<SC::Challenge, SC::Challenger>>::Domain: PolynomialSpace<Val = F>,
        <SC as StarkGenericConfig>::Pcs: PcsValidaExt<E, <SC as StarkGenericConfig>::Challenger>;
}

impl<F, E, T> PublicValues<F, E> for T
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
    T: From<RowMajorMatrix<F>> + Matrix<F> + Sized + Clone,
{
    fn get_ldes<SC>(&self, config: &SC) -> Self
    where
        SC: StarkGenericConfig<Challenge = E>,
        <SC::Pcs as Pcs<SC::Challenge, SC::Challenger>>::Domain: PolynomialSpace<Val = F>,
        <SC as StarkGenericConfig>::Pcs: PcsValidaExt<E, <SC as StarkGenericConfig>::Challenger>,
    {
        let pcs = config.pcs();
        let mat = self.clone().to_row_major_matrix();
        pcs.compute_lde_batch(mat).into()
    }
}

// In the case that the public values are a vector rather than a matrix,
// we view it as a matrix with a single row repeated as many times as desired.
#[derive(Clone, Debug, Default)]
pub struct PublicRow<F>(pub Vec<F>);

// LITA: original impl incompatible with Matrix rework:
//       https://github.com/Plonky3/Plonky3/pull/300

impl<T: Clone + Send + Sync> Matrix<T> for PublicRow<T> {
    #[inline]
    fn width(&self) -> usize {
        self.0.len()
    }
    #[inline]
    fn height(&self) -> usize {
        1
    }

    type Row<'a>
        = iter::Cloned<slice::Iter<'a, T>>
    where
        T: 'a,
        Self: 'a;

    #[inline]
    fn row(&self, _r: usize) -> Self::Row<'_> {
        assert_eq!(_r, 1);
        self.0.iter().cloned()
    }
}

impl<F, E> PublicValues<F, E> for PublicRow<F>
where
    F: TwoAdicField,
    E: ExtensionField<F> + TwoAdicField,
{
    fn interpolate(&self, _zeta: E, _offset: usize) -> Vec<E> {
        self.0.iter().map(|v| E::from_base(*v)).collect()
    }

    fn get_ldes<SC>(&self, _config: &SC) -> Self
    where
        SC: StarkGenericConfig,
    {
        self.clone()
    }
}
