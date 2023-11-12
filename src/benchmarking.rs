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

//! Artists pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Artists;

use crate::types::ArtistAliasOf;
use frame_benchmarking::v2::*;
use frame_support::dispatch::RawOrigin;
use frame_support::traits::fungible::Inspect;
use frame_support::traits::fungible::Mutate;
use frame_system::Pallet as System;
use genres_registry::ElectronicSubtype;
use genres_registry::MusicGenre::Electronic;
use sp_runtime::bounded_vec;
use sp_runtime::Saturating;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn dumb_name_with_capacity<T: Config>(capacity: u32) -> ArtistAliasOf<T> {
    let vec = (0..capacity)
        .map(|_| "X")
        .collect::<String>()
        .as_bytes()
        .to_vec();
    vec.try_into().unwrap()
}

fn dumb_genres_with_capacity<T: Config>(capacity: u32) -> BoundedVec<MusicGenre, T::MaxGenres> {
    let mut b_vec: BoundedVec<MusicGenre, T::MaxGenres> = bounded_vec!(
        Electronic(Some(ElectronicSubtype::House)),
        Electronic(Some(ElectronicSubtype::Ambient)),
        Electronic(Some(ElectronicSubtype::Techno)),
        Electronic(Some(ElectronicSubtype::Trance)),
        Electronic(Some(ElectronicSubtype::DrumNBass))
    );

    if capacity < T::MaxGenres::get() {
        let mut i = capacity;
        while i < T::MaxGenres::get() {
            b_vec.pop();
            i += 1;
        }
    }

    b_vec
}

fn dumb_assets_with_capacity<T: Config>(capacity: u32) -> BoundedVec<Vec<u8>, T::MaxAssets> {
    let mut b_vec: BoundedVec<Vec<u8>, T::MaxAssets> = bounded_vec!();

    for i in 0..capacity {
        let asset = format!("asset{}", i);
        b_vec.try_push(asset.as_bytes().to_vec()).unwrap();
    }

    b_vec
}

fn register_test_artist<T: Config>(
    id: T::AccountId,
    name_length: u32,
    genres_count: u32,
    assets_count: u32,
) {
    let name: ArtistAliasOf<T> = dumb_name_with_capacity::<T>(name_length);
    let alias: ArtistAliasOf<T> = dumb_name_with_capacity::<T>(name_length);
    let genres: BoundedVec<MusicGenre, T::MaxGenres> = dumb_genres_with_capacity::<T>(genres_count);
    let description = Some(String::from("test").as_bytes().to_vec());
    let assets: BoundedVec<Vec<u8>, T::MaxAssets> = dumb_assets_with_capacity::<T>(assets_count);

    Artists::<T>::register(
        RawOrigin::Signed(id).into(),
        name,
        Some(alias),
        genres,
        description,
        assets,
    )
    .expect("benchmark test should not fail");
}

#[benchmarks]
mod benchmarks {
    use super::*;
    use crate::types::{UpdatableData, UpdatableDataVec};
    use genres_registry::ClassicalSubtype;

    #[benchmark]
    fn register(
        n: Linear<1, { T::MaxNameLen::get() }>,
        g: Linear<0, { T::MaxGenres::get() }>,
        a: Linear<0, { T::MaxAssets::get() }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        let name: ArtistAliasOf<T> = dumb_name_with_capacity::<T>(n);
        let alias: ArtistAliasOf<T> = dumb_name_with_capacity::<T>(n);
        let genres: BoundedVec<MusicGenre, T::MaxGenres> = dumb_genres_with_capacity::<T>(g);
        let description = Some(String::from("test").as_bytes().to_vec());
        let assets: BoundedVec<Vec<u8>, T::MaxAssets> = dumb_assets_with_capacity::<T>(a);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller.clone().into()),
            name.clone(),
            Some(alias),
            genres,
            description,
            assets,
        );

        assert_last_event::<T>(Event::ArtistRegistered { id: caller, name }.into());

        Ok(())
    }

    #[benchmark]
    fn unregister(
        n: Linear<1, { T::MaxNameLen::get() }>,
        g: Linear<0, { T::MaxGenres::get() }>,
        a: Linear<0, { T::MaxAssets::get() }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), n, g, a);

        System::<T>::set_block_number(
            System::<T>::block_number().saturating_add(T::UnregisterPeriod::get().into()),
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone().into()));

        assert_last_event::<T>(Event::ArtistUnregistered { id: caller }.into());

        Ok(())
    }

    /// `n` is the existing artist data and `x` is the new data to update with.
    #[benchmark]
    fn update_alias(
        n: Linear<1, { T::MaxNameLen::get() }>,
        x: Linear<1, { T::MaxNameLen::get() }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), n, 0, 0);

        let new_data =
            UpdatableData::<ArtistAliasOf<T>>::Alias(Some(dumb_name_with_capacity::<T>(x)));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_add_genres(
        n: Linear<0, { T::MaxGenres::get().saturating_sub(1) }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, n, 0);

        let new_data = UpdatableData::<ArtistAliasOf<T>>::Genres(UpdatableDataVec::Add(
            MusicGenre::Classical(Some(ClassicalSubtype::Symphony)),
        ));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_remove_genres(n: Linear<1, { T::MaxGenres::get() }>) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, n, 0);

        // Always remove what we are sure this is the first element so there is always something
        // to remove even with only one genre existing in the benchmarking artist.
        let new_data = UpdatableData::<ArtistAliasOf<T>>::Genres(UpdatableDataVec::Remove(
            Electronic(Some(ElectronicSubtype::House)),
        ));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_clear_genres(n: Linear<0, { T::MaxGenres::get() }>) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, n, 0);

        let new_data = UpdatableData::<ArtistAliasOf<T>>::Genres(UpdatableDataVec::Clear);

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// Description is a hashed data so the length is fixed, we don't need to benchmark multiple lengths.
    #[benchmark]
    fn update_description() -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, 0, 0);

        let new_data =
            UpdatableData::<ArtistAliasOf<T>>::Description(Some(b"new_description".to_vec()));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_add_assets(
        n: Linear<0, { T::MaxAssets::get().saturating_sub(1) }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, 0, n);

        let new_data = UpdatableData::<ArtistAliasOf<T>>::Assets(UpdatableDataVec::Add(
            b"test asset".to_vec(),
        ));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_remove_assets(n: Linear<1, { T::MaxAssets::get() }>) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, 0, n);

        // Always remove what we are sure this is the first element so there is always something
        // to remove even with only one genre existing in the benchmarking artist.
        let new_data =
            UpdatableData::<ArtistAliasOf<T>>::Assets(UpdatableDataVec::Remove(b"asset0".to_vec()));

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    /// `n` is the existing artist data.
    #[benchmark]
    fn update_clear_assets(n: Linear<0, { T::MaxAssets::get() }>) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller,
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        register_test_artist::<T>(caller.clone(), 1, 0, n);

        let new_data = UpdatableData::<ArtistAliasOf<T>>::Assets(UpdatableDataVec::Clear);

        #[extrinsic_call]
        update(RawOrigin::Signed(caller.clone().into()), new_data.clone());

        assert_last_event::<T>(
            Event::ArtistUpdated {
                id: caller,
                new_data,
            }
            .into(),
        );

        Ok(())
    }

    impl_benchmark_test_suite! {
        Artists,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    }
}
