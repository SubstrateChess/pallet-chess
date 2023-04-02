# ♜♞♝♛♚ Substrate Chess ♔♕♗♘♖

**Note**: this branch contains `pallet_chess` functionality refactored for [`GMorDie`](https://github.com/GMorDIE/gm-chain/).

It has a loose coupling with `orml_currencies` via the `MultiCurrency` trait, instead of `pallet_assets` via `Inspect` + `Transfer` traits from `frame_support::traits::fungibles`.

## Overview

This pallet provides a way to play on-chain chess. It benefits from [`cozy-chess`](https://crates.io/crates/cozy-chess) and its ability to compile to WASM out-of-the-box (`no_std` compatible).

The chess board is represented on-chain as a [*Forsyth–Edwards Notation* (FEN)](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) string, as well as the moves.

### Players

Each match has two players:
- Challenger (Whites)
- Opponent (Blacks)

### Match

When Challenger calls `create_match`, they establish the following parameters:
- Opponent Address
- Style (`Bullet`, `Blitz`, `Rapid`, or `Daily`)
- Bet Asset Id
- Bet Amount

A Match Id is calculated by hashing the tuple `(challenger, opponent, nonce)`, where the `nonce` is incremented for every new match created.

#### Match Bets

In order to create a match, Challenger chooses an Asset Id and an amount. During the execution of `create_match`, a deposit of such asset amount is made from their account.

As soon as Opponent calls `join_match`, an equal deposit is made from their account.

The winner of the match receives both deposits as reward. In case of draws, both players get their deposits back.

#### Match Style

Match styles define how much time each player has to make their move. Time is measured in blocks, and each style is defined as a `Config` type.

Assuming 6s per block, the following values are recommended:
- `BulletPeriod`: 10 blocks (~1 minute)
- `BlitzPeriod`: 50 blocks (~5 minutes)
- `RapidPeriod`: 150 blocks (~15 minutes)
- `DailyPeriod`: 14400 blocks (~1 day)

In case player `A` takes longer than the expected time for their move, then player `B` has the right to call `clear_abandoned_match` and claim victory, taking both deposits.

Bet deposits must cover janitor incentives such that `2 * Bet * IncentiveShare >= MinimumBalance`.
For example, if the asset has `MinimumBalance = 100` and `IncentiveShare = 10%`, then the minimum allowed deposit is `500`.

### Extrinsic Weights

Although conveniently able to compile to WASM, `cozy_chess` crate wasn't written with Substrate in mind. That means that there is no guarantee that its execution will be linear. This has direct implications on how the extrinsic weights are calculated for this pallet.

The [`docs`](docs/) directory has a detailed description on the strategy used for benchmarking the extrinsic weights.