use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_eq!(TemplateModule::get_and_increment_nonce().0, 0);
		assert_eq!(TemplateModule::get_and_increment_nonce().0, 1);
		assert_eq!(TemplateModule::get_and_increment_nonce().0, 2);
		assert_eq!(TemplateModule::get_and_increment_nonce().0, 3);
	});
}

#[test]
fn wontwork() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_eq!(TemplateModule::get_and_increment_nonce().0, 1);
	});
}

#[test]
fn update_index_and_check_pair_list() {
	new_test_ext().execute_with(|| {
        let model = "article".as_bytes().to_vec();
        let reqid = "00003211".as_bytes().to_vec();
        let id = "001".as_bytes().to_vec();
        let hash = "xvdfewirwjfsdlkfnsdfjewo".as_bytes().to_vec();
		// Dispatch a signed extrinsic.
        TemplateModule::update_index(Origin::signed(1), model.clone(), reqid, id.clone(), hash.clone());

		assert_eq!(TemplateModule::check_pair_list(model, vec![(id, hash)]), true);
	});
}

#[test]
fn wontwork_update_index_and_check_pair_list() {
	new_test_ext().execute_with(|| {
        let model = "article".as_bytes().to_vec();
        let reqid = "00003211".as_bytes().to_vec();
        let id = "001".as_bytes().to_vec();
        let hash = "xvdfewirwjfsdlkfnsdfjewo".as_bytes().to_vec();
		// Dispatch a signed extrinsic.
        _ = TemplateModule::update_index(Origin::signed(1), model.clone(), reqid, id.clone(), hash.clone());

		assert_eq!(TemplateModule::check_pair_list(model, vec![(id, "ztfjajdjfj".as_bytes().to_vec())]), true);
	});
}


/*
#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}
*/
