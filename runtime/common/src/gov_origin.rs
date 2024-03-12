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

pub use frame_support::traits::{EitherOf, EitherOfDiverse};

// darwinia
use dc_primitives::AccountId;
// substrate
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, EnsureProportionMoreThan};

pub type RootOrDiverse<T> = EitherOfDiverse<Root, T>;
pub type RootOr<T> = EitherOf<Root, T>;
pub type Root = EnsureRoot<AccountId>;

pub type RootOrAtLeastHalf<T> = EitherOfDiverse<Root, AtLeastHalf<T>>;
pub type AtLeastHalf<T> = EnsureProportionAtLeast<AccountId, T, 1, 2>;

pub type RootOrMoreThanHalf<T> = EitherOfDiverse<Root, MoreThanHalf<T>>;
pub type MoreThanHalf<T> = EnsureProportionMoreThan<AccountId, T, 1, 2>;

pub type RootOrAtLeastTwoThird<T> = EitherOfDiverse<Root, AtLeastTwoThird<T>>;
pub type AtLeastTwoThird<T> = EnsureProportionAtLeast<AccountId, T, 2, 3>;

pub type RootOrAtLeastThreeFifth<T> = EitherOfDiverse<Root, AtLeastThreeFifth<T>>;
pub type AtLeastThreeFifth<T> = EnsureProportionAtLeast<AccountId, T, 3, 5>;

pub type RootOrAtLeastFourFifth<T> = EitherOfDiverse<Root, AtLeastFourFifth<T>>;
pub type AtLeastFourFifth<T> = EnsureProportionAtLeast<AccountId, T, 4, 5>;

pub type RootOrAll<T> = RootOrDiverse<All<T>>;
pub type All<T> = EnsureProportionAtLeast<AccountId, T, 1, 1>;
