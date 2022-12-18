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
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::WeightInfo;
	use cozy_chess::{Board, Color, GameStatus, Move};
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::format;
	use sp_std::{
		str::{from_utf8, FromStr},
		vec::Vec,
	};

	#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
	pub enum MatchStyle {
		Bullet, // 1 minute
		Blitz,  // 5 minutes
		Rapid,  // 15 minutes
		Daily,  // 1 day
	}

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
		// no need for BoundedVec, we only modify this internally (and safely)
		// introducing an extra runtime constant would be overkill
		pub board: Vec<u8>,
		pub state: MatchState,
		pub nonce: u128,
		pub style: MatchStyle,
		pub last_move: T::BlockNumber,
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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type BulletPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type BlitzPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type RapidPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type DailyPeriod: Get<Self::BlockNumber>;
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// for every ongoing match, checks if the player defined by NextMove has run out of time
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut matches_to_finish = Vec::new();
			for (m_id, m) in Matches::<T>::iter() {
				match m.state {
					MatchState::OnGoing(_) => {
						// first move can delay longer
						if m.last_move == 0u32.into() {
							continue
						}
						let delta = now - m.last_move;
						let finish = match m.style {
							MatchStyle::Bullet => (delta > T::BulletPeriod::get()),
							MatchStyle::Blitz => (delta > T::BlitzPeriod::get()),
							MatchStyle::Rapid => (delta > T::RapidPeriod::get()),
							MatchStyle::Daily => (delta > T::DailyPeriod::get()),
						};
						if finish {
							matches_to_finish.push((m_id, m));
						}
					},
					_ => continue,
				}
			}

			for (m_id, m) in matches_to_finish {
				let winner = match m.state {
					MatchState::OnGoing(NextMove::Whites) => m.opponent,
					MatchState::OnGoing(NextMove::Blacks) => m.challenger,
					_ => continue,
				};

				Self::deposit_event(Event::MatchWon(m_id, winner, m.board.clone()));

				// todo: winner gets both deposits

				// match is over, clean up storage
				<Matches<T>>::remove(m_id);
				<MatchIdFromNonce<T>>::remove(m.nonce);
			}

			// todo: calculate proper weights
			Weight::zero()
		}
	}

	const MOVE_FEN_LENGTH: usize = 4;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_match())]
		pub fn create_match(
			origin: OriginFor<T>,
			opponent: T::AccountId,
			style: MatchStyle,
		) -> DispatchResult {
			let challenger = ensure_signed(origin)?;

			// todo: reserve deposit of challenger
			let nonce = <NextNonce<T>>::get();

			let new_match: Match<T> = Match {
				challenger: challenger.clone(),
				opponent: opponent.clone(),
				board: Self::init_board(),
				state: MatchState::AwaitingOpponent,
				nonce: nonce.clone(),
				style,
				last_move: 0u32.into(),
			};

			let match_id = Self::match_id(challenger.clone(), opponent.clone(), nonce.clone());
			<Matches<T>>::insert(match_id, new_match);
			<MatchIdFromNonce<T>>::insert(nonce, match_id);
			Self::increment_nonce()?;

			Self::deposit_event(Event::MatchCreated(challenger, opponent, match_id));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::abort_match())]
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

		#[pallet::weight(T::WeightInfo::join_match())]
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

		#[pallet::weight(T::WeightInfo::make_move())]
		pub fn make_move(
			origin: OriginFor<T>,
			match_id: T::Hash,
			move_fen: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(move_fen.len() == MOVE_FEN_LENGTH, Error::<T>::InvalidMoveEncoding);

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
			chess_match.last_move = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::MoveExecuted(match_id, who.clone(), move_fen));
			if chess_match.state == MatchState::Won {
				Self::deposit_event(Event::MatchWon(match_id, who, chess_match.board.clone()));

				// todo: winner gets both deposits

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

		// needed for benchmarking
		// todo: check if pub is a vulnerability
		pub fn force_board_state(
			match_id: T::Hash,
			encoded_board: Vec<u8>,
		) -> sp_std::result::Result<(), Error<T>> {
			let mut chess_match = match Self::chess_matches(match_id) {
				Some(m) => m,
				None => return Err(Error::<T>::NonExistentMatch.into()),
			};

			chess_match.board = encoded_board.clone();

			let board_obj = Self::decode_board(encoded_board)?;
			chess_match.state = match board_obj.status() {
				GameStatus::Ongoing => match board_obj.side_to_move() {
					Color::White => MatchState::OnGoing(NextMove::Whites),
					Color::Black => MatchState::OnGoing(NextMove::Blacks),
				},
				GameStatus::Won => MatchState::Won,
				GameStatus::Drawn => MatchState::Drawn,
			};

			if chess_match.state == MatchState::Won {
				// match is over, clean up storage
				<Matches<T>>::remove(match_id);
				<MatchIdFromNonce<T>>::remove(chess_match.nonce);
			} else if chess_match.state == MatchState::Drawn {
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
}
