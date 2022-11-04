use std::collections::HashMap;

use fnv::FnvBuildHasher;
use tonic::transport::Channel;

use crate::registry::created_room::CreateRoomEventReader;

pub mod created_room;
#[cfg(test)]
pub mod stubs;

pub type RoomId = u64;

#[derive(Default)]
pub struct Registry {
	pub plugins: HashMap<ServerPluginId, ServerPlugin, FnvBuildHasher>,
	server_plugin_generator_id: ServerPluginId,
}

pub type ServerPluginId = u16;

pub struct ServerPlugin {
	create_room_event_reader: CreateRoomEventReader,
}

impl ServerPlugin {
	pub fn new(server_grpc_addr: String) -> Result<Self, anyhow::Error> {
		// let c = Channel::from_shared(server_grpc_addr)?;
		// let connect = c.connect().await;
		todo!()
		// Ok(ServerPlugin {
		// 	create_room_event_reader: CreateRoomEventReader::new(channel?),
		// })
	}
}
