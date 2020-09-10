//! Darwinia types shared between the runtime and the Node-side code.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

/// App-specific crypto used for reporting equivocation/misbehavior in BABE,
/// GRANDPA and Parachains, described in the white paper as the fisherman role.
/// Any rewards for misbehavior reporting will be paid out to this account.
pub mod fisherman {
	// --- substrate ---
	use sp_core::crypto::KeyTypeId;
	// --- crates ---
	use super::{Signature, Verify};

	/// Key type for the reporting module. Used for reporting BABE, GRANDPA
	/// and Parachain equivocations.
	pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"fish");

	mod app {
		use sp_application_crypto::{app_crypto, sr25519};
		app_crypto!(sr25519, super::KEY_TYPE);
	}

	/// Identity of the equivocation/misbehavior reporter.
	pub type FishermanId = app::Public;

	/// An `AppCrypto` type to allow submitting signed transactions using the fisherman
	/// application key as signer.
	pub struct FishermanAppCrypto;
	impl frame_system::offchain::AppCrypto<<Signature as Verify>::Signer, Signature>
		for FishermanAppCrypto
	{
		type RuntimeAppPublic = FishermanId;
		type GenericPublic = sp_core::sr25519::Public;
		type GenericSignature = sp_core::sr25519::Signature;
	}
}

// --- substrate ---
pub use sp_runtime::traits::{BlakeTwo256, Hash as HashT, IdentifyAccount, Verify};
/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

// --- substrate ---
use sp_core::H256;
use sp_runtime::{generic, MultiSignature};

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// A hash of some data used by the relay chain.
pub type Hash = H256;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Nonce = u32;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// The power of an account.
pub type Power = u32;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

#[ignore]
#[test]
fn print_module_account() {
	// --- substrate ---
	use sp_runtime::{traits::AccountIdConversion, ModuleId};

	fn account_of(alias: [u8; 8]) {
		eprintln!(
			"module_id: {}",
			<ModuleId as AccountIdConversion<AccountId>>::into_account(&ModuleId(alias))
		);
	}

	// old_module_id: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
	// new_module_id: 5EYCAe5gKAhKhPeR7nUZzpcX2f9eYoAhqtEHqnG433EfnCpQ
	account_of(*b"py/trsry");
	account_of(*b"da/trsry");
	// old_module_id: 5EYCAe5gKAhHFmT9jmwH1HL4mDpjC7KWzKSSDxo244qvxncz
	// new_module_id: 5EYCAe5gKAhHfsDGJniP6JVgq2bq92gWT7CJwPWh51MMTcYK
	account_of(*b"da/backi");
	account_of(*b"da/ethbk");
	// module_id: 5EYCAe5gKAhHQ8Hp3UUSqEGzsUtdrevrhUadXKWuwzDYmX9T
	account_of(*b"da/crais");
}
