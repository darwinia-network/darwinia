
extern crate parity_codec as codec;
extern crate sr_std as rstd;

use codec::{Codec, Encode, Decode};
use srml_support::dispatch::Result;
use sr_primitives::traits::{
Zero, SimpleArithmetic, As, StaticLookup, Member, CheckedAdd, CheckedSub,
MaybeSerializeDebug, Saturating
};
use rstd::{prelude::*, vec};
