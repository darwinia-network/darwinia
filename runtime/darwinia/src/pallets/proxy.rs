// --- crates.io ---
use codec::{Decode, Encode};
// --- paritytech ---
use frame_support::traits::InstanceFilter;
use pallet_proxy::Config;
use sp_runtime::RuntimeDebug;
// --- darwinia-network ---
use crate::{weights::pallet_proxy::WeightInfo, *};

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	EthereumBridge,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::System(..) |
				Call::Babe(..) |
				Call::Timestamp(..) |
				// Specifically omitting the entire Balances pallet
				Call::Authorship(..) |
				Call::Staking(..) |
				Call::Offences(..) |
				Call::Session(..) |
				Call::Grandpa(..) |
				Call::ImOnline(..) |
				Call::AuthorityDiscovery(..) |
				Call::Council(..) |
				Call::TechnicalCommittee(..) |
				Call::PhragmenElection(..) |
				Call::TechnicalMembership(..) |
				Call::Treasury(..) |
				Call::KtonTreasury(..) |
				Call::Tips(..) |
				Call::Bounties(..) |
				Call::Democracy(..) |
				Call::Utility(..) |
				Call::Identity(..) |
				Call::Society(..) |
				Call::Recovery(pallet_recovery::Call::as_recovered(..)) |
				Call::Recovery(pallet_recovery::Call::vouch_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::claim_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::close_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::remove_recovery(..)) |
				Call::Recovery(pallet_recovery::Call::cancel_recovered(..)) |
				Call::Vesting(darwinia_vesting::Call::vest(..)) |
				Call::Vesting(darwinia_vesting::Call::vest_other(..)) |
				// Specifically omitting Vesting `vested_transfer`, and `force_vested_transfer`
				Call::Scheduler(..) |
				Call::Proxy(..) |
				Call::Multisig(..) |
				// Specifically omitting the entire CrabBacking pallet
				// Specifically omitting the entire EthereumBacking pallet
				Call::EthereumRelay(..) |
				// Specifically omitting the entire TronBacking pallet
				Call::DarwiniaHeaderMMR(..) // Specifically omitting the entire EthereumRelayAuthorities pallet
			),
			ProxyType::Governance => matches!(
				c,
				Call::Council(..)
					| Call::TechnicalCommittee(..)
					| Call::PhragmenElection(..)
					| Call::Treasury(..) | Call::KtonTreasury(..)
					| Call::Tips(..) | Call::Bounties(..)
					| Call::Democracy(..)
					| Call::Utility(..)
			),
			ProxyType::Staking => matches!(c, Call::Staking(..) | Call::Utility(..)),
			ProxyType::IdentityJudgement => matches!(
				c,
				Call::Identity(pallet_identity::Call::provide_judgement(..))
					| Call::Utility(pallet_utility::Call::batch(..))
			),
			ProxyType::EthereumBridge => matches!(
				c,
				Call::EthereumBacking(..)
					| Call::EthereumRelay(..)
					| Call::EthereumRelayAuthorities(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

frame_support::parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type MaxPending = MaxPending;
	type CallHasher = Hashing;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type WeightInfo = WeightInfo<Runtime>;
}
