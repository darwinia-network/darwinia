use ethereum_types::{Address, Bloom, H160, H256, U256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

pub enum TransactionOutcome {
	/// Status and state root are unknown under EIP-98 rules.
	Unknown,
	/// State root is known. Pre EIP-98 and EIP-658 rules.
	StateRoot(H256),
	/// Status code is known. EIP-658 rules.
	StatusCode(u8),
}
