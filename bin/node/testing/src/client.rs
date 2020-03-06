//! Utilities to build a `TestClient` for `node-runtime`.

use sp_runtime::BuildStorage;

/// Re-export test-client utilities.
pub use substrate_test_client::*;

/// Call executor for `node-runtime` `TestClient`.
pub type Executor = sc_executor::NativeExecutor<node_executor::Executor>;

/// Default backend type.
pub type Backend = sc_client_db::Backend<node_primitives::Block>;

/// Test client type.
pub type Client = sc_client::Client<
	Backend,
	sc_client::LocalCallExecutor<Backend, Executor>,
	node_primitives::Block,
	node_runtime::RuntimeApi,
>;

/// Transaction for node-runtime.
pub type Transaction = sc_client_api::backend::TransactionFor<Backend, node_primitives::Block>;

/// Genesis configuration parameters for `TestClient`.
#[derive(Default)]
pub struct GenesisParameters {
	support_changes_trie: bool,
}

impl substrate_test_client::GenesisInit for GenesisParameters {
	fn genesis_storage(&self) -> Storage {
		crate::genesis::config(self.support_changes_trie, None)
			.build_storage()
			.unwrap()
	}
}

/// A `test-runtime` extensions to `TestClientBuilder`.
pub trait TestClientBuilderExt: Sized {
	/// Create test client builder.
	fn new() -> Self;

	/// Build the test client.
	fn build(self) -> Client;
}

impl TestClientBuilderExt
	for substrate_test_client::TestClientBuilder<
		node_primitives::Block,
		sc_client::LocalCallExecutor<Backend, Executor>,
		Backend,
		GenesisParameters,
	>
{
	fn new() -> Self {
		Self::default()
	}

	fn build(self) -> Client {
		self.build_with_native_executor(None).0
	}
}
