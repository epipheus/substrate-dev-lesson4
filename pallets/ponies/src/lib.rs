#![cfg_attr(not(feature = "std"),no_std)]

// so far at 19:06

use frame_support::{
    pallet_prelude::*,
    traits::Randomness,
};
use frame_system::pallet_prelude::*;
use sp_runtime::ArithmeticError;
use sp_io::hashing::blake2_128;
use sp_std::result::Result;

pub use pallet::*;

#[cfg(test)]
mod tests;

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
        PonyCreated(T::AccountId, u32, Pony),
        PonyBred(T::AccountId,u32,Pony),
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidPonyId,
        SameGender,
        NoChemistry,
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
                let current_id = Self::get_next_pony_id()?;
                let dna = Self::random_value();

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

        #[pallet::weight(1000)]
        pub fn breed(origin: OriginFor<T>, pony_id_1: u32, pony_id_2: u32) -> DispatchResult {
            let sender  = ensure_signed(origin)?;
            let pony1   = Self::ponies(&sender,pony_id_1).ok_or(Error::<T>::InvalidPonyId)?;
            let pony2   = Self::ponies(&sender,pony_id_2).ok_or(Error::<T>::InvalidPonyId)?;

            ensure!(pony1.gender() != pony2.gender(), Error::<T>::SameGender);
            let pony_id = Self::get_next_pony_id();

            let pony1_dna = pony1.0;
            let pony2_dna = pony2.0;

            let selector = Self::random_value(&sender);
            let mut new_dna = [0u8;16];

            // Combine parents and selector to make a new pony
            for i in 0..pony1_dna.len() {
                new_dna[i] = combine_dna(pony2_dna[i], pony2_dna[i], selector[i]);
            }

            let new_pony = Pony(new_dna);
            Ponies::<T>::insert(&sender, pony_id, &new_pony);
            Self::deposit_event(Event::PonyBred(sender, pony_id, new_pony));
            OK(())
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (dna1 & !selector) | (dna2 & selector)
}

impl<T: Config> Pallet<T> {
    fn get_next_pony_id() -> Result<u32,DispatchError> {
        NextPonyId::<T>::try_mutate(|next_id| -> DispatchResult {
            let current_id = *next_id;
            *next_id = next_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            OK(current_id)
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        // Generate a random 128-bit value
        let payload = (
            <pallet_randomness_collective_flip::Pallet<T> as Randomness<T::Hash, T::BlockNumber>>::random_seed().0,
            &sender,
            <frame_system::Pallet<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }
}