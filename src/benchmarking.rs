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

use frame_benchmarking::v2::*;
use frame_support::dispatch::RawOrigin;
use frame_support::traits::fungible::Inspect;
use frame_support::traits::fungible::Mutate;
use genres_registry::ElectronicSubtype;
use genres_registry::MusicGenre::Electronic;
use sp_runtime::bounded_vec;
use sp_runtime::Saturating;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn dumb_name_with_capacity<T: Config>(capacity: u32) -> BoundedVec<u8, T::MaxNameLen> {
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
        while i <= T::MaxGenres::get() {
            b_vec.pop();
            i += 1;
        }
    }

    b_vec
}

fn dumb_assets_with_capacity<T: Config>(capacity: u32) -> BoundedVec<Vec<u8>, T::MaxAssets> {
    let mut b_vec: BoundedVec<Vec<u8>, T::MaxAssets> = bounded_vec!();

    for _ in 0..capacity {
        b_vec
            .try_push(String::from("assets").as_bytes().to_vec())
            .unwrap();
    }

    b_vec
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register(
        n: Linear<1, { T::MaxNameLen::get() }>,
        g: Linear<0, { T::MaxGenres::get() }>,
        a: Linear<0, { T::MaxAssets::get() }>,
    ) -> Result<(), BenchmarkError> {
        let caller: T::AccountId = whitelisted_caller();

        T::Currency::set_balance(
            &caller.clone(),
            T::Currency::minimum_balance().saturating_add(100u32.into()),
        );

        let name: BoundedVec<u8, T::MaxNameLen> = dumb_name_with_capacity::<T>(n);
        let alias: BoundedVec<u8, T::MaxNameLen> = dumb_name_with_capacity::<T>(n);
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

    impl_benchmark_test_suite! {
        Artists,
        crate::mock::new_test_ext(),
        crate::mock::Test,
    }
}
