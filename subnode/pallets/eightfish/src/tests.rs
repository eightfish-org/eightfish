use crate::mock::*;

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
		assert_ne!(TemplateModule::get_and_increment_nonce().0, 1);
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
		_ = TemplateModule::update_index(
			Origin::signed(1),
			model.clone(),
			reqid,
			id.clone(),
			hash.clone(),
		);

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
		_ = TemplateModule::update_index(
			Origin::signed(1),
			model.clone(),
			reqid,
			id.clone(),
			hash.clone(),
		);

		assert_ne!(
			TemplateModule::check_pair_list(model, vec![(id, "ztfjajdjfj".as_bytes().to_vec())]),
			true
		);
	});
}

#[test]
fn test_wasm_upgrade() {
	new_test_ext().execute_with(|| {
		let wasm_file = b"13324234324243242342323423424".to_vec();
		_ = TemplateModule::wasm_upgrade(Origin::signed(1), wasm_file.clone());

		assert_eq!(wasm_file, TemplateModule::wasm_file());
		assert_eq!(true, TemplateModule::wasm_file_new_flag());
	});
}

#[test]
fn test_disable_wasm_upgrade_flag() {
	new_test_ext().execute_with(|| {
		_ = TemplateModule::disable_wasm_upgrade_flag(Origin::signed(1));

		assert_eq!(false, TemplateModule::wasm_file_new_flag());
	});
}

#[test]
fn test_act_event() {
	new_test_ext().execute_with(|| {
		let model = b"aaa".to_vec();
		let action = b"doti".to_vec();
		let payload = b"iama payload".to_vec();

		System::set_block_number(1);
		_ = TemplateModule::act(Origin::signed(1), model.clone(), action.clone(), payload.clone());
		let expected_event = pallet_template::Event::Action(
			model,
			action,
			payload,
			0,
			vec![
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0,
			],
			0,
		);

		System::assert_last_event(Event::TemplateModule(expected_event.into()));
		assert_eq!(System::events().len(), 1);
	});
}

#[test]
fn test_update_index_event() {
	new_test_ext().execute_with(|| {
		let model = b"aaa".to_vec();
		let reqid = b"doti".to_vec();
		let id = b"fsdfjskjgjfklfjkfjwejf".to_vec();
		let hash = b"vnveritoietioetoetpergeprwgrgweerjoepj".to_vec();

		System::set_block_number(1);
		_ = TemplateModule::update_index(
			Origin::signed(1),
			model.clone(),
			reqid.clone(),
			id.clone(),
			hash.clone(),
		);

		let action = b"update_index".to_vec();
		let mut payload: Vec<u8> = Vec::new();
		payload.extend_from_slice(&reqid);
		payload.push(b':');
		payload.extend_from_slice(&id);

		let expected_event = pallet_template::Event::IndexUpdated(model, action, payload, 0);

		System::assert_last_event(Event::TemplateModule(expected_event.into()));
		assert_eq!(System::events().len(), 1);
	});
}

#[test]
fn test_wasm_upgrade_event() {
	new_test_ext().execute_with(|| {
		let wasm_file = b"vnveritoietioetoetpergeprwgrgweerjoepj".to_vec();

		System::set_block_number(1);
		_ = TemplateModule::wasm_upgrade(Origin::signed(1), wasm_file.clone());

		let expected_event = pallet_template::Event::Upgrade(true, 0);

		System::assert_last_event(Event::TemplateModule(expected_event.into()));
		assert_eq!(System::events().len(), 1);
	});
}

#[test]
fn test_disable_wasm_upgrade_flag_event() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		_ = TemplateModule::disable_wasm_upgrade_flag(Origin::signed(1));

		let expected_event = pallet_template::Event::DisableUpgrade(false, 0);

		System::assert_last_event(Event::TemplateModule(expected_event.into()));
		assert_eq!(System::events().len(), 1);
	});
}
