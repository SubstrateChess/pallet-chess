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

pub const INCENTIVE_SHARE: u8 = 10; // TODO configure value

#[frame_support::pallet]
pub mod pallet {
	use crate::{WeightInfo, INCENTIVE_SHARE};
	use cozy_chess::{Board, Color, GameStatus, Move};
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		sp_runtime::{
			traits::{AccountIdConversion, Hash},
			FixedPointOperand, Percent, Saturating,
		},
		traits::{
			fungibles::{Inspect, Transfer},
			tokens::Balance,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::format;
	use sp_std::{
		str::{from_utf8, FromStr},
		vec::Vec,
	};

	type AssetIdOf<T> =
		<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

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
		pub start: T::BlockNumber,
		pub bet_asset_id: AssetIdOf<T>,
		pub bet_amount: T::AssetBalance,
	}

	impl<T: Config> Match<T> {
		fn challenger_bet(&self) -> DispatchResult {
			if !T::Assets::asset_exists(self.bet_asset_id) {
				return Err(Error::<T>::BetDoesNotExist.into());
			}

			if self.bet_amount < T::Assets::minimum_balance(self.bet_asset_id) {
				return Err(Error::<T>::BetTooLow.into());
			}

			T::Assets::transfer(
				self.bet_asset_id,
				&self.challenger,
				&T::pallet_account(),
				self.bet_amount,
				false,
			)?;
			Ok(())
		}

		fn opponent_bet(&self) -> DispatchResult {
			T::Assets::transfer(
				self.bet_asset_id,
				&self.opponent,
				&T::pallet_account(),
				self.bet_amount,
				false,
			)?;
			Ok(())
		}

		fn abort_bet(&self) -> DispatchResult {
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				&self.challenger,
				self.bet_amount,
				false,
			)?;
			Ok(())
		}

