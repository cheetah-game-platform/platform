use std::io::Cursor;

use strum_macros::AsRefStr;

use crate::commands::binary_value::BinaryValue;
use crate::commands::field::{Field, FieldId};
use crate::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
use crate::commands::types::delete::DeleteGameObjectCommand;
use crate::commands::types::event::{EventCommand, TargetEventCommand};
use crate::commands::types::field::{DeleteFieldCommand, SetFieldCommand};
use crate::commands::types::float::IncrementDoubleC2SCommand;
use crate::commands::types::forwarded::ForwardedCommand;
use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
use crate::commands::types::structure::CompareAndSetStructureCommand;
use crate::commands::{CommandDecodeError, CommandTypeId, FieldType, FieldValue};
use crate::protocol::codec::commands::context::CommandContextError;
use crate::room::object::GameObjectId;

#[derive(Debug, PartialEq, Clone, AsRefStr)]
#[allow(clippy::large_enum_variant)]
pub enum C2SCommand {
	CreateGameObject(CreateGameObjectCommand),
	CreatedGameObject(C2SCreatedGameObjectCommand),
	IncrementLongValue(IncrementLongC2SCommand),
	CompareAndSetLong(CompareAndSetLongCommand),
	SetField(SetFieldCommand),
	IncrementDouble(IncrementDoubleC2SCommand),
	CompareAndSetStructure(CompareAndSetStructureCommand),
	Event(EventCommand),
	TargetEvent(TargetEventCommand),
	Delete(DeleteGameObjectCommand),
	DeleteField(DeleteFieldCommand),
	///
	/// Загрузить все объекты комнаты
	///
	AttachToRoom,
	DetachFromRoom,
	Forwarded(Box<ForwardedCommand>),
}

