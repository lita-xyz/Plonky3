use alloc::slice;
use alloc::vec::Vec;
use core::iter;

use p3_commit::{Pcs, PcsValidaExt, PolynomialSpace};
use p3_field::{ExtensionField, Field, TwoAdicField};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
pub trait PublicValues<F, E>: Matrix<F> + Sized + Clone
where
    F: Field,
    E: ExtensionField<F> + Field,
{
    fn interpolate<P, Challenger>(
        &self,
        pcs: &P,
        points: &[E],
        evaluation_domain: P::Domain,
    ) -> Vec<Vec<E>>
    where
        P: PcsValidaExt<E, Challenger>,
        P::Domain: PolynomialSpace<Val = F>,
    {
        pcs.evaluate_at_points(self, evaluation_domain, points)
    }

    fn get_evaluations_on_domain<'a, P, Challenger>(
        self,
        pcs: &P,
        evaluation_domain: P::Domain,
        extension_domain: P::Domain,
    ) -> impl Matrix<F> + 'a
    where
        P: PcsValidaExt<E, Challenger>,
        P::Domain: PolynomialSpace<Val = F>,
    {
        let mat = self.to_row_major_matrix();
        pcs.domain_extend_evaluations(mat, evaluation_domain, extension_domain)
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
        self.0.iter().cloned()
    }
}

impl<F, E> PublicValues<F, E> for PublicRow<F>
where
    F: Field,
    E: ExtensionField<F>,
{
    fn interpolate<P, Challenger>(
        &self,
        _pcs: &P,
        points: &[E],
        _evaluation_domain: P::Domain,
    ) -> Vec<Vec<E>>
    where
        P: PcsValidaExt<E, Challenger>,
        P::Domain: PolynomialSpace<Val = F>,
    {
        points
            .iter()
            .map(|_zeta| self.0.iter().map(|v| E::from_base(*v)).collect())
            .collect()
    }

    fn get_evaluations_on_domain<'a, P, Challenger>(
        self,
        _pcs: &P,
        _evaluation_domain: P::Domain,
        _extension_domain: P::Domain,
    ) -> impl Matrix<F> + 'a
    where
        P: PcsValidaExt<E, Challenger>,
        P::Domain: PolynomialSpace<Val = F>,
    {
        self.clone()
    }
}
