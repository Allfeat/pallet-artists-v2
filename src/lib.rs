// This file is part of Allfeat.

// Copyright (C) Allfeat (FR) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Artists Pallet v2
//!
//! If you're diving into the "Artists Pallet v2," here's a quick guide to help you
//! navigate and understand its core components and functionalities.
//!
//! ### Overview
//!
//! The "Artists Pallet v2" is a pallet implementation designed for the management of artists on Allfeat blockchain.
//! This module enables users to register as artists, associate details to their profiles, and handle this
//! information on-chain.
//!
//! ### Key Features
//!
//! 1. **Artist Registration**: Users can register themselves as artists, providing details like their main
//! name, an alias, music genres, a description, and related assets.
//!
//! 2. **Storage**: Artist data is securely stored on-chain. Artists can be retrieved either by their account
//! ID or by their name.
//!
//! 3. **Asset Handling**: Artist assets undergo hashing to ensure data integrity.
//!
//! 4. **Error Management**: Several error cases are covered, like when an artist tries to register with an
//! already taken name or attempts to unregister while verified.
//!
//! ### Configuration (`Config`)
//!
//! This pallet offers multiple configurable constants:
//! - `BaseDeposit`: The base deposit for registering as an artist.
//! - `ByteDeposit`: The per-byte deposit for hashing data on-chain.
//! - `UnregisterPeriod`: The time a registered artist must wait before being allowed to unregister.
//! - `MaxNameLen`: Maximum allowable length for an artist's name.
//! - `MaxGenres`: Maximum number of genres an artist can associate with.
//! - `MaxAssets`: Maximum assets an artist can have.
//! - `MaxContracts`: Maximum contracts an artist can have.
//!
//! ### Events
//!
//! - `ArtistRegistered`: Triggered when a new artist gets registered. Carries the artist's account ID and name.
//!
//! ### Errors
//!
//! A few of the potential errors include:
//! - `NotUniqueGenre`: Raised when a genre appears multiple times in an artist's data.
//! - `NameUnavailable`: Raised if the artist's name is already taken by a verified artist.
//! - `NotRegistered`: If an account isn't registered as an artist.
//! - `AlreadyRegistered`: If the account ID is already registered as an artist.
//! - `IsVerified`: If the artist is verified and therefore cannot unregister.
//! - `PeriodNotPassed`: If the unregister period isn't fully elapsed yet.
//!
//! ### Extrinsics
//!
//! - `register`: Allows a user to register as an artist by mapping the Account ID.
//!
//! ### Wrapping Up
//!
//! As you navigate through "Artists Pallet v2," you'll find it's a robust module for on-chain artist profile
//! management. If you have questions, the comments in the code should guide you, but this overview should give
//! you a head start

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;
pub mod weights;

use weights::WeightInfo;

use frame_support::dispatch::DispatchErrorWithPostInfo;
use frame_support::pallet_prelude::{DispatchResultWithPostInfo, Get, Weight};
use frame_support::BoundedVec;
use genres_registry::MusicGenre;
pub use types::Artist;

use crate::types::{ArtistAliasOf, UpdatableData};
use crate::types::{BalanceOf, UpdatableDataVec};
use crate::Event::ArtistRegistered;
use crate::Event::{ArtistUnregistered, ArtistUpdated};
use frame_support::traits::ReservableCurrency;
use sp_runtime::traits::Hash;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

pub use pallet::*;

