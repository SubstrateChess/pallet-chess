use crate::{mock::*, Config, Error, Event, MatchState, MatchStyle, NextMove, PlayerMatches};
use cozy_chess::Board;
use frame_benchmarking::account;
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_match_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();
        let bet_amount_low = AssetMinBalance::get() * 4; // assuming T::IncentiveShare is 10%

        // assert BetTooLow error
        assert_noop!(
            Chess::create_match(
                RuntimeOrigin::signed(alice),
                bob,
                MatchStyle::Bullet,
                bet_asset_id,
                bet_amount_low
            ),
            Error::<Test>::BetTooLow
        );

        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%
        let bet_asset_id_noop = AssetId::get() + 1;

        // assert BetDoesNotExist error
        assert_noop!(
            Chess::create_match(
                RuntimeOrigin::signed(alice),
                bob,
                MatchStyle::Bullet,
                bet_asset_id_noop,
                bet_amount
            ),
            Error::<Test>::BetDoesNotExist
        );

        // assert InvalidOpponent error
        assert_noop!(
            Chess::create_match(
                RuntimeOrigin::signed(alice),
                alice,
                MatchStyle::Bullet,
                bet_asset_id,
                bet_amount
            ),
            Error::<Test>::InvalidOpponent
        );

        // assert successful create_match
        let initial_balance_a = Assets::balance(bet_asset_id, alice);

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();
        let chess_match = Chess::chess_matches(match_id).unwrap();

        assert_eq!(chess_match.challenger, alice);
        assert_eq!(chess_match.opponent, bob);
        assert_eq!(
            chess_match.board,
            Board::default().to_string().as_bytes().to_vec()
        );
        assert_eq!(chess_match.state, MatchState::AwaitingOpponent);
        assert_eq!(chess_match.nonce, 0);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        assert_eq!(final_balance_a, initial_balance_a - bet_amount);
    });
}

#[test]
fn abort_match_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();

        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_noop!(
            Chess::abort_match(RuntimeOrigin::signed(bob), match_id),
            Error::<Test>::NotMatchChallenger
        );

        assert_ok!(Chess::abort_match(RuntimeOrigin::signed(alice), match_id));

        assert_eq!(Chess::chess_matches(match_id), None);
        assert_eq!(Chess::chess_match_id_from_nonce(0), None);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        assert_eq!(final_balance_a, initial_balance_a);
    });
}

#[test]
fn join_match_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();

        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let initial_balance_b = Assets::balance(bet_asset_id, bob);

        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));

        let chess_match = Chess::chess_matches(match_id).unwrap();
        assert_eq!(chess_match.state, MatchState::OnGoing(NextMove::Whites));

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        let final_balance_b = Assets::balance(bet_asset_id, bob);
        assert_eq!(final_balance_a, initial_balance_a - bet_amount);
        assert_eq!(final_balance_b, initial_balance_b - bet_amount);
    });
}

