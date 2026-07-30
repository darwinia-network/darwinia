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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Weight interface for `darwinia_deposit`.

use frame_support::weights::Weight;

/// Weight functions needed for `darwinia_deposit`.
pub trait WeightInfo {
	/// Weight of `migrate_for`.
	fn migrate_for() -> Weight;
	/// Weight of `set_deposit_contract`.
	fn set_deposit_contract() -> Weight;
}