impl C2SCommand {
	#[must_use]
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::SetField(command) => Some(command.field_id),
			C2SCommand::IncrementLongValue(command) => Some(command.field_id),
			C2SCommand::CompareAndSetLong(command) => Some(command.field_id),
			C2SCommand::CompareAndSetStructure(command) => Some(command.field_id),
			C2SCommand::IncrementDouble(command) => Some(command.field_id),
			C2SCommand::Event(command) => Some(command.field_id),
			C2SCommand::TargetEvent(command) => Some(command.event.field_id),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_id),
			C2SCommand::Forwarded(command) => command.c2s.get_field_id(),
		}
	}
	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			C2SCommand::CreateGameObject(command) => Some(command.object_id),
			C2SCommand::CreatedGameObject(command) => Some(command.object_id),
			C2SCommand::SetField(command) => Some(command.object_id),
			C2SCommand::IncrementLongValue(command) => Some(command.object_id),
			C2SCommand::CompareAndSetLong(command) => Some(command.object_id),
			C2SCommand::IncrementDouble(command) => Some(command.object_id),
			C2SCommand::Event(command) => Some(command.object_id),
			C2SCommand::TargetEvent(command) => Some(command.event.object_id),
			C2SCommand::Delete(command) => Some(command.object_id),
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.object_id),
			C2SCommand::CompareAndSetStructure(command) => Some(command.object_id),
			C2SCommand::Forwarded(command) => command.c2s.get_object_id(),
		}
	}

	#[must_use]
	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			C2SCommand::CreateGameObject(_) => None,
			C2SCommand::CreatedGameObject(_) => None,
			C2SCommand::SetField(command) => Some(command.value.field_type()),
			C2SCommand::IncrementLongValue(_) => Some(FieldType::Long),
			C2SCommand::CompareAndSetLong(_) => Some(FieldType::Long),
			C2SCommand::IncrementDouble(_) => Some(FieldType::Double),
			C2SCommand::Event(_) => Some(FieldType::Event),
			C2SCommand::TargetEvent(_) => Some(FieldType::Event),
			C2SCommand::Delete(_) => None,
			C2SCommand::AttachToRoom => None,
			C2SCommand::DetachFromRoom => None,
			C2SCommand::DeleteField(command) => Some(command.field_type),
			C2SCommand::CompareAndSetStructure(_) => Some(FieldType::Structure),
			C2SCommand::Forwarded(command) => command.c2s.get_field_type(),
		}
	}

	#[must_use]
	pub fn get_type_id(&self) -> CommandTypeId {
		match self {
			C2SCommand::CreateGameObject(_) => CommandTypeId::CreateGameObject,
			C2SCommand::CreatedGameObject(_) => CommandTypeId::CreatedGameObject,
			C2SCommand::SetField(command) => match command.value {
				FieldValue::Long(_) => CommandTypeId::SetLong,
				FieldValue::Double(_) => CommandTypeId::SetDouble,
				FieldValue::Structure(_) => CommandTypeId::SetStructure,
			},
			C2SCommand::IncrementLongValue(_) => CommandTypeId::IncrementLong,
			C2SCommand::CompareAndSetLong(_) => CommandTypeId::CompareAndSetLong,
			C2SCommand::CompareAndSetStructure(_) => CommandTypeId::CompareAndSetStructure,
			C2SCommand::IncrementDouble(_) => CommandTypeId::IncrementDouble,
			C2SCommand::Event(_) => CommandTypeId::Event,
			C2SCommand::TargetEvent(_) => CommandTypeId::TargetEvent,
			C2SCommand::Delete(_) => CommandTypeId::Delete,
			C2SCommand::AttachToRoom => CommandTypeId::AttachToRoom,
			C2SCommand::DetachFromRoom => CommandTypeId::DetachFromRoom,
			C2SCommand::DeleteField(_) => CommandTypeId::DeleteField,
			C2SCommand::Forwarded(_) => CommandTypeId::Forwarded,
		}
	}

	#[must_use]
	pub fn get_field(&self) -> Option<Field> {
		if let (Some(id), Some(field_type)) = (self.get_field_id(), self.get_field_type()) {
			Some(Field { id, field_type })
		} else {
			None
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			C2SCommand::CreateGameObject(command) => command.encode(out),
			C2SCommand::CreatedGameObject(command) => command.encode(out),
			C2SCommand::SetField(command) => command.encode(out),
			C2SCommand::IncrementLongValue(command) => command.encode(out),
			C2SCommand::CompareAndSetLong(command) => command.encode(out),
			C2SCommand::IncrementDouble(command) => command.encode(out),
			C2SCommand::Event(command) => command.encode(out),
			C2SCommand::TargetEvent(command) => command.encode(out),
			C2SCommand::Delete(_) => Ok(()),
			C2SCommand::AttachToRoom => Ok(()),
			C2SCommand::DetachFromRoom => Ok(()),
			C2SCommand::DeleteField(command) => command.encode(out),
			C2SCommand::CompareAndSetStructure(command) => command.encode(out),
			C2SCommand::Forwarded(command) => command.encode(out),
		}
	}

	#[must_use]
	pub fn get_trace_string(&self) -> String {
		match self {
			C2SCommand::CreateGameObject(command) => format!("access({:?}), template({:?}) ", command.access_groups.0, command.template),
			C2SCommand::CreatedGameObject(command) => {
				format!("room_owner({:?}), singleton_key ({:?}) ", command.room_owner, command.get_singleton_key())
			}
			C2SCommand::IncrementLongValue(command) => format!("{:?}", command.increment),
			C2SCommand::CompareAndSetLong(command) => {
				format!(
					"new = {:?}, current = {:?}, reset = {:?}",
					command.new,
					command.current,
					command.get_reset()
				)
			}
			C2SCommand::SetField(command) => format!("{:?}", command.value),
			C2SCommand::IncrementDouble(command) => format!("{:?}", command.increment),
			C2SCommand::CompareAndSetStructure(command) => {
				format!(
					"new = {:?}, current = {:?}, reset = {:?}",
					command.new,
					command.current,
					command.get_reset()
				)
			}
			C2SCommand::Event(command) => format!("{:?}", command.event),
			C2SCommand::TargetEvent(command) => format!("target_member = {:?}, value = {:?}", command.target, command.event.event),
			C2SCommand::Delete(_) => String::new(),
			C2SCommand::DeleteField(command) => format!("field_type = {:?}", command.field_type),
			C2SCommand::AttachToRoom => String::new(),
			C2SCommand::DetachFromRoom => String::new(),
			C2SCommand::Forwarded(command) => format!("forward: member({:?}) command({:?})", command.creator, command.c2s.get_trace_string()),
		}
	}

	pub(crate) fn decode(
		command_type_id: CommandTypeId,
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<C2SCommand, CommandDecodeError> {
		Ok(match command_type_id {
			CommandTypeId::AttachToRoom => C2SCommand::AttachToRoom,
			CommandTypeId::DetachFromRoom => C2SCommand::DetachFromRoom,
			CommandTypeId::CreatedGameObject => C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::Delete => C2SCommand::Delete(DeleteGameObjectCommand { object_id: object_id? }),
			CommandTypeId::CreateGameObject => C2SCommand::CreateGameObject(CreateGameObjectCommand::decode(object_id?, input)?),
			CommandTypeId::IncrementLong => C2SCommand::IncrementLongValue(IncrementLongC2SCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::IncrementDouble => C2SCommand::IncrementDouble(IncrementDoubleC2SCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::CompareAndSetLong => C2SCommand::CompareAndSetLong(CompareAndSetLongCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::CompareAndSetStructure => {
				C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand::decode(object_id?, field_id?, input)?)
			}
			CommandTypeId::SetDouble => C2SCommand::SetField(SetFieldCommand::decode::<f64>(object_id?, field_id?, input)?),
			CommandTypeId::SetLong => C2SCommand::SetField(SetFieldCommand::decode::<i64>(object_id?, field_id?, input)?),
			CommandTypeId::SetStructure => C2SCommand::SetField(SetFieldCommand::decode::<BinaryValue>(object_id?, field_id?, input)?),
			CommandTypeId::Event => C2SCommand::Event(EventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::TargetEvent => C2SCommand::TargetEvent(TargetEventCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::DeleteField => C2SCommand::DeleteField(DeleteFieldCommand::decode(object_id?, field_id?, input)?),
			CommandTypeId::Forwarded => C2SCommand::Forwarded(Box::new(ForwardedCommand::decode(object_id, field_id, input)?)),
			CommandTypeId::MemberConnected => return Err(CommandDecodeError::UnknownTypeId(command_type_id)),
		})
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::binary_value::BinaryValue;
	use crate::commands::c2s::C2SCommand;
	use crate::commands::field::FieldId;
	use crate::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
	use crate::commands::types::delete::DeleteGameObjectCommand;
	use crate::commands::types::event::{EventCommand, TargetEventCommand};
	use crate::commands::types::field::SetFieldCommand;
	use crate::commands::types::float::IncrementDoubleC2SCommand;
	use crate::commands::types::forwarded::ForwardedCommand;
	use crate::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
	use crate::commands::types::structure::CompareAndSetStructureCommand;
	use crate::commands::CommandTypeId;
	use crate::protocol::codec::commands::context::CommandContextError;
	use crate::room::access::AccessGroups;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_decode_encode_attach() {
		check(&C2SCommand::AttachToRoom, CommandTypeId::AttachToRoom, None, None);
	}
	#[test]
	fn should_decode_encode_detach() {
		check(&C2SCommand::DetachFromRoom, CommandTypeId::DetachFromRoom, None, None);
	}

	#[test]
	fn should_decode_encode_create_member_object() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&C2SCommand::CreateGameObject(CreateGameObjectCommand {
				object_id,
				template: 3,
				access_groups: AccessGroups(5),
			}),
			CommandTypeId::CreateGameObject,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_created() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand::new(object_id, false, None)),
			CommandTypeId::CreatedGameObject,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetField(SetFieldCommand {
				object_id,
				field_id,
				value: 100.into(),
			}),
			CommandTypeId::SetLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_increment_long_value() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
				object_id,
				field_id,
				increment: 100,
			}),
			CommandTypeId::IncrementLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_compare_and_set_long_value() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::CompareAndSetLong(CompareAndSetLongCommand::new(object_id, field_id, 100, 101, Some(102))),
			CommandTypeId::CompareAndSetLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_compare_and_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand::new(
				object_id,
				field_id,
				vec![100].as_slice().into(),
				vec![101].as_slice().into(),
				Some(vec![102].as_slice().into()),
			)),
			CommandTypeId::CompareAndSetStructure,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetField(SetFieldCommand {
				object_id,
				field_id,
				value: 3.15.into(),
			}),
			CommandTypeId::SetDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_incremental_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::IncrementDouble(IncrementDoubleC2SCommand {
				object_id,
				field_id,
				increment: 3.15,
			}),
			CommandTypeId::IncrementDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::SetField(SetFieldCommand {
				object_id,
				field_id,
				value: BinaryValue::from([1, 2, 3, 4].as_ref()).into(),
			}),
			CommandTypeId::SetStructure,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::Event(EventCommand {
				object_id,
				field_id,
				event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
			}),
			CommandTypeId::Event,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_target_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::TargetEvent(TargetEventCommand {
				target: 10,
				event: EventCommand {
					object_id,
					field_id,
					event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
				},
			}),
			CommandTypeId::TargetEvent,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&C2SCommand::Delete(DeleteGameObjectCommand { object_id }),
			CommandTypeId::Delete,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_forwarded() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&C2SCommand::Forwarded(Box::new(ForwardedCommand {
				creator: 123,
				c2s: C2SCommand::TargetEvent(TargetEventCommand {
					target: 10,
					event: EventCommand {
						object_id,
						field_id,
						event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
					},
				}),
			})),
			CommandTypeId::Forwarded,
			Some(object_id),
			Some(field_id),
		);
	}

	fn check(expected: &C2SCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = C2SCommand::decode(command_type_id, object_id, field_id, &mut read_cursor).unwrap();
		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, &actual);
	}
}
