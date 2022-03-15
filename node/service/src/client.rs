// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Darwinia Client meta trait

/// Darwinia client abstraction, this super trait only pulls in functionality required for
/// Darwinia internal crates like Darwinia-collator.
pub trait DarwiniaClient<Block, Backend, Runtime>:
	Sized
	+ Send
	+ Sync
	+ sp_api::CallApiAt<Block>
	+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
	+ sp_blockchain::HeaderBackend<Block>
	+ sc_client_api::BlockchainEvents<Block>
where
	Block: sp_runtime::traits::Block,
	Backend: sc_client_api::Backend<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}
impl<Block, Backend, Runtime, Client> DarwiniaClient<Block, Backend, Runtime> for Client
where
	Block: sp_runtime::traits::Block,
	Backend: sc_client_api::Backend<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
	Client: Sized
		+ Send
		+ Sync
		+ sp_api::CallApiAt<Block>
		+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::BlockchainEvents<Block>,
{
}

/// Crab client abstraction, this super trait only pulls in functionality required for
/// Crab internal crates like Crab-collator.
pub trait CrabClient<Block, Backend, Runtime>:
	Sized
	+ Send
	+ Sync
	+ sp_api::CallApiAt<Block>
	+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
	+ sp_blockchain::HeaderBackend<Block>
	+ sc_client_api::BlockchainEvents<Block>
where
	Block: sp_runtime::traits::Block,
	Backend: sc_client_api::Backend<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
{
}
impl<Block, Backend, Runtime, Client> CrabClient<Block, Backend, Runtime> for Client
where
	Block: sp_runtime::traits::Block,
	Backend: sc_client_api::Backend<Block>,
	Runtime: sp_api::ConstructRuntimeApi<Block, Self>,
	Client: Sized
		+ Send
		+ Sync
		+ sp_api::CallApiAt<Block>
		+ sp_api::ProvideRuntimeApi<Block, Api = Runtime::RuntimeApi>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::BlockchainEvents<Block>,
{
}