#[test]
fn make_move_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();

        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let initial_balance_b = Assets::balance(bet_asset_id, bob);

        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));

        // test successful make_move
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "e2e4".into()
        ));
        System::assert_last_event(
            Event::MoveExecuted {
                0: match_id,
                1: alice,
                2: "e2e4".into(),
            }
            .into(),
        );

        // test NotYourTurn error
        assert_noop!(
            Chess::make_move(RuntimeOrigin::signed(alice), match_id, "e7e5".into()),
            Error::<Test>::NotYourTurn
        );

        // test IllegalMove error
        assert_noop!(
            Chess::make_move(RuntimeOrigin::signed(bob), match_id, "e2e4".into()),
            Error::<Test>::IllegalMove
        );

        // test InvalidMoveEncoding
        assert_noop!(
            Chess::make_move(RuntimeOrigin::signed(bob), match_id, "1234".into()),
            Error::<Test>::InvalidMoveEncoding
        );

        // test InvalidMoveEncoding
        assert_noop!(
            Chess::make_move(RuntimeOrigin::signed(bob), match_id, "e1e2e3".into()),
            Error::<Test>::InvalidMoveEncoding
        );

        // test InvalidMoveEncoding
        assert_noop!(
            Chess::make_move(RuntimeOrigin::signed(bob), match_id, "1".into()),
            Error::<Test>::InvalidMoveEncoding
        );

        // test MatchWon
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "e7e5".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "g1f3".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "b8c6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "d2d4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "e5d4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "f3d4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "f8c5".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "c2c3".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "d8f6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "d4c6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "f6f2".into()
        ));
        System::assert_has_event(
            Event::MatchWon {
                0: match_id,
                1: bob,
                2: "r1b1k1nr/pppp1ppp/2N5/2b5/4P3/2P5/PP3qPP/RNBQKB1R w KQkq - 0 7".into(),
            }
            .into(),
        );
        assert_eq!(Chess::chess_matches(match_id), None);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        let final_balance_b = Assets::balance(bet_asset_id, bob);
        assert_eq!(final_balance_a, initial_balance_a - bet_amount);
        assert_eq!(final_balance_b, initial_balance_b + bet_amount);

        // --------------------------------------
        // test MatchDrawn
        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let initial_balance_b = Assets::balance(bet_asset_id, bob);

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(1).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));

        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "c2c4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "h7h5".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "h2h4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "a7a5".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "d1a4".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "a8a6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "a4a5".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "a6h6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "a5c7".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "f7f6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "c7d7".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "e8f7".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "d7b7".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "d8d3".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "b7b8".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "d3h7".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "b8c8".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(bob),
            match_id,
            "f7g6".into()
        ));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "c8e6".into()
        ));
        System::assert_has_event(
            Event::MatchDrawn {
                0: match_id,
                1: "5bnr/4p1pq/4Qpkr/7p/2P4P/8/PP1PPPP1/RNB1KBNR b KQ - 2 10".into(),
            }
            .into(),
        );
        assert_eq!(Chess::chess_matches(match_id), None);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        let final_balance_b = Assets::balance(bet_asset_id, bob);
        assert_eq!(final_balance_a, initial_balance_a);
        assert_eq!(final_balance_b, initial_balance_b);
    });
}

const BOARD_STATE: &str = "Q7/5Q2/8/8/3k4/6P1/6BP/7K b - - 0 67";

#[test]
fn force_board_state_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();
        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));

        assert_ok!(Chess::force_board_state(match_id, BOARD_STATE.into()));

        let chess_match = Chess::chess_matches(match_id).unwrap();
        assert_eq!(chess_match.board, BOARD_STATE.as_bytes());
    });
}

#[test]
fn claim_victory_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);

        let bet_asset_id = AssetId::get();
        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let initial_balance_b = Assets::balance(bet_asset_id, bob);

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "e2e4".into()
        ));

        // advance the block number to the point where bob's time-to-move is expired
        System::set_block_number(
            System::block_number() + <Test as Config>::BulletPeriod::get() + 1,
        );

        // alice claims victory
        assert_ok!(Chess::clear_abandoned_match(
            RuntimeOrigin::signed(alice),
            match_id
        ));

        System::assert_has_event(
            Event::MatchWon {
                0: match_id,
                1: alice,
                2: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into(),
            }
            .into(),
        );

        assert_eq!(Chess::chess_matches(match_id), None);
        assert_eq!(Chess::chess_match_id_from_nonce(0), None);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        let final_balance_b = Assets::balance(bet_asset_id, bob);
        assert_eq!(final_balance_a, initial_balance_a + bet_amount);
        assert_eq!(final_balance_b, initial_balance_b - bet_amount);
    });
}

