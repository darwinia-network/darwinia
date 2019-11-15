#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod receipt;
#[cfg(test)]
mod tests;

pub mod transaction;
