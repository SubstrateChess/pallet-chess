use crate::{mock::*, Error, MatchState};
use cozy_chess::Board;
use frame_support::{assert_noop, assert_ok};

const A: u64 = 1;
const B: u64 = 2;

#[test]
fn create_match_works() {
	new_test_ext().execute_with(|| {
		// todo: assert initial free balance of A

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));

		let match_id = Chess::chess_match_id_from_nonce(0);
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

		let match_id = Chess::chess_match_id_from_nonce(0);

		assert_noop!(
			Chess::abort_match(RuntimeOrigin::signed(B), match_id),
			Error::<Test>::NotMatchChallenger
		);

		assert_ok!(Chess::abort_match(RuntimeOrigin::signed(A), match_id));

		assert_eq!(Chess::chess_matches(match_id), None);
		// todo: assert Chess::chess_match_id_from_nonce(0)

		// todo: assert final free balance of A
	});
}

#[test]
fn join_match_works() {
	new_test_ext().execute_with(|| {
		// todo: assert initial balance of A and B

		assert_ok!(Chess::create_match(RuntimeOrigin::signed(A), B));
		let match_id = Chess::chess_match_id_from_nonce(0);

		assert_ok!(Chess::join_match(RuntimeOrigin::signed(B), match_id));

		let chess_match = Chess::chess_matches(match_id).unwrap();
		assert_eq!(chess_match.state, MatchState::OnGoing);

		// todo: assert final balance of A and B
	});
}
