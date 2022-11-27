use crate::{mock::*, Error, Event, MatchState, NextMove};
use cozy_chess::Board;
use frame_support::{assert_noop, assert_ok};

const A: u64 = 1;
const B: u64 = 2;

#[test]
fn create_match_works() {
	new_test_ext().execute_with(|| {
		// todo: assert initial free balance of A

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));

		let match_id = Chess::chess_match_id_from_nonce(0).unwrap();
		let chess_match = Chess::chess_matches(match_id).unwrap();

		assert_eq!(chess_match.challenger, A);
		assert_eq!(chess_match.opponent, B);
		assert_eq!(chess_match.board, Board::default().to_string().as_bytes().to_vec());
		assert_eq!(chess_match.state, MatchState::AwaitingOpponent);
		assert_eq!(chess_match.nonce, 0);

		// todo: assert final free balance of A
	});
}

#[test]
fn abort_match_works() {
	new_test_ext().execute_with(|| {
		// todo: assert initial free balance of A

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));

		let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

		assert_noop!(
			Chess::abort_match(RuntimeOrigin::signed(B), match_id),
			Error::<Test>::NotMatchChallenger
		);

		assert_ok!(Chess::abort_match(RuntimeOrigin::signed(A), match_id));

		assert_eq!(Chess::chess_matches(match_id), None);
		assert_eq!(Chess::chess_match_id_from_nonce(0), None);

		// todo: assert final free balance of A
	});
}

#[test]
fn join_match_works() {
	new_test_ext().execute_with(|| {
		// todo: assert initial balance of A and B

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));
		let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

		assert_ok!(Chess::join_match(RuntimeOrigin::signed(B), match_id));

		let chess_match = Chess::chess_matches(match_id).unwrap();
		assert_eq!(chess_match.state, MatchState::OnGoing(NextMove::Whites));

		// todo: assert final balance of A and B
	});
}

#[test]
fn make_move_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));
		let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

		assert_ok!(Chess::join_match(RuntimeOrigin::signed(B), match_id));

		// test successful make_move
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "e2e4".into()));
		System::assert_last_event(
			Event::MoveExecuted { 0: match_id, 1: A, 2: "e2e4".into() }.into(),
		);

		// test NotYourTurn error
		assert_noop!(
			Chess::make_move(RuntimeOrigin::signed(A), match_id, "e7e5".into()),
			Error::<Test>::NotYourTurn
		);

		// test IllegalMove error
		assert_noop!(
			Chess::make_move(RuntimeOrigin::signed(B), match_id, "e2e4".into()),
			Error::<Test>::IllegalMove
		);

		// test InvalidMoveEncoding
		assert_noop!(
			Chess::make_move(RuntimeOrigin::signed(B), match_id, "123".into()),
			Error::<Test>::InvalidMoveEncoding
		);

		// test MatchWon
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "e7e5".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "g1f3".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "b8c6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "d2d4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "e5d4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "f3d4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "f8c5".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "c2c3".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "d8f6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "d4c6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "f6f2".into()));
		System::assert_last_event(
			Event::MatchWon {
				0: match_id,
				1: B,
				2: "r1b1k1nr/pppp1ppp/2N5/2b5/4P3/2P5/PP3qPP/RNBQKB1R w KQkq - 0 7".into(),
			}
			.into(),
		);
		assert_eq!(Chess::chess_matches(match_id), None);

		// test MatchDrawn
		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));
		let match_id = Chess::chess_match_id_from_nonce(1).unwrap();
		assert_ok!(Chess::join_match(RuntimeOrigin::signed(B), match_id));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "c2c4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "h7h5".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "h2h4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "a7a5".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "d1a4".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "a8a6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "a4a5".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "a6h6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "a5c7".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "f7f6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "c7d7".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "e8f7".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "d7b7".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "d8d3".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "b7b8".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "d3h7".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "b8c8".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(B), match_id, "f7g6".into()));
		assert_ok!(Chess::make_move(RuntimeOrigin::signed(A), match_id, "c8e6".into()));
		System::assert_last_event(
			Event::MatchDrawn {
				0: match_id,
				1: "5bnr/4p1pq/4Qpkr/7p/2P4P/8/PP1PPPP1/RNB1KBNR b KQ - 2 10".into(),
			}
			.into(),
		);
		assert_eq!(Chess::chess_matches(match_id), None);
	});
}
