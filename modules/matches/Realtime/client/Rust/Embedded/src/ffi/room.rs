use cheetah_matches_realtime::room::template::config::RoomTemplate;
use cheetah_matches_realtime_common::room::RoomId;
use std::sync::Arc;

use crate::ffi::{ServerId, REGISTRY};

#[no_mangle]
pub(crate) extern "C" fn create_room(server_id: ServerId, room_id: &mut RoomId, on_error: extern "C" fn(*const u16)) -> bool {
	let mut registry = REGISTRY.lock().unwrap();
	return if let Some(server) = registry.servers.get_mut(&server_id) {
		let manager = Arc::clone(&server.manager);
		match server
			.runtime
			.block_on(async move { manager.lock().await.create_room(RoomTemplate::default()) })
		{
			Ok(create_room_id) => {
				*room_id = create_room_id;
				true
			}
			Err(e) => {
				on_error(widestring::U16CString::from_str(format!("{:?}", e)).unwrap().as_ptr());
				false
			}
		}
	} else {
		on_error(widestring::U16CString::from_str("Embedded server not found").unwrap().as_ptr());
		false
	};
}
#[cfg(test)]
mod test {
	use crate::ffi::room::create_room;
	use crate::ffi::server::{run_new_server, EmbeddedServerDescription};

	#[test]
	pub(crate) fn should_create_room() {
		let mut result = EmbeddedServerDescription::default();
		run_new_server(&mut result, on_server_error, &Default::default());
		let mut room_id = 0;
		assert!(create_room(result.id, &mut room_id, on_room_error));
		assert_eq!(room_id, 1);
	}

	pub(crate) extern "C" fn on_server_error(message: *const u16) {
		panic!("Fail create server with message {:?}", message)
	}

	pub(crate) extern "C" fn on_room_error(_: *const u16) {
		panic!("Fail create room")
	}
}
