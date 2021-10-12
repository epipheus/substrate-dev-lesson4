use super::*;

use crate as ponies;
use sp_core::H256;
use frame_support::{parameter_types, assert_ok, assert_noop};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Teast>;
type Block = frame_system::mocking::MockBLock<Test>;

// config mock runtime
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBLock = Block,
        UncheckedExtrinsic = UncheckExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        PoniesModule: ponies::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

impl Config for Test {
    type Event = Event;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t: system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    t.execute_with(|| System::set_block_number(1));
    t
}

#[test]
fn can_create() {
    new_test_ext().execute_with(|| {
        assert_ok!(PoniesModule::create(Origin::signed(100)));
        let pony = Pony([59, 250, 138, 82, 209, 39, 141, 109, 163, 238, 183, 145, 235, 168, 18, 122]);

        assert_eql!(PoniesModule::ponies(100,0), Some(pony.clone()));
        assert_eql!(PoniesModule::next_pony_id(),1);

        System::assert_last_event(Event::PoniesModule(crate::Event::<Test>::PoniesCreated(100,0,pony)));
    });
}

#[test]
fn gender() {
    assert_eq!(Pony[0;16].gender(), PonyGender::Male);
    assert_eq!(Pony([1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]).gender(),PonyGender::Female);
}

#[test]
fn can_breed() {
    new_test_ext().execute_with(|| {
        assert_ok!(PoniesModule::create(Origin::signed(100)));
        System::set_extrinsic_index(1);
        assert_ok!(PoniesModule::create(Origin::signed(100)));

        assert_noop!(PoniesModule::breed(Origin::signed(100), 0, 11), Error::<Test>::InvalidPonyId);
        assert_noop!(PoniesModule::breed(Origin::signed(100), 0, 0), Error::<Test>::SameGender);
        assert_noop!(PoniesModule::breed(Origin::signed(101), 0, 1), Error::<Test>::InvalidPonyId);

        assert_ok!(PoniesModule::breed(Origin::signed(100), 0, 1));

        let pony = Pony([59, 254, 219, 122, 245, 239, 191, 125, 255, 239, 247, 247, 251, 239, 247, 254]);

        assert_eq!(PoniesModule::ponies(100, 2), Some(pony.clone()));
        assert_eq!(PoniesModule::next_pony_id(), 3);

        System::assert_last_event(Event::PoniesModule(crate::Event::<Test>::PonyBred(100u64, 2u32, pony)));
    }
}