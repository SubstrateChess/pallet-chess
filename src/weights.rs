
//! Autogenerated weights for `pallet_chess`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-11-30, STEPS: `1248`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `bernardo-benchmarking`, CPU: `AMD EPYC 7B13`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/node-template
// benchmark
// pallet
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_chess
// --extrinsic
// *
// --steps
// 1248
// --repeat
// 20
// --output
// weight.rs
// --json-file=benchmarks.json

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_match() -> Weight;
	fn abort_match() -> Weight;
	fn join_match() -> Weight;
	fn make_move() -> Weight;
	fn clear_abandoned_match() -> Weight;
}

/// Weight functions for `pallet_chess`.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Chess NextNonce (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Chess Matches (r:0 w:1)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn create_match() -> Weight {
		// Minimum execution time: 88_030 nanoseconds.
		(91_090_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn abort_match() -> Weight {
		// Minimum execution time: 82_190 nanoseconds.
		(83_430_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn join_match() -> Weight {
		// Minimum execution time: 70_710 nanoseconds.
		(72_110_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Chess Matches (r:1 w:1)
	// read `pallet-chess/docs` to understand how this weight was calculated.
	fn make_move() -> Weight {
		// Minimum execution time: 35_470 nanoseconds.
		(116_079_054 as Weight)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:3 w:3)
	// Storage: System Account (r:2 w:2)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn clear_abandoned_match() -> Weight {
		// Minimum execution time: 120_950 nanoseconds.
		(122_610_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(8))
	}
}

impl WeightInfo for () {
	// Storage: Chess NextNonce (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Chess Matches (r:0 w:1)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn create_match() -> Weight {
		// Minimum execution time: 88_030 nanoseconds.
		(91_090_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(7))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	// Storage: System Account (r:1 w:1)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn abort_match() -> Weight {
		// Minimum execution time: 82_190 nanoseconds.
		(83_430_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:2 w:2)
	fn join_match() -> Weight {
		// Minimum execution time: 70_710 nanoseconds.
		(72_110_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: Chess Matches (r:1 w:1)
	// read `pallet-chess/docs` to understand how this weight was calculated.
	fn make_move() -> Weight {
		// Minimum execution time: 35_470 nanoseconds.
		(116_079_054 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Chess Matches (r:1 w:1)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:3 w:3)
	// Storage: System Account (r:2 w:2)
	// Storage: Chess MatchIdFromNonce (r:0 w:1)
	fn clear_abandoned_match() -> Weight {
		// Minimum execution time: 120_950 nanoseconds.
		(122_610_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
}
