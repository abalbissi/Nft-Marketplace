use crate::{mock::*, Class, Error, Nft};
use frame_support::{assert_noop, assert_ok, print};
use frame_system::Origin;
#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {

	});
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
		// println!("here is it  {:?}", NftModule::classes(0));
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
			NftModule::nfts(0,0),
			Some(Nft {
				owner: 1,
				class_id:0,
				description: vec![102, 105, 114, 115, 116],
				name: vec![102, 105, 114, 115, 116],
				url: vec![102, 105, 114, 115, 116],
				external_url: vec![102, 105, 114, 115, 116]
			})
		);

		assert_eq!(
			NftModule::next_nft_id(),
			Some(1)
		);
		assert_eq!(
			NftModule::next_class_id(),
			Some(1)
		);

		assert_ok!(NftModule::transfer(
			RuntimeOrigin::signed(1),
			2,
			0,0
		));

		assert_ok!(NftModule::burn(
			RuntimeOrigin::signed(2),
			0,0
		));
		assert_eq!(
			NftModule::nfts(0,0),
			None
		);
	});
}
