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

use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::traits::Currency;
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use genres_registry::MusicGenre;
use scale_info::TypeInfo;
use sp_runtime::traits::Hash;
use sp_runtime::RuntimeDebug;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::Vec;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub(super) type ArtistAliasOf<T> = BoundedVec<u8, <T as Config>::MaxNameLen>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdatableData<ArtistAlias> {
    Alias(Option<ArtistAlias>),
    Genres(UpdatableDataVec<MusicGenre>),
    Description(Option<Vec<u8>>),
    Assets(UpdatableDataVec<Vec<u8>>),
}

#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum UpdatableDataVec<T> {
    Add(T),
    /// lookup into the existing value if the content exist and try to remove it
    Remove(T),
    Clear,
}

/// How an Artist is designed to be stored on-chain.
#[derive(Encode, MaxEncodedLen, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Artist<T>
where
    T: frame_system::Config + Config,
{
    // Main data
    /// The artist's identifier. While the predominant mapping employs AccountId => Artist,
    /// it's essential to include this in the artist's data since verified artists can be retrieved by their name as well.
    pub(crate) owner: AccountIdOf<T>,
    /// When the artist got registered on-chain.
    pub(crate) registered_at: BlockNumberFor<T>,
    /// When the artist got verified.
    verified_at: Option<BlockNumberFor<T>>,
    // Metadata
    /// The name of the artist.
    /// This is generally the main name of how we usually call the artist (e.g: 'The Weeknd')
    /// This is fixed and can't be changed after the registration.
    pub(crate) main_name: BoundedVec<u8, T::MaxNameLen>,
    /// An alias to the main name.
    /// This name can be changed compared to the 'nickname'
    pub(crate) alias: Option<ArtistAliasOf<T>>,
    /// The main music genres of the artists.
    genres: BoundedVec<MusicGenre, T::MaxGenres>,
    // Metadata Fingerprint
    // Given the significant size of certain data associated with an artist,
    // we choose to store a digital fingerprint (hash) of this data rather than
    // the raw data itself. This fingerprint acts as a unique digital reference,
    // and services can use it to compare and validate the artist's data, ensuring
    // that it has been approved and recorded on the blockchain by the artist themselves.
    /// The digital fingerprint (hash) of the artist's description.
    pub(crate) description: Option<T::Hash>,
    /// Digital assets (such as photos, profile pictures, banners, videos, etc.)
    /// that officially represent the artist. These fingerprints allow for the
    /// verification of the authenticity of these assets.
    pub(crate) assets: BoundedVec<T::Hash, T::MaxAssets>,
    // Linked chain logic data
    /// Associated smart-contracts deployed by dApps for the artist (e.g: royalties contracts)
    contracts: BoundedVec<AccountIdOf<T>, T::MaxContracts>,
}

impl<T> Artist<T>
where
    T: frame_system::Config + Config,
{
    pub(super) fn new(
        owner: AccountIdOf<T>,
        main_name: BoundedVec<u8, T::MaxNameLen>,
        alias: Option<ArtistAliasOf<T>>,
        description: Option<T::Hash>,
        assets: BoundedVec<T::Hash, T::MaxAssets>,
        contracts: BoundedVec<AccountIdOf<T>, T::MaxContracts>,
    ) -> Self {
        let current_block = <frame_system::Pallet<T>>::block_number();
        Artist {
            owner,
            registered_at: current_block,
            verified_at: None,
            main_name,
            alias,
            // need to set later with the checked fn
            genres: Default::default(),
            description,
            assets,
            contracts,
        }
    }

    /// Set the genres of the artist while verifying that there is not the same genre multiple times.
    pub(super) fn set_checked_genres(
        &mut self,
        genres: BoundedVec<MusicGenre, T::MaxGenres>,
    ) -> DispatchResultWithPostInfo {
        let mut seen = BTreeSet::new();

        for genre in genres.clone() {
            if !seen.insert(genre.clone()) {
                return Err(Error::<T>::NotUniqueGenre.into());
            }
        }

        self.genres = genres;

        Ok(().into())
    }

    fn add_checked_genres(&mut self, genre: MusicGenre) -> DispatchResultWithPostInfo {
        let mut actual_genres = self.genres.clone();
        actual_genres
            .try_push(genre)
            .map_err(|_| Error::<T>::Full)?;

        self.set_checked_genres(actual_genres)
    }

    pub(super) fn update(
        &mut self,
        field: UpdatableData<BoundedVec<u8, T::MaxNameLen>>,
    ) -> DispatchResultWithPostInfo {
        match field {
            UpdatableData::Alias(x) => self.set_alias(x),
            UpdatableData::Genres(UpdatableDataVec::Add(x)) => return self.add_checked_genres(x),
            UpdatableData::Genres(UpdatableDataVec::Remove(x)) => return self.remove_genre(x),
            UpdatableData::Genres(UpdatableDataVec::Clear) => self.genres = Default::default(),
            UpdatableData::Description(x) => self.set_description(x),
            UpdatableData::Assets(UpdatableDataVec::Add(x)) => return self.add_asset(&x),
            UpdatableData::Assets(UpdatableDataVec::Remove(x)) => return self.remove_asset(&x),
            UpdatableData::Assets(UpdatableDataVec::Clear) => self.assets = Default::default(),
        }

        Ok(().into())
    }
    /// Return true if the artist have a 'verified_at" timestamp which mean he's verified
    pub(super) fn is_verified(&self) -> bool {
        self.verified_at.is_some()
    }

    fn set_alias(&mut self, alias: Option<BoundedVec<u8, T::MaxNameLen>>) {
        self.alias = alias
    }

    fn set_description(&mut self, raw_description: Option<Vec<u8>>) {
        match raw_description {
            Some(x) => self.description = Some(T::Hashing::hash(&x)),
            None => self.description = None,
        }
    }

    fn add_asset(&mut self, asset: &Vec<u8>) -> DispatchResultWithPostInfo {
        let hash = T::Hashing::hash(asset);
        self.assets.try_push(hash).map_err(|_| Error::<T>::Full)?;
        Ok(().into())
    }

    fn remove_asset(&mut self, asset: &Vec<u8>) -> DispatchResultWithPostInfo {
        let hash = T::Hashing::hash(asset);

        if let Some(pos) = self.assets.iter().position(|&x| x == hash) {
            self.assets.remove(pos);
            Ok(().into())
        } else {
            Err(Error::<T>::NotFound.into())
        }
    }

    fn remove_genre(&mut self, genre: MusicGenre) -> DispatchResultWithPostInfo {
        if let Some(pos) = self.genres.iter().position(|&x| x == genre) {
            self.genres.remove(pos);
            Ok(().into())
        } else {
            Err(Error::<T>::NotFound.into())
        }
    }
}