		fn refund_bets(&self) -> DispatchResult {
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				&self.challenger,
				self.bet_amount,
				false,
			)?;
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				&self.opponent,
				self.bet_amount,
				false,
			)?;
			Ok(())
		}

		fn win_bet(&self, winner: &T::AccountId) -> DispatchResult {
			let win_amount = self.bet_amount.saturating_add(self.bet_amount);
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				winner,
				win_amount,
				false,
			)?;
			Ok(())
		}

		fn clear_expired_bet(&self, winner: &T::AccountId, third_party: &T::AccountId) -> DispatchResult {
			let win_amount = self.bet_amount.saturating_add(self.bet_amount);
			let incentive = Percent::from_percent(INCENTIVE_SHARE) * win_amount;
			let the_rest = win_amount - incentive;
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				third_party,
				incentive,
				false,
			)?;
			T::Assets::transfer(
				self.bet_asset_id,
				&T::pallet_account(),
				winner,
				the_rest,
				false,
			)?;
			Ok(())
		}
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
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type Assets: Inspect<Self::AccountId, Balance = Self::AssetBalance>
			+ Transfer<Self::AccountId>;
		type AssetBalance: Balance
			+ FixedPointOperand
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ TypeInfo;

		#[pallet::constant]
		type BulletPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type BlitzPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type RapidPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type DailyPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		type IncentivePeriod: Get<Self::BlockNumber>;
	}

	pub trait ConfigHelper: Config {
		fn pallet_account() -> Self::AccountId;
	}

	impl<T: Config> ConfigHelper for T {
		fn pallet_account() -> T::AccountId {
			Self::PalletId::get().into_account_truncating()
		}
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
		MatchRefundError(T::Hash),
		MatchAwardError(T::Hash, T::AccountId),
		MatchClearanceError(T::Hash, T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NonceOverflow,
		NonExistentMatch,
		InvalidOpponent,
		NotMatchOpponent,
		NotMatchChallenger,
		InvalidBoardEncoding,
		InvalidMoveEncoding,
		NotAwaitingOpponent,
		StillAwaitingOpponent,
		MatchAlreadyFinished,
		NotYourTurn,
		IllegalMove,
		BetTooLow,
		BetDoesNotExist,
		MatchNotOnGoing,
		MatchNotExpired,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// for every ongoing match, checks if the player defined by NextMove has run out of time
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut matches_win = Vec::new();
			let mut matches_draw = Vec::new();
			for (m_id, m) in Matches::<T>::iter() {
				match m.state {
					MatchState::OnGoing(_) => {
						// first move of the match can delay 100x longer
						if m.last_move == 0u32.into() {
							let diff = now - m.start;
							let draw: bool = match m.style {
								MatchStyle::Bullet => diff > T::BulletPeriod::get() * 100u32.into(),
								MatchStyle::Blitz => diff > T::BlitzPeriod::get() * 100u32.into(),
								MatchStyle::Rapid => diff > T::RapidPeriod::get() * 100u32.into(),
								MatchStyle::Daily => diff > T::DailyPeriod::get() * 100u32.into(),
							};
							if draw {
								matches_draw.push((m_id, m));
							}
						} else {
							let diff = now - m.last_move;
							let finish: bool = match m.style {
								MatchStyle::Bullet => diff > T::BulletPeriod::get(),
								MatchStyle::Blitz => diff > T::BlitzPeriod::get(),
								MatchStyle::Rapid => diff > T::RapidPeriod::get(),
								MatchStyle::Daily => diff > T::DailyPeriod::get(),
							};
							if finish {
								matches_win.push((m_id, m));
							}
						}
					},
					_ => continue,
				}
			}

			for (m_id, m) in matches_draw {
				Self::deposit_event(Event::MatchDrawn(m_id, m.board.clone()));

				// refund deposit to both players
				match m.refund_bets() {
					Ok(()) => {},
					Err(_) => Self::deposit_event(Event::MatchRefundError(m_id)),
				}

				// match is over, clean up storage
				<Matches<T>>::remove(m_id);
				<MatchIdFromNonce<T>>::remove(m.nonce);
			}

			for (m_id, m) in matches_win {
				let winner = match m.state {
					MatchState::OnGoing(NextMove::Whites) => m.opponent.clone(),
					MatchState::OnGoing(NextMove::Blacks) => m.challenger.clone(),
					_ => continue,
				};

				Self::deposit_event(Event::MatchWon(m_id, winner.clone(), m.board.clone()));

				// winner gets both deposits
				match m.win_bet(&winner) {
					Ok(()) => {},
					Err(_) => Self::deposit_event(Event::MatchAwardError(m_id, winner)),
				}

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
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_match())]
		pub fn create_match(
			origin: OriginFor<T>,
			opponent: T::AccountId,
			style: MatchStyle,
			bet_asset_id: AssetIdOf<T>,
			bet_amount: T::AssetBalance,
		) -> DispatchResult {
			let challenger = ensure_signed(origin)?;

			if challenger == opponent {
				return Err(Error::<T>::InvalidOpponent.into());
			}

			let nonce = <NextNonce<T>>::get();

			let new_match: Match<T> = Match {
				challenger: challenger.clone(),
				opponent: opponent.clone(),
				board: Self::init_board(),
				state: MatchState::AwaitingOpponent,
				nonce: nonce.clone(),
				style,
				last_move: 0u32.into(),
				start: 0u32.into(),
				bet_asset_id,
				bet_amount,
			};

			new_match.challenger_bet()?;

			let match_id = Self::match_id(challenger.clone(), opponent.clone(), nonce.clone());
			<Matches<T>>::insert(match_id, new_match);
			<MatchIdFromNonce<T>>::insert(nonce, match_id);
			Self::increment_nonce()?;

			Self::deposit_event(Event::MatchCreated(challenger, opponent, match_id));

			Ok(())
		}

		#[pallet::call_index(1)]
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

			chess_match.abort_bet()?;

			<Matches<T>>::remove(match_id);
			<MatchIdFromNonce<T>>::remove(chess_match.nonce);

			Self::deposit_event(Event::MatchAborted(match_id));

			Ok(())
		}

		#[pallet::call_index(2)]
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

			chess_match.opponent_bet()?;

			chess_match.state = MatchState::OnGoing(NextMove::Whites);
			chess_match.start = <frame_system::Pallet<T>>::block_number();
			<Matches<T>>::insert(match_id, chess_match);

			Self::deposit_event(Event::MatchStarted(match_id));

			Ok(())
		}

		#[pallet::call_index(3)]
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
				Self::deposit_event(Event::MatchWon(
					match_id,
					who.clone(),
					chess_match.board.clone(),
				));

				// winner gets both deposits
				chess_match.win_bet(&who)?;

				// match is over, clean up storage
				<Matches<T>>::remove(match_id);
				<MatchIdFromNonce<T>>::remove(chess_match.nonce);
			} else if chess_match.state == MatchState::Drawn {
				Self::deposit_event(Event::MatchDrawn(match_id, chess_match.board.clone()));

				// return deposit to both players
				chess_match.refund_bets()?;

				// match is over, clean up storage
				<Matches<T>>::remove(match_id);
				<MatchIdFromNonce<T>>::remove(chess_match.nonce);
			} else {
				// match still ongoing, update on-chain board
				<Matches<T>>::insert(match_id, chess_match);
			}

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(1000_000)] // TODO weights
		pub fn clear_expired_match(
			origin: OriginFor<T>,
			match_id: T::Hash
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let chess_match = match Self::chess_matches(match_id) {
				Some(m) => m,
				None => return Err(Error::<T>::NonExistentMatch.into()),
			};

			ensure!(
				(chess_match.state == MatchState::OnGoing(NextMove::Whites)) | 
				(chess_match.state == MatchState::OnGoing(NextMove::Blacks)), 
				Error::<T>::MatchNotOnGoing
			);
			
			let now = <frame_system::Pallet<T>>::block_number();
			let diff = now - chess_match.last_move;
			let expired: bool = match chess_match.style {
				MatchStyle::Bullet => diff > T::BulletPeriod::get(),
				MatchStyle::Blitz => diff > T::BlitzPeriod::get(),
				MatchStyle::Rapid => diff > T::RapidPeriod::get(),
				MatchStyle::Daily => diff > T::DailyPeriod::get(),
			};

			ensure!(expired, Error::<T>::MatchNotExpired);

			let winner = match chess_match.state {
				MatchState::OnGoing(NextMove::Whites) => chess_match.opponent.clone(),
				_ => chess_match.challenger.clone(),
			};

			Self::deposit_event(Event::MatchWon(match_id, winner.clone(), chess_match.board.clone()));

			let time_since_expiry = match chess_match.style {
				MatchStyle::Bullet => now - (chess_match.last_move + T::BulletPeriod::get()),
				MatchStyle::Blitz => now - (chess_match.last_move + T::BlitzPeriod::get()),
				MatchStyle::Rapid => now - (chess_match.last_move + T::RapidPeriod::get()),
				MatchStyle::Daily => now - (chess_match.last_move + T::DailyPeriod::get()),
			};

			if (who == chess_match.challenger) | (who == chess_match.opponent) | (time_since_expiry <= T::IncentivePeriod::get()) {
				// winner gets both deposits
				match chess_match.win_bet(&winner) {
					Ok(()) => {},
					Err(_) => Self::deposit_event(Event::MatchAwardError(match_id, winner)),
				}
			} else {
				// who cleared the match after incentive period gets the incentive, 
				// and the winner gets both deposits minus the incentive share
				match chess_match.clear_expired_bet(&winner, &who) {
					Ok(()) => {},
					Err(_) => Self::deposit_event(Event::MatchClearanceError(match_id, winner, who)),
				}
			}

			<Matches<T>>::remove(match_id);
			<MatchIdFromNonce<T>>::remove(chess_match.nonce);

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
