use alloc::vec::Vec;

use p3_commit::Mmcs;
use p3_field::Field;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "Witness: Serialize",
    deserialize = "Witness: Deserialize<'de>"
))]
pub struct FriProof<F: Field, M: Mmcs<F>, Witness>
where
    F: Send + Sync,
    M::Commitment: Send + Sync,
    M::Proof: Send + Sync,
    Witness: Send + Sync,
{
    pub(crate) commit_phase_commits: Vec<M::Commitment>,
    pub(crate) query_proofs: Vec<QueryProof<F, M>>,
    pub(crate) final_poly: F,
    pub(crate) pow_witness: Witness,
}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>, Witness: Send + Sync> Send
    for FriProof<F, M, Witness>
where
    M::Commitment: Send + Sync,
    M::Proof: Send + Sync,
{
}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>, Witness: Send + Sync> Sync
    for FriProof<F, M, Witness>
where
    M::Commitment: Send + Sync,
    M::Proof: Send + Sync,
{
}

#[derive(Serialize, Deserialize)]
#[serde(bound = "")]
pub struct QueryProof<F: Field, M: Mmcs<F>>
where
    F: Send + Sync,
    M::Proof: Send + Sync,
{
    pub(crate) commit_phase_openings: Vec<CommitPhaseProofStep<F, M>>,
}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>> Send for QueryProof<F, M> where M::Proof: Send + Sync
{}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>> Sync for QueryProof<F, M> where M::Proof: Send + Sync
{}

#[derive(Serialize, Deserialize)]
// #[serde(bound(serialize = "F: Serialize"))]
#[serde(bound = "")]
pub struct CommitPhaseProofStep<F: Field, M: Mmcs<F>>
where
    F: Send + Sync,
    M::Proof: Send + Sync,
{
    pub(crate) sibling_value: F,
    pub(crate) opening_proof: M::Proof,
}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>> Send for CommitPhaseProofStep<F, M> where
    M::Proof: Send + Sync
{
}

unsafe impl<F: Field + Send + Sync, M: Mmcs<F>> Sync for CommitPhaseProofStep<F, M> where
    M::Proof: Send + Sync
{
}
