// This file is part of Darwinia.
//
// Copyright (C) 2018-2023 Darwinia Network
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

// darwinia
use dc_primitives::AccountId;
// moonbeam
use account::AccountId20;
// substrate
use frame_support::traits::EitherOfDiverse;
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, EnsureProportionMoreThan};

pub type Root = EnsureRoot<AccountId>;

pub type RootOrAtLeastHalf<Collective> =
	EitherOfDiverse<Root, EnsureProportionAtLeast<AccountId, Collective, 1, 2>>;

pub type RootOrMoreThanHalf<Collective> =
	EitherOfDiverse<Root, EnsureProportionMoreThan<AccountId, Collective, 1, 2>>;

pub type RootOrAtLeastTwoThird<Collective> =
	EitherOfDiverse<Root, EnsureProportionAtLeast<AccountId, Collective, 2, 3>>;

pub type RootOrAtLeastThreeFifth<Collective> =
	EitherOfDiverse<Root, EnsureProportionAtLeast<AccountId, Collective, 3, 5>>;

pub type RootOrAll<Collective> =
	EitherOfDiverse<Root, EnsureProportionAtLeast<AccountId, Collective, 1, 1>>;

/// An [`AccountId20`] generated from b"root".
pub const ROOT: AccountId20 =
	AccountId20([0x72, 0x6f, 0x6f, 0x74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
