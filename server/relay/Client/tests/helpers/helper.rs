use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use cheetah_relay::server::RelayServer;
use cheetah_relay_client::ffi;
use cheetah_relay_client::ffi::client::do_create_client;
use cheetah_relay_client::ffi::GameObjectIdFFI;
use cheetah_relay_client::registry::ClientId;
use cheetah_relay_common::room::{RoomId, UserId, UserPrivateKey};

use crate::helpers::server::IntegrationTestServerBuilder;

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	room_id: RoomId,
	pub server: RelayServer,
}

impl IntegrationTestHelper {
	pub fn new(builder: IntegrationTestServerBuilder) -> Self {
		let (socket_addr, server, room_id) = builder.build();
		Self {
			socket_addr,
			room_id,
			server,
		}
	}

	pub fn create_client(&self, user_id: UserId, user_key: UserPrivateKey) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(self.socket_addr.to_string(), user_id, self.room_id, &user_key, 0, &mut client);
		client
	}

	pub fn wait_udp(&self) {
		thread::sleep(Duration::from_millis(500));
	}

	pub fn create_user_object(&self) -> GameObjectIdFFI {
		let mut object_id = GameObjectIdFFI::new();
		ffi::command::object::create_object(
			IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
			IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
			&mut object_id,
		);
		ffi::command::object::created_object(&object_id);
		object_id
	}
}

pub fn setup(mut builder: IntegrationTestServerBuilder) -> (IntegrationTestHelper, u16, u16) {
	let (user1_id, user1_key) = builder.create_user();
	let (user2_id, user2_key) = builder.create_user();
	let helper = IntegrationTestHelper::new(builder);

	let client1 = helper.create_client(user1_id, user1_key);
	let client2 = helper.create_client(user2_id, user2_key);
	(helper, client1, client2)
}
