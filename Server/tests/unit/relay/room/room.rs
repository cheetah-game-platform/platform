use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::room::access::AccessGroups;

use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};
use cheetah_relay::room::Room;

use crate::unit::relay::room::{setup_and_two_client, setup_client, setup_listener};

#[test]
fn should_load_game_objects_when_new_client_connected() {
	let (mut room, client_a, _client_b) = setup_and_two_client();
	room
		.new_game_object(
			ServerGameObjectId::new(10, ServerOwner::Client(client_a.configuration.id)),
			123,
			AccessGroups::from(0b1),
			Default::default(),
		)
		.unwrap();
	room
		.new_game_object(
			ServerGameObjectId::new(20, ServerOwner::Client(client_a.configuration.id)),
			123,
			AccessGroups::from(0b1),
			Default::default(),
		)
		.unwrap();
	
	
	let listener = setup_listener(&mut room);
	setup_client(&mut room, "CLIENT_C", 0b1);
	let listener = &*listener;
	let listener = listener.borrow();
	println!("listener {:?}", listener);
}

pub fn hash_value_stub() -> HashValue {
	HashValue::from("room_hash")
}

pub fn room_stub() -> Room {
	Room {
		auto_create_client: false,
		hash: hash_value_stub(),
		listener: Default::default(),
		clients: Default::default(),
		objects: Default::default(),
	}
}