/// Artists Pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + Sized {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The way to handle the storage deposit cost of Artist creation
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The base deposit for registering as an artist on chain.
        type BaseDeposit: Get<BalanceOf<Self>>;

        /// The per-byte deposit for placing data hashes on chain.
        type ByteDeposit: Get<BalanceOf<Self>>;

        /// How many time a registered artist have to wait to unregister himself.
        #[pallet::constant]
        type UnregisterPeriod: Get<u32>;

        /// The maximum length of the artist name.
        #[pallet::constant]
        type MaxNameLen: Get<u32>;

        /// The maximum amount of genres that an artist can have.
        #[pallet::constant]
        type MaxGenres: Get<u32>;

        /// The maximum amount of assets that an artist can have.
        #[pallet::constant]
        type MaxAssets: Get<u32>;

        /// The maximum amount of contracts that an artist can have.
        #[pallet::constant]
        type MaxContracts: Get<u32>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn get_artist_by_id)]
    pub(super) type ArtistOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Artist<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_artist_by_name)]
    pub(super) type ArtistNameOf<T: Config> =
        StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxNameLen>, Artist<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new artist got registered.
        ArtistRegistered {
            /// The address of the new artist.
            id: T::AccountId,
            /// main name of the new artist.
            name: BoundedVec<u8, T::MaxNameLen>,
        },

        /// An Artist as been unregistered
        ArtistUnregistered { id: T::AccountId },

        ArtistUpdated {
            /// The address of the updated artist.
            id: T::AccountId,
            /// The new data.
            new_data: UpdatableData<ArtistAliasOf<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A genre appear multiple time in the artist data.
        NotUniqueGenre,
        /// An asset appear multiple time in the artist data.
        NotUniqueAsset,
        /// The artist name is already attributed to a verified artist.
        NameUnavailable,
        /// Account isn't registered as an Artist.
        NotRegistered,
        /// This account ID is already registered as an artist.
        AlreadyRegistered,
        /// Artist is verified and can't unregister.
        IsVerified,
        /// Unregister period isn't fully passed.
        PeriodNotPassed,
        /// The maximum value possible for this field for an artist has been violated.
        Full,
        /// Element wasn't found.
        NotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register the caller as an Artist.
        #[pallet::weight(T::WeightInfo::register(
            T::MaxNameLen::get(),
            T::MaxGenres::get(),
            T::MaxAssets::get()
        ))]
        #[pallet::call_index(0)]
        pub fn register(
            origin: OriginFor<T>,
            main_name: BoundedVec<u8, T::MaxNameLen>,
            alias: Option<BoundedVec<u8, T::MaxNameLen>>,
            genres: BoundedVec<MusicGenre, T::MaxGenres>,
            description: Option<Vec<u8>>,
            assets: BoundedVec<Vec<u8>, T::MaxAssets>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            ensure!(
                !ArtistOf::<T>::contains_key(origin.clone()),
                Error::<T>::AlreadyRegistered
            );
            ensure!(
                !ArtistNameOf::<T>::contains_key(main_name.clone()),
                Error::<T>::NameUnavailable
            );

            T::Currency::reserve(&origin, T::BaseDeposit::get())?;

            let mut new_artist = Artist::<T>::new(
                origin.clone(),
                main_name.clone(),
                alias,
                match description {
                    Some(desc) => Some(T::Hashing::hash(&desc)),
                    None => None,
                },
                Self::checked_hash_assets(assets)?,
                Default::default(),
            );
            new_artist.set_checked_genres(genres)?;

            ArtistOf::insert(origin.clone(), new_artist);
            Self::deposit_event(ArtistRegistered {
                id: origin,
                name: main_name,
            });
            Ok(().into())
        }

        /// Unregister the caller from being an artist,
        /// clearing associated artist data mapped to this account
        #[pallet::weight(T::WeightInfo::unregister(
            T::MaxNameLen::get(),
            T::MaxGenres::get(),
            T::MaxAssets::get()
        ))]
        #[pallet::call_index(1)]
        pub fn unregister(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            Self::can_unregister(&origin)?;

            // return locked deposit
            T::Currency::unreserve(&origin, T::BaseDeposit::get());
            ArtistOf::<T>::remove(origin.clone());

            Self::deposit_event(ArtistUnregistered { id: origin });
            Ok(().into())
        }

        /// Update the passed caller artist data field with the passed data.
        #[pallet::weight({
            let weight_fn = Pallet::<T>::get_weight_update_fn(&data);
            weight_fn()
        })]
        #[pallet::call_index(2)]
        pub fn update(
            origin: OriginFor<T>,
            data: UpdatableData<ArtistAliasOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let origin = ensure_signed(origin)?;

            ArtistOf::<T>::try_mutate(origin.clone(), |maybe_artist| {
                if let Some(artist) = maybe_artist {
                    artist.update(data.clone())?;
                    Self::deposit_event(ArtistUpdated {
                        id: origin,
                        new_data: data,
                    });
                    Ok(().into())
                } else {
                    return Err(Error::<T>::NotRegistered.into());
                }
            })
        }
    }
}

