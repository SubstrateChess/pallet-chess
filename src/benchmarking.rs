use super::*;
use cozy_chess::{
	get_bishop_rays, get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_quiets,
	get_rook_rays, BitBoard, Board, Move, Piece,
};
use log;

#[allow(unused)]
use crate::Pallet as Chess;
//use crate::mock::*;
use frame_benchmarking::{account, benchmarks, vec, Vec};
use frame_system::{Pallet as System, RawOrigin};
//use pallet_assets::Pallet as Assets;
use scale_info::prelude::{format, string::String};
use sp_runtime::traits::Get;
use sp_runtime::SaturatedConversion;

const MOVES_PER_POSITION: u32 = 52;
const INITIAL_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const POSITIONS: &[&str] = &[
	"Q7/5Q2/8/8/3k4/6P1/6BP/7K b - - 0 67",
	"r4rk1/p4ppp/1q2p3/2n1P3/2p5/3bRNP1/1P3PBP/R2Q2K1 b - - 0 24",
	"r1bq1rk1/pp3ppp/2nbpn2/3p4/3P4/1PN1PN2/1BP1BPPP/R2Q1RK1 b - - 2 10",
	"1r4k1/1P3p2/6pp/2Pp4/4P3/PQ1K1R2/6P1/4q3 w - - 0 51",
	"8/8/R7/4n3/4k3/6P1/6K1/8 w - - 68 164",
	"2r3k1/1b4bp/1p2p1p1/3pNp2/3P1P1q/PB1Q3P/1P4P1/4R1K1 w - - 2 36",
	"4rrk1/1b4bp/p1p1p1p1/3pN3/1P3q2/PQN3P1/2P1RP1P/3R2K1 b - - 0 24",
	"rnbq1rk1/ppp1bppp/4p3/3pP1n1/2PP3P/5PP1/PP4B1/RNBQK1NR b KQ - 0 8",
	"3r1r1k/p1p3pp/2p5/8/4K3/2N3Pb/PPP5/R1B4R b - - 0 20",
	"r4k1r/ppq2ppp/4bB2/8/2p5/4P3/P3BPPP/1R1Q1RK1 b - - 0 17",
	"r4rk1/1b1nq1pp/p7/3pNp2/1p3Q2/3B3P/PPP1N1R1/R2K4 w - - 2 21",
	"8/5p2/8/p6k/8/3N4/5PPK/8 w - - 0 49",
	"2r1rbk1/4pp1p/1Q1P1np1/2B1Nq2/P4P2/1B3P2/1PP3bP/1K1RR3 b - - 0 29",
	"6k1/p4ppp/Bpp5/4P3/P7/4QKPb/2P3N1/3r3q w - - 5 36",
	"3br1k1/pp1r1ppp/3pbn2/P2Np3/1PPpP3/3P1NP1/5PBP/3RR1K1 w - - 1 21",
	"8/1p6/p3n3/4k3/8/6PR/1rr5/3R2K1 w - - 8 54",
	"1r4k1/p4p1p/5p2/8/4P3/4K3/PPP3P1/4R3 w - - 0 34",
	"6k1/6p1/7p/7R/7P/5n2/P3K1b1/8 b - - 2 48",
	"2rr2k1/pp5p/3p4/4p3/2b1p3/P4QP1/1P4P1/3R2K1 w - - 0 28",
	"q1r4k/1bR5/rp4pB/3p4/3P2nQ/8/PP3PPP/R5K1 w - - 1 29",
	"rnbqkbnr/pppppp1p/6p1/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
	"rnbqk1nr/1p3ppp/p3p3/2bp4/4P3/5N2/PPPN1PPP/R1BQKB1R w KQkq - 0 6",
	"r2q1rk1/1p1b1p1p/p5p1/3QP3/8/5N2/PP3PPP/2KR3R b - - 0 20",
	"r3r2k/pbp1q2p/1p6/4n3/2NQ4/2P2pB1/P1P2P1P/2R2RK1 b - - 6 26",
	"8/1p2k3/4rp2/p2R3Q/2q2B2/6P1/5P1P/6K1 b - - 14 73",
];

