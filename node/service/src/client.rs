//! Darwinia Client meta trait

/// Darwinia client abstraction, this super trait only pulls in functionality required for
/// Darwinia internal crates like Darwinia-collator.
pub trait DarwiniaClient<Block, Backend, Runtime>:
	Sized
	+ Send
	+ Sync
	+ sc_client_api::BlockchainEvents<Block>
	+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
	+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
	+ sp_blockchain::HeaderBackend<Block>
where
	Backend: sc_client_api::Backend<Block>,
	Block: sp_runtime::traits::Block,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}
impl<Block, Backend, Runtime, Client> DarwiniaClient<Block, Backend, Runtime> for Client
where
	Backend: sc_client_api::Backend<Block>,
	Block: sp_runtime::traits::Block,
	Client: Sized
		+ Send
		+ Sync
		+ sp_api::CallApiAt<Block, Error = sp_blockchain::Error, StateBackend = Backend::State>
		+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::BlockchainEvents<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}
