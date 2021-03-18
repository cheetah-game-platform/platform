use std::sync::Mutex;

use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::command::S2CMetaCommandInformationFFI;
use cheetah_relay_client::ffi::GameObjectIdFFI;
use cheetah_relay_common::constants::FieldId;

use crate::helpers::helper::*;

#[test]
fn should_inc() {
	let (helper, client1, client2) = setup(Default::default());

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	ffi::command::float_value::inc_float_value(&object_id, 1, 100.0);
	ffi::command::float_value::inc_float_value(&object_id, 1, 100.0);

	ffi::client::set_current_client(client2);
	ffi::command::float_value::set_float_value_listener(listener_for_inc);
	ffi::command::room::attach_to_room();
	helper.wait_udp();
	ffi::client::receive();

	println!("{:?}", INCR.lock().unwrap().as_ref());
	assert!(matches!(INCR.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200.0 ));
}

#[test]
fn should_set() {
	let (helper, client1, client2) = setup(Default::default());

	ffi::client::set_current_client(client1);
	let object_id = helper.create_user_object();
	ffi::command::float_value::set_float_value(&object_id, 1, 100.0);
	ffi::command::float_value::set_float_value(&object_id, 1, 200.0);

	ffi::client::set_current_client(client2);
	ffi::command::float_value::set_float_value_listener(listener_for_set);
	ffi::command::room::attach_to_room();
	helper.wait_udp();
	ffi::client::receive();

	println!("{:?}", SET.lock().unwrap().as_ref());
	assert!(matches!(SET.lock().unwrap().as_ref(),Option::Some((field_id, value)) if *field_id == 1 && *value==200.0 ));
}

lazy_static! {
	static ref INCR: Mutex<Option<(FieldId, f64)>> = Mutex::new(Default::default());
}

lazy_static! {
	static ref SET: Mutex<Option<(FieldId, f64)>> = Mutex::new(Default::default());
}

extern "C" fn listener_for_set(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: f64) {
	SET.lock().unwrap().replace((field_id, value));
}

extern "C" fn listener_for_inc(_: &S2CMetaCommandInformationFFI, _object_id: &GameObjectIdFFI, field_id: FieldId, value: f64) {
	INCR.lock().unwrap().replace((field_id, value));
}