impl<T> Pallet<T>
where
    T: frame_system::Config + Config,
{
    /// Returns a closure that computes the weight of an update operation based on the provided data.
    ///
    /// This function is part of Substrate's weight and benchmarking system for blockchain operations.
    /// It determines the computational and storage resources required for different update operations.
    ///
    /// # Arguments
    ///
    /// * `data` - A reference to `UpdatableData<ArtistAliasOf<T>>`, an enum representing the type of
    ///   data to be updated. The generic `T` is typically a type associated with a specific blockchain
    ///   implementation.
    ///
    /// # Returns
    ///
    /// A `Box<dyn FnOnce() -> Weight>` which is a boxed closure that can be called once to compute the
    /// weight of the specified update operation. `Weight` is a metric used to measure the resource
    /// consumption of the operation on the blockchain.
    ///
    /// # Implementation Details
    ///
    /// - The function uses a `match` expression to determine the type of the update operation from
    ///   `UpdatableData`.
    /// - For `Genres` and `Assets`, a sub-match on `UpdatableDataVec` discriminates whether items are
    ///   being added, removed, or if the list is cleared.
    /// - Each branch calls an appropriate method from the `WeightInfo` trait, which must be implemented
    ///   by `T`. These methods provide weight estimations for different operations, such as
    ///   `T::WeightInfo::update_add_genres(T::MaxGenres::get())` for adding genres.
    /// - Closures are used to encapsulate the specific logic for each update operation, ensuring the
    ///   returned function conforms to `FnOnce() -> Weight`.
    ///
    /// This approach allows dynamic determination of operation costs on the blockchain, adapting to
    /// the current context and specific parameters of each update operation.
    fn get_weight_update_fn(data: &UpdatableData<ArtistAliasOf<T>>) -> Box<dyn FnOnce() -> Weight> {
        match data {
            UpdatableData::Genres(x) => match x {
                UpdatableDataVec::Add(_) => {
                    Box::new(move || T::WeightInfo::update_add_genres(T::MaxGenres::get()))
                }
                UpdatableDataVec::Remove(_) => {
                    Box::new(move || T::WeightInfo::update_remove_genres(T::MaxGenres::get()))
                }
                UpdatableDataVec::Clear => {
                    Box::new(move || T::WeightInfo::update_clear_genres(T::MaxGenres::get()))
                }
            },
            UpdatableData::Assets(x) => match x {
                UpdatableDataVec::Add(_) => {
                    Box::new(move || T::WeightInfo::update_add_assets(T::MaxAssets::get()))
                }
                UpdatableDataVec::Remove(_) => {
                    Box::new(move || T::WeightInfo::update_remove_assets(T::MaxAssets::get()))
                }
                UpdatableDataVec::Clear => {
                    Box::new(move || T::WeightInfo::update_clear_assets(T::MaxAssets::get()))
                }
            },
            UpdatableData::Description(_) => Box::new(move || T::WeightInfo::update_description()),
            UpdatableData::Alias(_) => Box::new(move || {
                T::WeightInfo::update_alias(T::MaxNameLen::get(), T::MaxNameLen::get())
            }),
        }
    }

    /// Hash a collection of raw assets while checking for non-unique assets.
    fn checked_hash_assets(
        raw_assets: BoundedVec<Vec<u8>, T::MaxAssets>,
    ) -> Result<BoundedVec<T::Hash, T::MaxAssets>, DispatchErrorWithPostInfo> {
        let mut hashed: BoundedVec<T::Hash, T::MaxAssets> = Default::default();

        raw_assets
            .iter()
            .try_for_each(|asset| -> Result<(), DispatchErrorWithPostInfo> {
                let hash = T::Hashing::hash(asset);
                if hashed.contains(&hash) {
                    return Err(Error::<T>::NotUniqueAsset.into());
                }
                hashed.try_push(hash).expect("already bounded");
                Ok(())
            })?;

        Ok(hashed)
    }

    /// Return if the actual account ID can unregister from being an Artist.
    fn can_unregister(who: &T::AccountId) -> DispatchResultWithPostInfo {
        let artist_data = Pallet::<T>::get_artist_by_id(&who);

        match artist_data {
            Some(data) => {
                // verified artists can't unregister
                if data.is_verified() {
                    return Err(Error::<T>::IsVerified.into());
                }

                let current_block = <frame_system::Pallet<T>>::block_number();
                let expected_passed_time: u32 = T::UnregisterPeriod::get();

                // Verify that we passed the Unregister Period
                if current_block - data.registered_at < expected_passed_time.saturated_into() {
                    return Err(Error::<T>::PeriodNotPassed.into());
                }

                Ok(().into())
            }
            None => Err(Error::<T>::NotRegistered.into()),
        }
    }
}
