use crate::{mock::*, MatchState};
use chess::Game;
use frame_support::assert_ok;

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
		assert_eq!(
			chess_match.board,
			Game::new().current_position().to_string().as_bytes().to_vec()
		);
		assert_eq!(chess_match.state, MatchState::AwaitingOpponent);

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
