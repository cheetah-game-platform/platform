use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor, error_c2s_command, trace_c2s_command};
use crate::relay::room::clients::Client;
use crate::relay::room::groups::Access;
use crate::relay::room::objects::ErrorGetObjectWithCheckAccess;
use crate::relay::room::room::Room;

/// удаление игрового объекта
#[derive(Debug)]
pub struct DeleteGameObjectC2SCommand {
	pub global_object_id: u64
}


impl C2SCommandDecoder for DeleteGameObjectC2SCommand {
	const COMMAND_ID: u8 = 2;
	fn decode(bytes: &mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>> {
		return match bytes.read_u64() {
			Ok(id) => {
				Option::Some(Box::new(DeleteGameObjectC2SCommand { global_object_id: id }))
			}
			Err(_) => Option::None,
		};
	}
}

impl C2SCommandExecutor for DeleteGameObjectC2SCommand {
	fn execute(&self, client: &Client, room: &mut Room) {
		trace_c2s_command("DeleteGameObject", room, client, format!("params {:?}", self));
		let result = room.get_object_with_check_access(Access::ROOT, client, self.global_object_id);
		match result {
			Ok(object) => {
				room.objects.delete_object(object.clone().borrow().id)
			}
			Err(error) => {
				match error {
					ErrorGetObjectWithCheckAccess::ObjectNotFound => {
						error_c2s_command("DeleteGameObject", room, client, format!("object not found {}", self.global_object_id));
					}
					ErrorGetObjectWithCheckAccess::AccessNotAllowed => {
						error_c2s_command("DeleteGameObject", room, client, format!("access not allowed {}", self.global_object_id));
					}
				}
			}
		}
	}
}

