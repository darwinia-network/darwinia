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

#[frame_support::pallet]
pub mod custom_origins {
	// crates.io
	use strum::EnumString;
	// polkadot-sdk
	use frame_support::pallet_prelude::*;
	use sp_runtime::RuntimeDebug;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(
		Clone, PartialEq, Eq, Encode, Decode, TypeInfo, RuntimeDebug, MaxEncodedLen, EnumString,
	)]
	#[strum(serialize_all = "snake_case")]
	#[pallet::origin]
	pub enum Origin {
		/// Origin able to dispatch a whitelisted call.
		WhitelistedCaller,
		/// Origin able to cancel referenda.
		ReferendumCanceller,
		/// Origin able to kill referenda.
		ReferendumKiller,
	}

	macro_rules! decl_unit_ensures {
		($name:ident: $success_type:ty = $success:expr) => {
			pub struct $name;
			impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for $name {
				type Success = $success_type;

				fn try_origin(o: O) -> Result<Self::Success, O> {
					o.into().and_then(|o| match o {
						Origin::$name => Ok($success),
						r => Err(O::from(r)),
					})
				}

				#[cfg(feature = "runtime-benchmarks")]
				fn try_successful_origin() -> Result<O, ()> {
					Ok(O::from(Origin::$name))
				}
			}
		};
		($name:ident) => { decl_unit_ensures! { $name : () = () } };
		($name:ident: $success_type:ty = $success:expr, $($rest:tt)*) => {
			decl_unit_ensures! { $name: $success_type = $success }
			decl_unit_ensures! { $($rest)* }
		};
		($name:ident, $( $rest:tt )*) => {
			decl_unit_ensures! { $name }
			decl_unit_ensures! { $($rest)* }
		};
		() => {}
	}
	decl_unit_ensures!(ReferendumCanceller, ReferendumKiller, WhitelistedCaller);

	// macro_rules! decl_ensure {
	// 	(
	// 		$vis:vis type $name:ident: EnsureOrigin<Success = $success_type:ty> {
	// 			$( $item:ident = $success:expr, )*
	// 		}
	// 	) => {
	// 		$vis struct $name;
	// 		impl<O: Into<Result<Origin, O>> + From<Origin>>
	// 			EnsureOrigin<O> for $name
	// 		{
	// 			type Success = $success_type;
	// 			fn try_origin(o: O) -> Result<Self::Success, O> {
	// 				o.into().and_then(|o| match o {
	// 					$(
	// 						Origin::$item => Ok($success),
	// 					)*
	// 					r => Err(O::from(r)),
	// 				})
	// 			}
	// 			#[cfg(feature = "runtime-benchmarks")]
	// 			fn try_successful_origin() -> Result<O, ()> {
	// 				// By convention the more privileged origins go later, so for greatest chance
	// 				// of success, we want the last one.
	// 				let _result: Result<O, ()> = Err(());
	// 				$(
	// 					let _result: Result<O, ()> = Ok(O::from(Origin::$item));
	// 				)*
	// 				_result
	// 			}
	// 		}
	// 	}
	// }
}
pub use custom_origins::*;
