use cheetah_common::commands::types::create::CreateGameObjectCommand;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomMemberId;

use crate::room::command::{ServerCommandError, ServerCommandExecutor};
use crate::room::object::GameObject;
use crate::room::Room;

impl ServerCommandExecutor for CreateGameObjectCommand {
	fn execute(&self, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let member = room.get_member(&member_id)?;

		if self.object_id.id == 0 {
			return Err(ServerCommandError::Error("0 is forbidden for game object id".to_owned()));
		}

		let groups = self.access_groups;

		if !groups.is_sub_groups(&member.template.groups) {
			return Err(ServerCommandError::Error(format!(
				"Incorrect access group {:?} with client groups {:?}",
				groups, member.template.groups
			)));
		}

		if let GameObjectOwner::Member(object_id_member) = self.object_id.owner {
			if object_id_member != member.id {
				return Err(ServerCommandError::Error(format!(
					"Incorrect object_id {:?} for member {member:?}",
					self.object_id
				)));
			}
		}

		if room.contains_object(&self.object_id) {
			return Err(ServerCommandError::Error(format!("Object already exists with id {:?}", self.object_id)));
		}
		room.insert_object(GameObject::new(self.object_id, self.template, groups, false));
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::types::create::CreateGameObjectCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::command::{ServerCommandError, ServerCommandExecutor};
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	#[test]
	fn should_create() {
		let (mut room, member_id) = setup(AccessGroups(0b11));
		room.test_mark_as_connected(member_id).unwrap();

		let object_id = GameObjectId::new(1, GameObjectOwner::Member(member_id));
		let command = CreateGameObjectCommand {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b10),
		};
		command.execute(&mut room, member_id).unwrap();

		assert!(matches!(
			room.get_object_mut(object_id),
			Ok(object)
				if object.template_id == command.template
				&& object.access_groups == command.access_groups
		));
	}

	///
	/// Проверяем что `owner_id` в идентификаторе объекта должен быть равен текущему
	///
	#[test]
	fn should_not_create_when_owner_in_object_id_is_wrong() {
		let (mut room, member_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(1, GameObjectOwner::Member(1000));
		let command = CreateGameObjectCommand {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b10),
		};

		assert!(matches!(command.execute(&mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	///
	/// `AccessGroup` нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_access_group_is_wrong() {
		let (mut room, member_id) = setup(AccessGroups(0b11));
		let object_id = GameObjectId::new(1, GameObjectOwner::Member(member_id));
		let command = CreateGameObjectCommand {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		assert!(matches!(command.execute(&mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	///
	/// `AccessGroup` нового объекта не должна быть больше чем группа клиента
	///
	#[test]
	fn should_not_create_when_id_is_zero() {
		let (mut room, member_id) = setup(AccessGroups(0b11));

		let object_id = GameObjectId::new(0, GameObjectOwner::Member(member_id));
		let command = CreateGameObjectCommand {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b11),
		};
		assert!(matches!(command.execute(&mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Err(_)));
	}

	//
	/// Если есть объект - то он не должен быть замещен
	///
	#[test]
	fn should_not_replace_exists_object() {
		let access_groups = AccessGroups(0b11);
		let (mut room, member_id) = setup(access_groups);
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.template_id = 777;
		let object_id = object.id;
		room.test_out_commands.clear();
		let command = CreateGameObjectCommand {
			object_id,
			template: 100,
			access_groups: AccessGroups(0b1000),
		};

		assert!(matches!(command.execute(&mut room, member_id), Err(ServerCommandError::Error(_))));
		assert!(matches!(room.get_object_mut(object_id), Ok(object) if object.template_id == 777));
	}

	fn setup(access_groups: AccessGroups) -> (Room, u16) {
		let template = RoomTemplate::default();
		let mut room = Room::from_template(template);
		let member_id = room.register_member(MemberTemplate::stub(access_groups));
		(room, member_id)
	}
}