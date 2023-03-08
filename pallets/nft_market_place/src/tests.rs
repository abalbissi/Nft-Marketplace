use crate::{mock::*, Auction, Offer};
use frame_support::{assert_noop, assert_ok, print};
use frame_system::Origin;
use pallet_nft::{Class, Error, Nft};
use sp_runtime::SaturatedConversion;
#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {});
}
#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftModule::create_class(
			RuntimeOrigin::signed(1),
			"first".into(),
			"first".into(),
			"first".into(),
			"first".into()
		));
		assert_eq!(
			NftModule::classes(0),
			Some(Class {
				owner: 1,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);
		assert_ok!(NftModule::mint(
			RuntimeOrigin::signed(1),
			0,
			"first".into(),
			"first".into(),
			"first".into(),
			"first".into()
		));
		assert_eq!(
			NftModule::nfts(0, 0),
			Some(Nft {
				owner: 1,
				class_id: 0,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);

		assert_eq!(NftModule::next_nft_id(), Some(1));
		assert_eq!(NftModule::next_class_id(), Some(1));

		//###############Pallet Market Place###############//
		assert_ok!(NftMarketPlaceModule::create_offer(RuntimeOrigin::signed(1), 0, 0, 100));
		assert_eq!(
			NftMarketPlaceModule::offers(0, 0),
			Some(Offer { owner: 1, class_id: 0, nft_id: 0, amount: 100 })
		);

		assert_ok!(NftMarketPlaceModule::buy_offer(RuntimeOrigin::signed(2), 0, 0, 100));

		assert_eq!(NftMarketPlaceModule::offers(0, 0), None);

		assert_eq!(
			NftModule::nfts(0, 0),
			Some(Nft {
				owner: 2,
				class_id: 0,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);

		assert_eq!(
			NftModule::classes(0),
			Some(Class {
				owner: 1,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);

		//  auction
		let now = NftMarketPlaceModule::now();

		assert_ok!(NftMarketPlaceModule::create_auction(
			RuntimeOrigin::signed(2),
			0,
			0,
			100,
			now,
			now + 100
		));

		assert_eq!(
			NftMarketPlaceModule::auctions(0, 0),
			Some(Auction {
				owner: 2,
				nft_id: 0,
				spender: 2,
				class_id: 0,
				min_amount: 0,
				amount: 100,
				start_time: now,
				end_time: now + 100
			})
		);

		assert_ok!(NftMarketPlaceModule::buy_auction(RuntimeOrigin::signed(1), 0, 0, 101));
		let c = NftMarketPlaceModule::auctions(0,0).unwrap();
		assert_eq!(
			c.spender,
			1
		);
		// assert_ok!(NftMarketPlaceModule::cancel_auction(RuntimeOrigin::signed(2), 0, 0));

		// assert_eq!(NftMarketPlaceModule::auctions(0, 0), None);
		assert_ok!(NftModule::mint(
			RuntimeOrigin::signed(1),
			0,
			"first".into(),
			"first".into(),
			"first".into(),
			"first".into()
		));
		assert_eq!(
			NftModule::nfts(0, 1),
			Some(Nft {
				owner: 1,
				class_id: 0,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);

		assert_eq!(NftModule::next_nft_id(), Some(2));


		assert_ok!(NftMarketPlaceModule::create_offer(RuntimeOrigin::signed(1), 0, 1, 100));



		assert_ok!(NftMarketPlaceModule::cancel_offer(
			RuntimeOrigin::signed(1),
			1,0
		));
		assert_eq!(
			NftMarketPlaceModule::offers(0, 1),
			None
		);
	});
}
