use std::sync::Mutex;

use lazy_static::lazy_static;

use cheetah_client::ffi;
use cheetah_common::commands::binary_value::BinaryValue;
use cheetah_common::commands::field::FieldId;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn test() {
	let (helper, [client1, client2]) = setup(IntegrationTestServerBuilder::default());

	ffi::command::event::set_event_listener(client2, on_event_listener);
	ffi::command::room::attach_to_room(client2);
	helper.wait_udp();

	let mut object_id = GameObjectId::default();
	ffi::command::object::create_object(client1, 1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	ffi::command::object::created_object(client1, &object_id, false, &BinaryValue::default());

	let mut event_buffer = BinaryValue {
		len: 1,
		..Default::default()
	};
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;
	ffi::command::event::send_event(client1, &object_id, event_field_id, &event_buffer);

	helper.wait_udp();
	ffi::client::receive(client2);

	assert!(matches!(EVENT.lock().unwrap().as_ref(),Some((field_id, buffer)) if *field_id == event_field_id && *buffer == event_buffer ));
}

lazy_static! {
	static ref EVENT: Mutex<Option<(FieldId, BinaryValue)>> = Mutex::new(Default::default());
}

extern "C" fn on_event_listener(_: RoomMemberId, _object_id: &GameObjectId, field_id: FieldId, buffer: &BinaryValue) {
	EVENT.lock().unwrap().replace((field_id, (*buffer).clone()));
}
