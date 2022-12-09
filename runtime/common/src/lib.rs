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

#![cfg_attr(not(feature = "std"), no_std)]
// TODO:
// #![deny(missing_docs)]

pub mod gov_origin;
pub mod xcm_configs;

pub use bp_darwinia_core as bp_crab;
pub use bp_darwinia_core as bp_darwinia;
pub use bp_darwinia_core as bp_pangolin;

#[macro_export]
macro_rules! fast_runtime_or_not {
	($name:ident, $development_type:ty, $production_type:ty) => {
		#[cfg(feature = "fast-runtime")]
		type $name = $development_type;
		#[cfg(not(feature = "fast-runtime"))]
		type $name = $production_type;
	};
}
