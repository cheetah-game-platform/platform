use std::ffi::CStr;
use std::fmt::Display;
use std::os::raw::c_char;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::registry::{Registry, ServerPlugin, ServerPluginId};

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

#[repr(u8)]
pub enum ResultCode {
	OK = 0,
	Error = 1,
}

#[no_mangle]
pub unsafe extern "C" fn create_plugin(grpc_server_addr: *const c_char, out_plugin_id: &mut ServerPluginId) -> ResultCode {
	let server_address = CStr::from_ptr(grpc_server_addr).to_str().unwrap().to_string();
	// execute(|r|{
	// 	let plugin = ServerPlugin::new();
	// })

	ResultCode::OK
}

pub fn execute<F, R, E: Display>(body: F) -> ResultCode
where
	F: FnOnce(&mut Registry) -> Result<R, E>,
{
	let mut lock = REGISTRY.lock();
	match lock.as_mut() {
		Ok(registry) => match body(registry) {
			Ok(_) => ResultCode::OK,
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
