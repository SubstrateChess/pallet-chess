use super::*;

#[allow(unused)]
use crate::Pallet as Chess;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const INITIAL_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

benchmarks! {
	create_match {
		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
	}: _(RawOrigin::Signed(challenger.clone()), opponent.clone())
	verify {
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
		let chess_match = Chess::<T>::chess_matches(match_id).unwrap();
		assert_eq!(chess_match.challenger, challenger);
		assert_eq!(chess_match.opponent, opponent);
		assert_eq!(chess_match.board, INITIAL_BOARD.as_bytes().to_vec());
		assert_eq!(chess_match.state, MatchState::AwaitingOpponent);
		assert_eq!(chess_match.nonce, 0);
	}

	abort_match {
		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
		Chess::<T>::create_match(RawOrigin::Signed(challenger.clone()).into(), opponent.clone()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
	}: _(RawOrigin::Signed(challenger), match_id)
	verify {
		assert!(Chess::<T>::chess_matches(match_id).is_none());
		assert!(Chess::<T>::chess_match_id_from_nonce(0).is_none());
	}

	join_match {
		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
		Chess::<T>::create_match(RawOrigin::Signed(challenger.clone()).into(), opponent.clone()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
	}: _(RawOrigin::Signed(opponent), match_id)
	verify {
		let chess_match = Chess::<T>::chess_matches(match_id).unwrap();
		assert_eq!(chess_match.state, MatchState::OnGoing(NextMove::Whites));
	}

	impl_benchmark_test_suite!(Chess, crate::mock::new_test_ext(), crate::mock::Test);
}
