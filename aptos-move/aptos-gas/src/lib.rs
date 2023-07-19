// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

//! This crate is the core of the gas metering system of the Aptos blockchain.
//!
//! More specifically, it
//!   - Is home to the gas meter implementation
//!   - Defines the gas parameters and formulae for instructions
//!   - Defines the gas parameters for transactions
//!   - Sets the initial values for all gas parameters, including the instruction, transaction
//!     move-stdlib and aptos-framework ones.
//!   - Defines a bi-directional mapping between the (Rust) gas parameter structs and their
//!     corresponding representation on-chain.
//!
//! The reason why we need two different representations is that they serve different purposes:
//!   - The Rust structs are used for quick (static) lookups by the gas meter and native functions
//!     when calculating gas costs.
//!   - The on-chain gas schedule needs to be extensible and unordered so we can upgrade it easily
//!     in the future.

#[macro_use]
mod natives;

mod aptos_framework;
mod gas_meter;
pub mod gen;
mod move_stdlib;
mod table;

pub use gas_meter::{
    AptosGasMeter, AptosGasParameters, FromOnChainGasSchedule, InitialGasSchedule,
    NativeGasParameters, StandardGasAlgebra, StandardGasMeter, ToOnChainGasSchedule,
};
