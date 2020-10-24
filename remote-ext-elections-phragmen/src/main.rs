//! A remote ext template for elections-phragmen pallet in a live chain.
//!
//! Note: You need to have this repo cloned next to a local substrate repo, and have all the
//! dependencies pointed to a local one. To do so, use `node update_cargo.js local`.

use frame_support::impl_outer_origin;
use frame_support::{parameter_types, IterableStorageMap, StorageMap, weights::constants::RocksDbWeight};
use sp_core::H256;
use sp_runtime::traits::Convert;
use sp_runtime::traits::IdentityLookup;
use pallet_elections_phragmen::*;
use frame_support::migration::*;

pub struct CurrencyToVoteHandler;
impl CurrencyToVoteHandler {
	fn factor() -> Balance {
		(Balances::total_issuance() / u64::max_value() as Balance).max(1)
	}
}
impl Convert<Balance, u64> for CurrencyToVoteHandler {
	fn convert(x: Balance) -> u64 {
		(x / Self::factor()) as u64
	}
}
impl Convert<u128, Balance> for CurrencyToVoteHandler {
	fn convert(x: u128) -> Balance {
		x * Self::factor()
	}
}

macro_rules! init_log {
	() => {
		let _ = env_logger::try_init();
	};
}

pub(crate) type AccountId = sp_core::crypto::AccountId32;
pub(crate) type BlockNumber = u32;
pub(crate) type Balance = u128;
pub(crate) type Header = sp_runtime::generic::Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;

pub(crate) type Balances = pallet_balances::Module<Runtime>;
pub(crate) type Elections = pallet_elections_phragmen::Module<Runtime>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Runtime;

impl_outer_origin! {
	pub enum Origin for Runtime where system = frame_system {}
}

impl frame_system::Trait for Runtime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u32;
	type BlockNumber = BlockNumber;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = ();
	type MaximumBlockWeight = ();
	type DbWeight = RocksDbWeight;
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = ();
	type AvailableBlockRatio = ();
	type MaximumBlockLength = ();
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl pallet_balances::Trait for Runtime {
	type Balance = Balance;
	type MaxLocks = ();
	type Event = ();
	type DustRemoval = ();
	type ExistentialDeposit = ();
	type AccountStore = frame_system::Module<Runtime>;
	type WeightInfo = ();
}

parameter_types! {
	pub const ElectionsPhragmenModuleId: frame_support::traits::LockIdentifier = *b"phrelect";
	pub const DesiredMembers: u32 = 17;
	pub const DesiredRunnersUp: u32 = 13;
}

impl pallet_elections_phragmen::Trait for Runtime {
	type ModuleId = ElectionsPhragmenModuleId;
	type Event = ();
	type Currency = Balances;
	type CurrencyToVote = CurrencyToVoteHandler;
	type ChangeMembers = ();
	type InitializeMembers = ();
	type CandidacyBond = ();
	type VotingBondBase = ();
	type VotingBondFactor = ();
	type TermDuration = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type LoserCandidate = ();
	type KickedMember = ();
	type BadReport = ();
	type WeightInfo = ();
}

const URI: &'static str = "wss://kusama-rpc.polkadot.io";

#[async_std::main]
async fn main() -> () {

	init_log!();
	let client = sub_storage::create_ws_client(URI).await;
	let now = sub_storage::get_head(&client).await;

	remote_externalities::Builder::new()
		.module("PhragmenElection")
		.uri(URI.to_owned())
		.at(now)
		.build_async()
		.await
		.execute_with(|| {

			let random_voter = <StorageKeyIterator<AccountId, (Balance, Vec<AccountId>), frame_support::Twox64Concat>>::new(
				b"PhragmenElection",
				b"Voting",
			).skip(5)
			.take(1)
			.map(|(voter, (stake, votes))| (voter, stake, votes))
			.collect::<Vec<_>>();

			let (random_voter, stake, votes) = random_voter.first().unwrap();

			pallet_elections_phragmen::migrations::migrate_to_recorded_deposit::<Runtime>(1000_000);

			let voting = <Voting<Runtime>>::get(random_voter);
			assert_eq!(&voting.votes, votes);
			assert_eq!(voting.stake, *stake);
			assert_eq!(voting.deposit, 1000_000);
		})
}
