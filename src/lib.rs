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
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use sp_std::{
		str::{from_utf8, FromStr},
		vec,
	};

	#[derive(Debug, Encode, Decode, TypeInfo, PartialEq)]
	pub enum MatchState {
		AwaitingOpponent,
		OnGoing,
		Finished,
	}

	#[derive(Encode, Decode, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Match<T: Config> {
		pub challenger: T::AccountId,
		pub opponent: T::AccountId,
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
	pub(super) type Nonce<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn matches)]
	pub(super) type Matches<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Match<T>>;

	#[pallet::storage]
	#[pallet::getter(fn match_id_from_nonce)]
	pub(super) type AllMatchesArray<T: Config> =
		StorageMap<_, Twox64Concat, u128, T::Hash, ValueQuery>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MatchCreated(T::AccountId, T::AccountId, T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		NonceOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_match(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let challenger = ensure_signed(origin)?;

			// todo: reserve deposit of challenger
			let nonce = <Nonce<T>>::get();

			let new_match: Match<T> = Match {
				challenger: challenger.clone(),
				opponent: opponent.clone(),
				board: Self::init_game(),
				state: MatchState::AwaitingOpponent,
			};

			let match_id = Self::match_id(challenger.clone(), opponent.clone(), nonce.clone());
			<Matches<T>>::insert(match_id, new_match);
			<AllMatchesArray<T>>::insert(nonce, match_id);
			Self::increment_nonce()?;

			Self::deposit_event(Event::MatchCreated(challenger, opponent, match_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn join_match(origin: OriginFor<T>, match_id: T::Hash) -> DispatchResult {
			// todo: check origin is opponent

			// todo: reserve deposit of opponent

			// todo: change match state

			// todo: anything else?

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn increment_nonce() -> DispatchResult {
			<Nonce<T>>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or(Error::<T>::NonceOverflow)?;
				*nonce = next;

				Ok(().into())
			})
		}

		fn match_id(challenger: T::AccountId, opponent: T::AccountId, nonce: u128) -> T::Hash {
			T::Hashing::hash_of(&(challenger, opponent, nonce))
		}

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
