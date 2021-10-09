#![cfg_attr(not(feature = "std"),no_std)]

// go back to 41:16 also 35:27 double check everything

use frame_support::{
    pallet_prelude::*,
    traits::Randomness,
};
use frame_system::pallet_prelude::*;
use sp_runtime::ArithmeticError;
use sp_io::hashing::blake2_128;
pub use pallet::*;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub struct Pony(pub [u8; 16]);

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub enum PonyGender{
    Male,
    Female
}

impl Pony {
    pub fn gender(&self) -> PonyGender {
        if self.0[0] % 2 == 0 { PonyGender::Male }
        else { PonyGender::Female }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::sp_runtime::app_crypto::sp_core::blake2_128;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_randomness_collective_flip::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    /// Store all the ponies. Key is (user,pony_id)
    #[pallet::storage]
    #[pallet::getter(fn ponies)]
    pub type Ponies<T:Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, u32,
        Pony, OptionQuery
    >;

    /// Stores the next pony ID
    #[pallet::storage]
    #[pallet::getter(fn next_pony_id)]
    pub type NextPonyId<T: Config> = StorageValue<_,u32,ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId")]
    pub enum Event<T: Config> {
        /// A pony is created \[owner, pony_id, pony\]
        PonyCreated(T::AccountId, u32, Pony)
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T:Config> Pallet<T> {

        /// Create a new Pony
        #[pallet::weight(1000)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            NextPonyId::<T>::try_mutate(|next_id| -> DispatchResult {
                let current_id = *next_id;

                // Generate random 128bit value
                let payload = (
                    <pallet_randomness_collective_flip::Pallet<T> as Randomness<T::Hash, T::BlockNumber>>::random_seed().0,
                    &sender,
                    <frame_system::Pallet<T>>::extrinsic_index(),
                );
                let dna = payload.using_encoded(blake2_128);

                // Create and store pony
                let pony = Pony(dna);
                // let pony_id = Self::next_pony_id();
                Ponies::<T>::insert(&sender, current_id, pony.clone());
                // NextPonyId::<T>::put(pony_id.clone().checked_add(1).ok_or(ArithmeticError::Overflow).unwrap());
                // Emit event
                Self::deposit_event(Event::PonyCreated(sender,current_id, pony));
                Ok(())
            })
        }


    }
}