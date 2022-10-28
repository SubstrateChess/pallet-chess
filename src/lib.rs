#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use chess::Game;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::{
		str::{from_utf8, FromStr},
		vec,
	};

	#[derive(Encode, Decode, TypeInfo)]
	pub enum MatchState {
		AwaitingOpponent,
		OnGoing,
		Finished,
	}

	#[derive(Encode, Decode, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Match<T: Config> {
		pub white: T::AccountId,
		pub black: T::AccountId,
		pub board: Vec<u8>,
		pub state: MatchState,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	// https://stackoverflow.com/questions/70206199/substrate-tutorials-trait-maxencodedlen-is-not-implemented-for-vecu8
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn letter)]
	pub(super) type Matches<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Match<T>>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		fn init_game() -> Vec<u8> {
			Game::new().current_position().to_string().as_bytes().to_vec()
		}

		fn encode_game(game: Game) -> Vec<u8> {
			game.current_position().to_string().as_bytes().to_vec()
		}

		fn decode_game(
			encoded_game: Vec<u8>,
		) -> sp_std::result::Result<Game, TransactionValidityError> {
			let s = match from_utf8(encoded_game.as_slice()) {
				Ok(s) => s,
				Err(_) => "",
			};
			match Game::from_str(s) {
				Ok(g) => Ok(g),
				// todo: check if there's a better way to handle this
				Err(_) => Err(TransactionValidityError::Unknown(UnknownTransaction::Custom(0))),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encode_game_works() {}

	#[test]
	fn decode_game_works() {}
}
