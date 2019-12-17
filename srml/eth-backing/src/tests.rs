//! Tests for the module.
use std::str::FromStr;

//use hex_literal::hex;
//use rustc_hex::FromHex;
use sr_eth_primitives::{
	receipt::{LogEntry, TransactionOutcome},
	Bloom, EthAddress, H64, U128,
};
use support::assert_ok;

use crate::{mock::*, *};