fn generate_moves(board_fen: &str) -> Vec<String> {
	let board_vec: Vec<Board> = vec![board_fen.parse().unwrap()];
	let promos: &Vec<Option<Piece>> = &Piece::ALL.into_iter().map(Some).chain([None]).collect();

	let pawn_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::Pawn) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					(get_pawn_quiets(from, board.side_to_move(), BitBoard::EMPTY)
						| get_pawn_attacks(from, board.side_to_move()))
					.iter()
					.flat_map(move |to| {
						promos.iter().map(move |&promotion| {
							let move_obj = Move { from, to, promotion };
							(board.is_legal(move_obj), format!("{}", move_obj))
						})
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let rook_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::Rook) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					get_rook_rays(from).iter().map(move |to| {
						let move_obj = Move { from, to, promotion: None };
						(board.is_legal(move_obj), format!("{}", move_obj))
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let knight_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::Knight) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					get_knight_moves(from).iter().map(move |to| {
						let move_obj = Move { from, to, promotion: None };
						(board.is_legal(move_obj), format!("{}", move_obj))
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let bishop_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::Bishop) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					get_bishop_rays(from).iter().map(move |to| {
						let move_obj = Move { from, to, promotion: None };
						(board.is_legal(move_obj), format!("{}", move_obj))
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let queen_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::Queen) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					(get_rook_rays(from) | get_bishop_rays(from)).iter().map(move |to| {
						let move_obj = Move { from, to, promotion: None };
						(board.is_legal(move_obj), format!("{}", move_obj))
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let king_moves: Vec<_> = board_vec
		.iter()
		.flat_map(move |board| {
			(board.pieces(Piece::King) & board.colors(board.side_to_move()))
				.iter()
				.flat_map(move |from| {
					get_king_moves(from).iter().map(move |to| {
						let move_obj = Move { from, to, promotion: None };
						(board.is_legal(move_obj), format!("{}", move_obj))
					})
				})
		})
		.filter(|(l, _)| *l)
		.map(|(_, m)| m)
		.collect();

	let mut all_moves: Vec<_> = Vec::new();
	all_moves.extend(pawn_moves);
	all_moves.extend(rook_moves);
	all_moves.extend(bishop_moves);
	all_moves.extend(knight_moves);
	all_moves.extend(queen_moves);
	all_moves.extend(king_moves);

	// we repeat moves until all_moves has MOVES_PER_POSITION elements
	let all_moves_len = all_moves.len();
	let mut repeat_moves: Vec<_> = Vec::new();
	let mut i = 0;
	while all_moves_len + repeat_moves.len() < MOVES_PER_POSITION as usize {
		repeat_moves.push(all_moves[i % all_moves_len].clone());
		i += 1;
	}

	all_moves.extend(repeat_moves);
	all_moves
}

pub const ASSET_ID: u32 = 200u32;
pub const ASSET_MIN_BALANCE: u64 = 1_000u64;

benchmarks! {
	where_clause {
		where
			AssetIdOf<T>: From<u32>,
			BalanceOf<T>: From<u64>,
			T::BlockNumber: From<u32>,
			// T: pallet_assets::Config,
			// <T as pallet_assets::Config>::AssetId: From<u32>,
			// <T as pallet_assets::Config>::Balance: From<u64>,
	}

	create_match {
		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
		let bet_asset_id = ASSET_ID;
		let bet_amount = ASSET_MIN_BALANCE * 10;
	}: _(RawOrigin::Signed(challenger.clone()), opponent.clone(), MatchStyle::Bullet, bet_asset_id.into(), bet_amount.into())
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
		let bet_asset_id = ASSET_ID;
		let bet_amount = ASSET_MIN_BALANCE * 10;
		Chess::<T>::create_match(RawOrigin::Signed(challenger.clone()).into(), opponent.clone(), MatchStyle::Bullet, bet_asset_id.into(), bet_amount.into()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
	}: _(RawOrigin::Signed(challenger), match_id)
	verify {
		assert!(Chess::<T>::chess_matches(match_id).is_none());
		assert!(Chess::<T>::chess_match_id_from_nonce(0).is_none());
	}

	join_match {
		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
		let bet_asset_id = ASSET_ID;
		let bet_amount = ASSET_MIN_BALANCE * 10;
		Chess::<T>::create_match(RawOrigin::Signed(challenger.clone()).into(), opponent.clone(), MatchStyle::Bullet, bet_asset_id.into(), bet_amount.into()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
	}: _(RawOrigin::Signed(opponent), match_id)
	verify {
		let chess_match = Chess::<T>::chess_matches(match_id).unwrap();
		assert_eq!(chess_match.state, MatchState::OnGoing(NextMove::Whites));
	}

	make_move {
		let i in 0 .. ((POSITIONS.len() as u32 - 1) * MOVES_PER_POSITION) as u32;

		let position_index = (i / MOVES_PER_POSITION) as usize;
		let position_to_benchmark = POSITIONS[position_index];

		let position_moves = generate_moves(position_to_benchmark);

		let move_index = (i % MOVES_PER_POSITION) as usize;
		let move_to_benchmark = &position_moves[move_index];

		log::info!("i: {}, pos_index: {}, move_index:{}, move: {}, board: {}", i, position_index, move_index, move_to_benchmark, position_to_benchmark);

		let challenger: T::AccountId = account("Alice", 0, 0);
		let opponent: T::AccountId = account("Bob", 0, 1);
		let bet_asset_id = ASSET_ID;
		let bet_amount = ASSET_MIN_BALANCE * 10;

		Chess::<T>::create_match(RawOrigin::Signed(challenger.clone()).into(), opponent.clone(), MatchStyle::Bullet, bet_asset_id.into(), bet_amount.into()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
		Chess::<T>::join_match(RawOrigin::Signed(opponent.clone()).into(), match_id).unwrap();

		Chess::<T>::force_board_state(match_id, position_to_benchmark.as_bytes().to_vec()).unwrap();
		let chess_match = Chess::<T>::chess_matches(match_id).unwrap();

		let player = match chess_match.state {
			MatchState::OnGoing(NextMove::Whites) => challenger,
			MatchState::OnGoing(NextMove::Blacks) => opponent,
			_ => panic!("invalid match state! nothing to benchmark..."),
		};

	}: _(RawOrigin::Signed(player), match_id, move_to_benchmark.as_str().into())

	clear_abandoned_match {
		let alice: T::AccountId = account("Alice", 0, 0);
		let bob: T::AccountId = account("Bob", 0, 1);
		let janitor: T::AccountId = account("Charlie", 0, 2);
		let bet_asset_id = ASSET_ID;
		let bet_amount = ASSET_MIN_BALANCE * 10;

		// let initial_balance_a = Assets::<T>::balance(bet_asset_id.into(), alice.clone());
		// let initial_balance_b = Assets::<T>::balance(bet_asset_id.into(), bob.clone());
		// let initial_balance_c = Assets::<T>::balance(bet_asset_id.into(), janitor.clone());

		Chess::<T>::create_match(RawOrigin::Signed(alice.clone()).into(), bob.clone(), MatchStyle::Bullet, bet_asset_id.into(), bet_amount.into()).unwrap();
		let match_id = Chess::<T>::chess_match_id_from_nonce(0).unwrap();
		Chess::<T>::join_match(RawOrigin::Signed(bob.clone()).into(), match_id).unwrap();
		Chess::<T>::make_move(RawOrigin::Signed(alice.clone()).into(), match_id, "e2e4".into()).unwrap();

		let chess_match: pallet::Match<T> = Chess::chess_matches(match_id).unwrap();
		let (janitor_incentive, actual_prize): (BalanceOf<T>, BalanceOf<T>) = chess_match.janitor_incentive();
		let (janitor_incentive, actual_prize): (u64, u64) = (janitor_incentive.saturated_into(), actual_prize.saturated_into());

		// advance the block number to the point where Bob's time-to-move is expired
		// and Alice's time to claim victory is also expired
		System::<T>::set_block_number(
			System::<T>::block_number() + <T as Config>::BulletPeriod::get() * 10u32.into() + 1u32.into(),
		);
	}: _(RawOrigin::Signed(janitor.clone()), match_id)
	verify {
		assert!(Chess::<T>::chess_matches(match_id).is_none());
		assert!(Chess::<T>::chess_match_id_from_nonce(0).is_none());

		// let final_balance_a = Assets::<T>::balance(bet_asset_id.into(), alice);
		// let final_balance_b = Assets::<T>::balance(bet_asset_id.into(), bob);
		// let final_balance_c = Assets::<T>::balance(bet_asset_id.into(), janitor);
		// assert_eq!(final_balance_a, initial_balance_a - bet_amount.into() + actual_prize.into());
		// assert_eq!(final_balance_b, initial_balance_b - bet_amount.into());
		// assert_eq!(final_balance_c, initial_balance_c + janitor_incentive.into());
	}

	impl_benchmark_test_suite!(Chess, crate::mock::new_test_ext(), crate::mock::Test);
}
