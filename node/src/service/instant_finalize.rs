// This file is part of Darwinia.
//
// Copyright (C) Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// polkadot-sdk
use sc_consensus::BlockImport;
use sp_runtime::traits::Block as BlockT;

pub struct InstantFinalizeBlockImport<I>(I);
impl<I> InstantFinalizeBlockImport<I> {
	/// Create a new instance.
	pub fn new(inner: I) -> Self {
		Self(inner)
	}
}
#[async_trait::async_trait]
impl<Block, I> BlockImport<Block> for InstantFinalizeBlockImport<I>
where
	Block: BlockT,
	I: Send + Sync + BlockImport<Block>,
{
	type Error = I::Error;

	async fn check_block(
		&self,
		block: sc_consensus::BlockCheckParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		self.0.check_block(block).await
	}

	async fn import_block(
		&mut self,
		mut block_import_params: sc_consensus::BlockImportParams<Block>,
	) -> Result<sc_consensus::ImportResult, Self::Error> {
		block_import_params.finalized = true;
		self.0.import_block(block_import_params).await
	}
}
