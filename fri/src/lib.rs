//! An implementation of the FRI low-degree test (LDT).

#![no_std]

extern crate alloc;

mod config;
mod fold_even_odd;
// TODO: for now we deactivating HidingFriPCS:
// - HidingFriPcs does not implement the PcsValidaExt
//   so we cannot run the ZK examples
// - The proper Sync/Send requirements need to be added to use the parallel changes we introduced.
// mod hiding_pcs;
mod proof;
pub mod prover;
mod two_adic_pcs;
pub mod verifier;

pub use config::*;
pub use fold_even_odd::*;
// pub use hiding_pcs::*;
pub use proof::*;
pub use two_adic_pcs::*;