#[test]
fn janitor_incentive_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);
        let charlie = account("Charlie", 0, 2);

        let bet_asset_id = AssetId::get();
        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        let initial_balance_a = Assets::balance(bet_asset_id, alice);
        let initial_balance_b = Assets::balance(bet_asset_id, bob);
        let initial_balance_c = Assets::balance(bet_asset_id, charlie);

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));
        assert_ok!(Chess::make_move(
            RuntimeOrigin::signed(alice),
            match_id,
            "e2e4".into()
        ));

        let chess_match = Chess::chess_matches(match_id).unwrap();
        let (janitor_incentive, actual_prize) = chess_match.janitor_incentive();

        // advance the block number to the point where bob's time-to-move is expired
        // and alice's time to claim victory is also expired
        System::set_block_number(
            System::block_number() + <Test as Config>::BulletPeriod::get() * 10 + 1,
        );

        // charlie cleans abandoned match
        assert_ok!(Chess::clear_abandoned_match(
            RuntimeOrigin::signed(charlie),
            match_id
        ));

        System::assert_has_event(
            Event::MatchWon {
                0: match_id,
                1: alice,
                2: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into(),
            }
            .into(),
        );

        assert_eq!(Chess::chess_matches(match_id), None);
        assert_eq!(Chess::chess_match_id_from_nonce(0), None);

        let final_balance_a = Assets::balance(bet_asset_id, alice);
        let final_balance_b = Assets::balance(bet_asset_id, bob);
        let final_balance_c = Assets::balance(bet_asset_id, charlie);
        assert_eq!(
            final_balance_a,
            initial_balance_a - bet_amount + actual_prize
        );
        assert_eq!(final_balance_b, initial_balance_b - bet_amount);
        assert_eq!(final_balance_c, initial_balance_c + janitor_incentive);
    });
}

#[test]
fn get_player_matches_works() {
    new_test_ext().execute_with(|| {
        let alice = account("Alice", 0, 0);
        let bob = account("Bob", 0, 1);
        let charlie = account("Charlie", 0, 2);

        let bet_asset_id = AssetId::get();
        let bet_amount = AssetMinBalance::get() * 5; // assuming T::IncentiveShare is 10%

        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            bob,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));

        let match_id = Chess::chess_match_id_from_nonce(0).unwrap();
        let mut alice_matches = PlayerMatches::<Test>::iter_key_prefix(alice).collect::<Vec<_>>();
        let mut bob_matches = PlayerMatches::<Test>::iter_key_prefix(bob).collect::<Vec<_>>();

        assert_eq!(alice_matches[0], match_id);
        assert_eq!(bob_matches[0], match_id);

        assert_ok!(Chess::join_match(RuntimeOrigin::signed(bob), match_id));
        assert_ok!(Chess::create_match(
            RuntimeOrigin::signed(alice),
            charlie,
            MatchStyle::Bullet,
            bet_asset_id,
            bet_amount
        ));
        let new_match_id = Chess::chess_match_id_from_nonce(1).unwrap();

        alice_matches = PlayerMatches::<Test>::iter_key_prefix(alice).collect::<Vec<_>>();
        bob_matches = PlayerMatches::<Test>::iter_key_prefix(bob).collect::<Vec<_>>();

        assert_eq!(alice_matches[0], match_id);
        assert_eq!(alice_matches[1], new_match_id);
        assert_eq!(bob_matches[0], match_id);

        // advance the block number to the point where bob's time-to-move is expired
        System::set_block_number(
            System::block_number() + <Test as Config>::BulletPeriod::get() + 1,
        );

        assert_ok!(Chess::clear_abandoned_match(
            RuntimeOrigin::signed(alice),
            match_id
        ));

        alice_matches = PlayerMatches::<Test>::iter_key_prefix(alice).collect::<Vec<_>>();
        bob_matches = PlayerMatches::<Test>::iter_key_prefix(bob).collect::<Vec<_>>();
        assert_eq!(alice_matches[0], new_match_id);
        assert_eq!(bob_matches.len(), 0);
    });
}
