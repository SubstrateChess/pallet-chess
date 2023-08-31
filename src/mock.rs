use crate::{self as pallet_chess};
use frame_support::{
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU16, ConstU32, ConstU64},
    PalletId,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub struct Test {
        System: frame_system,
        Balances: pallet_balances,
        Assets: pallet_assets,
        Chess: pallet_chess,
    }
);

impl system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const BulletPeriod: u64 = 10;
    pub const BlitzPeriod: u64 = 50;
    pub const RapidPeriod: u64 = 150;
    pub const DailyPeriod: u64 = 14400;
    pub const ChessPalletId: PalletId = PalletId(*b"subchess");
    pub const IncentiveShare: u8 = 10; // janitor gets 10% of the prize
}

impl pallet_chess::Config for Test {
    type PalletId = ChessPalletId;
    type K = ConstU16<32>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_chess::weights::SubstrateWeight<Test>;
    type Assets = Assets;
    type AssetBalance = u64;
    type BulletPeriod = BulletPeriod;
    type BlitzPeriod = BlitzPeriod;
    type RapidPeriod = RapidPeriod;
    type DailyPeriod = DailyPeriod;
    type IncentiveShare = IncentiveShare;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Balance = u64;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
}

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u64;
    type RemoveItemsLimit = ConstU32<5>;
    type AssetId = u32;
    type AssetIdParameter = u32;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU64<1>;
    type AssetAccountDeposit = ConstU64<10>;
    type MetadataDepositBase = ConstU64<1>;
    type MetadataDepositPerByte = ConstU64<1>;
    type ApprovalDeposit = ConstU64<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
}

pub const ASSET_ID: u32 = 200u32;
pub const ASSET_MIN_BALANCE: u64 = 1_000u64;

frame_support::parameter_types! {
    pub const AssetId: u32 = ASSET_ID;
    pub const AssetMinBalance: u64 = ASSET_MIN_BALANCE;
}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    let asset_id = AssetId::get();
    let asset_min_balance = AssetMinBalance::get();

    pallet_assets::GenesisConfig::<Test> {
        assets: vec![
            // id, owner, is_sufficient, min_balance
            (asset_id, 0, true, asset_min_balance),
        ],
        metadata: vec![
            // id, name, symbol, decimals
            (asset_id, "Token Name".into(), "TOKEN".into(), 10),
        ],
        accounts: vec![
            // id, account_id, balance
            (
                asset_id,
                frame_benchmarking::account("Alice", 0, 0),
                asset_min_balance * 100,
            ),
            (
                asset_id,
                frame_benchmarking::account("Bob", 0, 1),
                asset_min_balance * 100,
            ),
            (
                asset_id,
                frame_benchmarking::account("Charlie", 0, 2),
                asset_min_balance * 100,
            ),
            (
                asset_id,
                frame_benchmarking::account("Dave", 0, 3),
                asset_min_balance * 100,
            ),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    pallet_chess::GenesisConfig::<Test> {
        elo: vec![
            (frame_benchmarking::account("Alice", 0, 0), 2000),
            (frame_benchmarking::account("Bob", 0, 1), 2400),
            (frame_benchmarking::account("Charlie", 0, 2), 1000),
            (frame_benchmarking::account("Dave", 0, 3), 1000),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}
