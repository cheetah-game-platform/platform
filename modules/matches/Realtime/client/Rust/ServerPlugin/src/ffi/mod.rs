use std::ffi::CStr;
use std::fmt::Display;
use std::os::raw::c_char;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::registry::{Registry, RoomId, ServerPlugin, ServerPluginId};

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

#[repr(u8)]
pub enum ResultCode {
	OK = 0,
	Empty = 1,
	Error = 2,
}

#[no_mangle]
pub unsafe extern "C" fn create_plugin(grpc_server_addr: *const c_char, out_plugin_id: &mut ServerPluginId) -> ResultCode {
	let grpc_server_addr_as_string = CStr::from_ptr(grpc_server_addr).to_str().unwrap().to_string();
	execute(|registry| {
		let plugin = ServerPlugin::new(grpc_server_addr_as_string);
		match plugin {
			Ok(plugin) => {
				let id = registry.register_plugin(plugin);
				*out_plugin_id = id;
				Ok(ResultCode::OK)
			}
			Err(err) => Err(err),
		}
	})
}

#[repr(C)]
pub struct RoomEvent {
	room_id: RoomId,
	event_type: RoomEventType,
}

#[repr(C)]
pub enum RoomEventType {
	Created = 0,
	Deleted = 1,
}

#[no_mangle]
pub unsafe extern "C" fn pop_room_event(plugin_id: ServerPluginId, out: &mut RoomEvent) -> ResultCode {
	execute::<_, String>(|registry| {
		let plugin = registry
			.get_plugin(plugin_id)
			.ok_or_else(|| format!("Plugin with id {} not found in registry", plugin_id))?;

		if let Some(room_id) = plugin.reader.pop_create_room().map_err(|e| format!("{}", e))? {
			out.room_id = room_id;
			out.event_type = RoomEventType::Created;
			Ok(ResultCode::OK)
		} else if let Some(room_id) = plugin.reader.pop_deleted_rooms().map_err(|e| format!("{}", e))? {
			out.room_id = room_id;
			out.event_type = RoomEventType::Deleted;
			Ok(ResultCode::OK)
		} else {
			Ok(ResultCode::Empty)
		}
	})
}

pub struct NativeString {
	pub len: u8,
	pub buffer: [u8; 255],
}

#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn get_last_error_msg(buffer: &mut NativeString) {
	let msg = LAST_ERROR.lock().unwrap();
	let msg = msg.as_bytes();
	let length = msg.len();
	buffer.len = length as u8;
	buffer.buffer[0..length].copy_from_slice(msg);
}

pub fn execute<F, E: Display>(body: F) -> ResultCode
where
	F: FnOnce(&mut Registry) -> Result<ResultCode, E>,
{
	let mut lock = REGISTRY.lock();
	match lock.as_mut() {
		Ok(registry) => match body(registry) {
			Ok(result) => result,
			Err(e) => {
				set_error_msg(format!("{}", e));
				ResultCode::Error
			}
		},
		Err(e) => {
			set_error_msg(format!("{:?}", e));
			ResultCode::Error
		}
	}
}

fn set_error_msg(msg: String) {
	let mut last_error = LAST_ERROR.lock().unwrap();
	*last_error = msg;
}
