use crate::{self as pallet_chess};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_benchmarking::account;
use frame_support::{
    parameter_types,
    traits::{ConstU128, ConstU16, ConstU32, ConstU64, GenesisBuild, Nothing},
    RuntimeDebug,
};
use frame_system as system;
use orml_currencies::BasicCurrencyAdapter;
use orml_tokens::GetOpposite;
use orml_traits::arithmetic::One;
use orml_traits::parameter_type_with_key;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Currencies: orml_currencies,
        Tokens: orml_tokens,
        Chess: pallet_chess,
    }
);

pub type ReserveIdentifier = [u8; 8];

pub type AccountId = u64;
impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
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
    pub const IncentiveShare: u8 = 10; // janitor gets 10% of the prize
}

impl pallet_chess::Config for Test {
    type Event = Event;
    type ChessWeightInfo = pallet_chess::weights::SubstrateWeight<Test>;
    type MultiCurrency = Currencies;
    type BulletPeriod = BulletPeriod;
    type BlitzPeriod = BlitzPeriod;
    type RapidPeriod = RapidPeriod;
    type DailyPeriod = DailyPeriod;
    type IncentiveShare = IncentiveShare;
}

type CurrencyId = Coooooins;
type Balance = u128;

pub const MILLIUNIT: Balance = 1_000_000_000;
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        match *currency_id {
            Coooooins::FREN => EXISTENTIAL_DEPOSIT,
            Coooooins::GM | Coooooins::GN => One::one(),
        }
    };
}

parameter_types! {
    pub DustAccount: AccountId = 1337u64;
}

impl orml_tokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type Amount = i64;
    type CurrencyId = CurrencyId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = orml_tokens::TransferDust<Test, DustAccount>;
    type MaxLocks = ConstU32<100_000>;
    type MaxReserves = ConstU32<100_000>;
    type ReserveIdentifier = ReserveIdentifier;
    type DustRemovalWhitelist = Nothing;
}

#[derive(
    Encode,
    Decode,
    Eq,
    PartialEq,
    Copy,
    Clone,
    RuntimeDebug,
    PartialOrd,
    Ord,
    TypeInfo,
    MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum Coooooins {
    FREN,
    GM,
    GN,
}

impl GetOpposite for Coooooins {
    fn get_opposite(self) -> Self {
        match self {
            Coooooins::FREN => panic!(),
            Coooooins::GM => Coooooins::GN,
            Coooooins::GN => Coooooins::GM,
        }
    }
}

type BlockNumber = u32;

parameter_types! {
    pub const GetNativeCurrencyId: Coooooins = Coooooins::FREN;
    pub const GetGMCurrencyId: Coooooins = Coooooins::GM;
    pub const GetGNCurrencyId: Coooooins = Coooooins::GN;

    pub const PeriodLength: BlockNumber = 690u32;
}

pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Test, Balances, i64, u64>;

impl orml_currencies::Config for Test {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type GetGMCurrencyId = GetGMCurrencyId;
    type GetGNCurrencyId = GetGNCurrencyId;
    type PeriodLength = PeriodLength;
    type TreasuryAccount = ();
    type WeightInfo = ();
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ReserveIdentifier;
}

pub struct ExtBuilder {
    balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![] }
    }
}

impl ExtBuilder {
    pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn fund_alice_and_bob(self) -> Self {
        self.balances(vec![
            (
                account("Alice", 0, 0),
                GetNativeCurrencyId::get(),
                EXISTENTIAL_DEPOSIT * 10,
            ),
            (
                account("Bob", 0, 1),
                GetNativeCurrencyId::get(),
                EXISTENTIAL_DEPOSIT * 10,
            ),
            (
                account("Alice", 0, 0),
                GetGMCurrencyId::get(),
                EXISTENTIAL_DEPOSIT * 10,
            ),
            (
                account("Bob", 0, 1),
                GetGMCurrencyId::get(),
                EXISTENTIAL_DEPOSIT * 10,
            ),
        ])
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: self
                .balances
                .clone()
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id == GetNativeCurrencyId::get())
                .map(|(account_id, _, initial_balance)| (account_id, initial_balance))
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        orml_tokens::GenesisConfig::<Test> {
            balances: self
                .balances
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id != GetNativeCurrencyId::get())
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }
}
