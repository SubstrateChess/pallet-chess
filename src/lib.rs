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
    use frame_support::{
        pallet_prelude::{DispatchResult, ValueQuery, *},
        sp_runtime::{
            traits::{AccountIdConversion, Hash},
            FixedPointOperand, Percent, Saturating,
        },
        traits::{
            fungibles::{Inspect, Mutate},
            tokens::{Balance, Preservation},
            BuildGenesisConfig,
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::format;
    use sp_std::{
        str::{from_utf8, FromStr},
        vec::Vec,
    };

    pub type AssetIdOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
    pub type BalanceOf<T> =
        <<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub struct DefaultElo;
    impl Get<u16> for DefaultElo {
        fn get() -> u16 {
            1600
        }
    }

    #[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
    pub enum MatchStyle {
        Bullet, // 1 minute
        Blitz,  // 5 minutes
        Rapid,  // 15 minutes
        Daily,  // 1 day
    }

    #[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
    pub enum MatchState {
        AwaitingOpponent,
        OnGoing,
        Won,
        Drawn,
    }

    #[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq)]
    #[scale_info(skip_type_params(T))]
    pub struct Match<T: Config> {
        pub challenger: T::AccountId,
        pub opponent: T::AccountId,
        pub state: MatchState,
        pub nonce: u128,
        pub style: MatchStyle,
        pub start: BlockNumberFor<T>,
        pub bet_asset_id: AssetIdOf<T>,
        pub bet_amount: T::AssetBalance,
    }

    impl<T: Config> Match<T> {
        fn challenger_bet(&self) -> DispatchResult {
            if !T::Assets::asset_exists(self.bet_asset_id.clone()) {
                return Err(Error::<T>::BetDoesNotExist.into());
            }

            // bet must cover janitor incentives
            if Percent::from_percent(T::IncentiveShare::get())
                * self.bet_amount.saturating_add(self.bet_amount)
                < T::Assets::minimum_balance(self.bet_asset_id.clone())
            {
                return Err(Error::<T>::BetTooLow.into());
            }

            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &self.challenger,
                &T::pallet_account(),
                self.bet_amount,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        fn opponent_bet(&self) -> DispatchResult {
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &self.opponent,
                &T::pallet_account(),
                self.bet_amount,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        fn abort_bet(&self) -> DispatchResult {
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                &self.challenger,
                self.bet_amount,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        fn refund_bets(&self) -> DispatchResult {
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                &self.challenger,
                self.bet_amount,
                Preservation::Expendable,
            )?;
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                &self.opponent,
                self.bet_amount,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        fn win_bet(&self, winner: &T::AccountId) -> DispatchResult {
            let win_amount = self.bet_amount.saturating_add(self.bet_amount);
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                winner,
                win_amount,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        fn clear_abandoned_bet(
            &self,
            winner: &T::AccountId,
            janitor: &T::AccountId,
        ) -> DispatchResult {
            let (janitor_incentive, actual_prize) = self.janitor_incentive();
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                janitor,
                janitor_incentive,
                Preservation::Expendable,
            )?;
            T::Assets::transfer(
                self.bet_asset_id.clone(),
                &T::pallet_account(),
                winner,
                actual_prize,
                Preservation::Expendable,
            )?;
            Ok(())
        }

        pub fn janitor_incentive(&self) -> (BalanceOf<T>, BalanceOf<T>) {
            let winner_prize = self.bet_amount.saturating_add(self.bet_amount);
            let janitor_incentive = Percent::from_percent(T::IncentiveShare::get()) * winner_prize;
            let actual_prize = winner_prize.saturating_sub(janitor_incentive);
            (janitor_incentive, actual_prize)
        }
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_nonce)]
    pub(super) type NextNonce<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn chess_matches)]
    pub(super) type Matches<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Match<T>>;

    #[pallet::storage]
    #[pallet::getter(fn player_matches)]
    pub(super) type PlayerMatches<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::Hash, (), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn chess_match_id_from_nonce)]
    pub(super) type MatchIdFromNonce<T: Config> = StorageMap<_, Twox64Concat, u128, T::Hash>;

    #[pallet::storage]
    #[pallet::getter(fn player_elo)]
    pub(super) type PlayerElo<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u16, ValueQuery, DefaultElo>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Elo constant
        /// See https://metinmediamath.wordpress.com/2013/11/27/how-to-calculate-the-elo-rating-including-example/
        #[pallet::constant]
        type K: Get<u16>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type Assets: Inspect<Self::AccountId, Balance = Self::AssetBalance>
            + Mutate<Self::AccountId>;
        type AssetBalance: Balance
            + FixedPointOperand
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        #[pallet::constant]
        type BulletPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type BlitzPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type RapidPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type DailyPeriod: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type IncentiveShare: Get<u8>;
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
        NotAwaitingOpponent,
        StillAwaitingOpponent,
        MatchAlreadyFinished,
        BetTooLow,
        BetDoesNotExist,
        MatchNotOnGoing,
        MatchNotAbandoned,
        MoveNotExpired,
    }

    type GenesisInfo<T> = (AccountIdOf<T>, u16);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub elo: Vec<GenesisInfo<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            GenesisConfig { elo: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (account, elo) in &self.elo {
                <PlayerElo<T>>::insert(account, elo);
            }
        }
    }

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
                state: MatchState::AwaitingOpponent,
                nonce: nonce.clone(),
                style,
                start: 0u32.into(),
                bet_asset_id,
                bet_amount,
            };

            new_match.challenger_bet()?;

            let match_id = Self::match_id(challenger.clone(), opponent.clone(), nonce.clone());
            <Matches<T>>::insert(match_id, new_match);
            <PlayerMatches<T>>::insert(challenger.clone(), match_id, ());
            <PlayerMatches<T>>::insert(opponent.clone(), match_id, ());
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
            <PlayerMatches<T>>::remove(who.clone(), match_id);
            <PlayerMatches<T>>::remove(chess_match.opponent, match_id);
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

            chess_match.state = MatchState::OnGoing;
            chess_match.start = <frame_system::Pallet<T>>::block_number();
            <Matches<T>>::insert(match_id, chess_match);

            Self::deposit_event(Event::MatchStarted(match_id));

            Ok(())
        }

        // TODO: to be refactored. Implements a dispute resolution mechanism whereby players
        // can submit their latest play along with the zkVM execution proof. The play that will
        // be deemed the latest (as per the fullmove counter) and verified, will determine who
        // is the winner.
        // #[pallet::call_index(4)]
        // #[pallet::weight(T::WeightInfo::clear_abandoned_match())]
        // pub fn clear_abandoned_match(origin: OriginFor<T>, match_id: T::Hash) -> DispatchResult {
        //     let who = ensure_signed(origin)?;
        //     let chess_match = match Self::chess_matches(match_id) {
        //         Some(m) => m,
        //         None => return Err(Error::<T>::NonExistentMatch.into()),
        //     };

        //     ensure!(
        //         (chess_match.state == MatchState::OnGoing(NextMove::Whites))
        //             | (chess_match.state == MatchState::OnGoing(NextMove::Blacks)),
        //         Error::<T>::MatchNotOnGoing
        //     );

        //     let now = <frame_system::Pallet<T>>::block_number();
        //     let diff = now - chess_match.last_move;

        //     let expired: bool = match chess_match.style {
        //         MatchStyle::Bullet => diff > T::BulletPeriod::get(),
        //         MatchStyle::Blitz => diff > T::BlitzPeriod::get(),
        //         MatchStyle::Rapid => diff > T::RapidPeriod::get(),
        //         MatchStyle::Daily => diff > T::DailyPeriod::get(),
        //     };

        //     ensure!(expired, Error::<T>::MoveNotExpired);

        //     let winner = match chess_match.state {
        //         MatchState::OnGoing(NextMove::Whites) => chess_match.opponent.clone(),
        //         _ => chess_match.challenger.clone(),
        //     };

        //     Self::deposit_event(Event::MatchWon(
        //         match_id,
        //         winner.clone(),
        //         chess_match.board.clone(),
        //     ));

        //     let abandoned: bool = match chess_match.style {
        //         MatchStyle::Bullet => diff > T::BulletPeriod::get() * 10u32.into(),
        //         MatchStyle::Blitz => diff > T::BlitzPeriod::get() * 10u32.into(),
        //         MatchStyle::Rapid => diff > T::RapidPeriod::get() * 10u32.into(),
        //         MatchStyle::Daily => diff > T::DailyPeriod::get() * 10u32.into(),
        //     };

        //     if (who == chess_match.challenger) | (who == chess_match.opponent) | !abandoned {
        //         // winner gets both deposits before match becomes abandoned
        //         match chess_match.win_bet(&winner) {
        //             Ok(()) => {}
        //             Err(_) => Self::deposit_event(Event::MatchAwardError(match_id, winner.clone())),
        //         }
        //     } else {
        //         // who cleared the match after match is abandoned gets the incentive,
        //         // and the winner gets both deposits minus the incentive share
        //         match chess_match.clear_abandoned_bet(&winner, &who) {
        //             Ok(()) => {}
        //             Err(_) => Self::deposit_event(Event::MatchClearanceError(
        //                 match_id,
        //                 winner.clone(),
        //                 who,
        //             )),
        //         }
        //     }

        //     // update elo rating
        //     let looser = if chess_match.challenger == winner {
        //         chess_match.opponent.clone()
        //     } else {
        //         chess_match.challenger.clone()
        //     };
        //     Self::update_elo(winner.clone(), 1_f32, looser, 0_f32);

        //     <Matches<T>>::remove(match_id);
        //     <PlayerMatches<T>>::remove(chess_match.challenger.clone(), match_id);
        //     <PlayerMatches<T>>::remove(chess_match.opponent, match_id);
        //     <MatchIdFromNonce<T>>::remove(chess_match.nonce);

        //     Ok(())
        // }
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

        fn update_elo(player1: T::AccountId, score_1: f32, player2: T::AccountId, score_2: f32) {
            let rating1 = Self::player_elo(&player1) as f32;
            let rating2 = Self::player_elo(&player2) as f32;
            let transformed_rating1 = 10_f32.powf(rating1 / 400_f32);
            let transformed_rating2 = 10_f32.powf(rating2 / 400_f32);
            let expected_score1 =
                &transformed_rating1 / (&transformed_rating1 + &transformed_rating2);
            let expected_score2 =
                &transformed_rating2 / (&transformed_rating1 + &transformed_rating2);
            let new_rating1 =
                (&rating1 + T::K::get() as f32 * (score_1 - expected_score1)).round() as u16;
            let new_rating2 =
                (&rating2 + T::K::get() as f32 * (score_2 - expected_score2)).round() as u16;
            <PlayerElo<T>>::insert(&player1, new_rating1);
            <PlayerElo<T>>::insert(&player2, new_rating2);
        }
    }
}
