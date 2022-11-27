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
	use cozy_chess::{Board, Color, GameStatus, Move};
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::format;
	use sp_std::{
		str::{from_utf8, FromStr},
		vec::Vec,
	};

	#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
	pub enum NextMove {
		Whites,
		Blacks,
	}

	#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
	pub enum MatchState {
		AwaitingOpponent,
		OnGoing(NextMove),
		Won,
		Drawn,
	}

	#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
	#[scale_info(skip_type_params(T))]
	pub struct Match<T: Config> {
		pub challenger: T::AccountId,
		pub opponent: T::AccountId,
		pub board: Vec<u8>,
		pub state: MatchState,
		pub nonce: u128,
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	// https://stackoverflow.com/questions/70206199/substrate-tutorials-trait-maxencodedlen-is-not-implemented-for-vecu8
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_nonce)]
	pub(super) type NextNonce<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn chess_matches)]
	pub(super) type Matches<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Match<T>>;

	#[pallet::storage]
	#[pallet::getter(fn chess_match_id_from_nonce)]
	pub(super) type MatchIdFromNonce<T: Config> = StorageMap<_, Twox64Concat, u128, T::Hash>;

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
		MatchAborted(T::Hash),
		MatchStarted(T::Hash),
		MoveExecuted(T::Hash, T::AccountId, Vec<u8>),
		MatchWon(T::Hash, T::AccountId, Vec<u8>),
		MatchDrawn(T::Hash, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NonceOverflow,
		NonExistentMatch,
		NotMatchOpponent,
		NotMatchChallenger,
		InvalidBoardEncoding,
		InvalidMoveEncoding,
		NotAwaitingOpponent,
		StillAwaitingOpponent,
		MatchAlreadyFinished,
		NotYourTurn,
		IllegalMove,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_match(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let challenger = ensure_signed(origin)?;

			// todo: reserve deposit of challenger
			let nonce = <NextNonce<T>>::get();

			let new_match: Match<T> = Match {
				challenger: challenger.clone(),
				opponent: opponent.clone(),
				board: Self::init_board(),
				state: MatchState::AwaitingOpponent,
				nonce: nonce.clone(),
			};

			let match_id = Self::match_id(challenger.clone(), opponent.clone(), nonce.clone());
			<Matches<T>>::insert(match_id, new_match);
			<MatchIdFromNonce<T>>::insert(nonce, match_id);
			Self::increment_nonce()?;

			Self::deposit_event(Event::MatchCreated(challenger, opponent, match_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn abort_match(origin: OriginFor<T>, match_id: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let chess_match = match Self::chess_matches(match_id) {
				Some(m) => m,
				None => return Err(Error::<T>::NonExistentMatch.into()),
			};

			if who != chess_match.challenger {
				return Err(Error::<T>::NotMatchChallenger.into());
			}

			if chess_match.state != MatchState::AwaitingOpponent {
				return Err(Error::<T>::NotAwaitingOpponent.into());
			}

			// todo: release reserve deposit to challenger

			<Matches<T>>::remove(match_id);
			<MatchIdFromNonce<T>>::remove(chess_match.nonce);

			Self::deposit_event(Event::MatchAborted(match_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn join_match(origin: OriginFor<T>, match_id: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut chess_match = match Self::chess_matches(match_id) {
				Some(m) => m,
				None => return Err(Error::<T>::NonExistentMatch.into()),
			};

			if who != chess_match.opponent {
				return Err(Error::<T>::NotMatchOpponent.into());
			}

			// todo: reserve deposit of opponent

			chess_match.state = MatchState::OnGoing(NextMove::Whites);
			<Matches<T>>::insert(match_id, chess_match);

			Self::deposit_event(Event::MatchStarted(match_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn make_move(
			origin: OriginFor<T>,
			match_id: T::Hash,
			move_fen: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut chess_match = match Self::chess_matches(match_id) {
				Some(m) => m,
				None => return Err(Error::<T>::NonExistentMatch.into()),
			};

			match chess_match.state {
				MatchState::AwaitingOpponent => {
					return Err(Error::<T>::StillAwaitingOpponent.into())
				},
				MatchState::Won | MatchState::Drawn => {
					return Err(Error::<T>::MatchAlreadyFinished.into())
				},
				MatchState::OnGoing(NextMove::Whites) => {
					if who != chess_match.challenger {
						return Err(Error::<T>::NotYourTurn.into());
					}
				},
				MatchState::OnGoing(NextMove::Blacks) => {
					if who != chess_match.opponent {
						return Err(Error::<T>::NotYourTurn.into());
					}
				},
			}

			let mut board_obj: Board = Self::decode_board(chess_match.board)?;
			let move_obj: Move = Self::decode_move(move_fen.clone())?;

			if !board_obj.is_legal(move_obj) {
				return Err(Error::<T>::IllegalMove.into());
			}

			// we already checked for legality, so we call play_unchecked (faster)
			board_obj.play_unchecked(move_obj);

			// check game status: Won? Drawn? OnGoing?
			chess_match.state = match board_obj.status() {
				GameStatus::Ongoing => match board_obj.side_to_move() {
					Color::White => MatchState::OnGoing(NextMove::Whites),
					Color::Black => MatchState::OnGoing(NextMove::Blacks),
				},
				GameStatus::Won => MatchState::Won,
				GameStatus::Drawn => MatchState::Drawn,
			};

			chess_match.board = Self::encode_board(board_obj);

			Self::deposit_event(Event::MoveExecuted(match_id, who.clone(), move_fen));
			if chess_match.state == MatchState::Won {
				Self::deposit_event(Event::MatchWon(match_id, who, chess_match.board.clone()));

				// match is over, clean up storage
				<Matches<T>>::remove(match_id);
				<MatchIdFromNonce<T>>::remove(chess_match.nonce);
			} else if chess_match.state == MatchState::Drawn {
				Self::deposit_event(Event::MatchDrawn(match_id, chess_match.board.clone()));

				// match is over, clean up storage
				<Matches<T>>::remove(match_id);
				<MatchIdFromNonce<T>>::remove(chess_match.nonce);
			} else {
				// match still ongoing, update on-chain board
				<Matches<T>>::insert(match_id, chess_match);
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn increment_nonce() -> DispatchResult {
			<NextNonce<T>>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or(Error::<T>::NonceOverflow)?;
				*nonce = next;

				Ok(().into())
			})
		}

		fn match_id(challenger: T::AccountId, opponent: T::AccountId, nonce: u128) -> T::Hash {
			T::Hashing::hash_of(&(challenger, opponent, nonce))
		}

		fn init_board() -> Vec<u8> {
			format!("{}", Board::default()).as_bytes().to_vec()
		}

		fn encode_board(board: Board) -> Vec<u8> {
			format!("{}", board).as_bytes().to_vec()
		}

		fn decode_board(encoded_board: Vec<u8>) -> sp_std::result::Result<Board, Error<T>> {
			let s = match from_utf8(encoded_board.as_slice()) {
				Ok(s) => s,
				Err(_) => "",
			};
			match Board::from_str(s) {
				Ok(g) => Ok(g),
				Err(_) => Err(Error::<T>::InvalidBoardEncoding.into()),
			}
		}

		fn decode_move(encoded_move: Vec<u8>) -> sp_std::result::Result<Move, Error<T>> {
			let s = match from_utf8(encoded_move.as_slice()) {
				Ok(s) => s,
				Err(_) => "",
			};
			match Move::from_str(s) {
				Ok(m) => Ok(m),
				Err(_) => Err(Error::<T>::InvalidMoveEncoding.into()),
			}
		}
	}
}
