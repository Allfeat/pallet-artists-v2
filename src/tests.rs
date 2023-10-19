//! # Artists tests.

#![cfg(test)]

use super::*;
use crate::mock::*;
use crate::types::{ArtistAliasOf, UpdatableData};
use crate::Error as ArtistsError;
use frame_support::pallet_prelude::Get;
use frame_support::{assert_noop, assert_ok};
use genres_registry::ElectronicSubtype;
use sp_std::prelude::Vec;

struct ArtistMock<T: Config> {
    pub main_name: BoundedVec<u8, <T as Config>::MaxNameLen>,
    pub alias: Option<BoundedVec<u8, <T as Config>::MaxNameLen>>,
    pub genres: BoundedVec<MusicGenre, T::MaxGenres>,
    pub description: Option<Vec<u8>>,
    pub assets: BoundedVec<Vec<u8>, T::MaxAssets>,
}

fn to_bounded_alias(str: String) -> ArtistAliasOf<Test> {
    ArtistAliasOf::<Test>::try_from(str.as_bytes().to_vec()).expect("invalid alias test string")
}

fn tester_artist<T: Config>() -> ArtistMock<T> {
    let mut genres = Vec::new();
    genres.push(MusicGenre::Electronic(Some(ElectronicSubtype::House)));

    ArtistMock {
        main_name: b"Tester".to_vec().try_into().unwrap(),
        alias: Some(b"Dark Singer".to_vec().try_into().unwrap()),
        genres: genres.try_into().unwrap(),
        description: Some(b"A simple tester artist.".to_vec()),
        assets: Default::default(),
    }
}

#[test]
fn artist_register_works() {
    new_test_ext().execute_with(|| {
        let artist = tester_artist::<Test>();
        let artist_id = 1u64;

        let old_balance = Balances::free_balance(&artist_id);

        assert_ok!(Artists::register(
            RuntimeOrigin::signed(artist_id),
            artist.main_name.clone(),
            artist.alias.clone(),
            artist.genres.clone(),
            artist.description.clone(),
            artist.assets.clone(),
        ));

        // Verify register cost
        let new_balance = Balances::free_balance(&artist_id);
        let expected_cost: u64 = <Test as Config>::BaseDeposit::get();
        assert_eq!(new_balance, old_balance - expected_cost);

        // Can't register a second time if already registered
        assert_noop!(
            Artists::register(
                RuntimeOrigin::signed(artist_id),
                artist.main_name,
                artist.alias,
                artist.genres,
                artist.description,
                artist.assets,
            ),
            ArtistsError::<Test>::AlreadyRegistered
        );
    })
}

#[test]
fn artist_unregister_works() {
    new_test_ext().execute_with(|| {
        let artist = tester_artist::<Test>();
        let artist_id = 1u64;

        // Can't unregister if not registered
        assert_noop!(
            Artists::unregister(RuntimeOrigin::signed(artist_id)),
            Error::<Test>::NotRegistered
        );

        assert_ok!(Artists::register(
            RuntimeOrigin::signed(artist_id),
            artist.main_name.clone(),
            artist.alias.clone(),
            artist.genres.clone(),
            artist.description.clone(),
            artist.assets.clone(),
        ));

        // Can't unregister if not waited the unregister period
        assert_noop!(
            Artists::unregister(RuntimeOrigin::signed(artist_id)),
            Error::<Test>::PeriodNotPassed
        );

        let unregister_cd: u32 = <Test as Config>::UnregisterPeriod::get();
        frame_system::Pallet::<Test>::set_block_number(unregister_cd.saturated_into());

        let old_balance = Balances::free_balance(&artist_id);

        assert_ok!(Artists::unregister(RuntimeOrigin::signed(artist_id)));

        // Deposit has been returned
        let new_balance = Balances::free_balance(&artist_id);
        let expected_cost: u64 = <Test as Config>::BaseDeposit::get();
        assert_eq!(new_balance, old_balance + expected_cost);
    })
}

#[test]
fn artist_update_alias_works() {
    new_test_ext().execute_with(|| {
        let artist = tester_artist::<Test>();
        let artist_id = 1u64;

        assert_ok!(Artists::register(
            RuntimeOrigin::signed(artist_id),
            artist.main_name.clone(),
            artist.alias.clone(),
            artist.genres.clone(),
            artist.description.clone(),
            artist.assets.clone(),
        ));

        let new_alias = to_bounded_alias(String::from("new artist alias"));

        assert_ok!(Artists::update(
            RuntimeOrigin::signed(artist_id),
            UpdatableData::<ArtistAliasOf<Test>>::Alias(Some(new_alias)),
        ));

        // Can't update if the caller is not a registered artist
        assert_noop!(
            Artists::update(
                RuntimeOrigin::signed(2),
                UpdatableData::<ArtistAliasOf<Test>>::Alias(None),
            ),
            Error::<Test>::NotRegistered
        );

        assert_ok!(Artists::update(
            RuntimeOrigin::signed(artist_id),
            UpdatableData::<ArtistAliasOf<Test>>::Alias(None),
        ));
    })
}
