use core::fmt;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Instant;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;
use serde::export::Formatter;

use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::commands::command::S2CCommandWithMeta;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::execute;
use crate::room::object::GameObject;
use crate::room::template::{RoomTemplate, UserTemplate};
use crate::rooms::OutFrame;

pub mod command;
pub mod object;
pub mod template;

pub type RoomId = u64;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	pub users: HashMap<UserPublicKey, User, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ApplicationCommandChannelType>,
	current_meta: Option<C2SMetaCommandInformation>,
	current_user: Option<UserPublicKey>,
	pub user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>,
	pub auto_create_user: bool,
	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	pub out_commands: VecDeque<(AccessGroups, S2CCommand)>,
	#[cfg(test)]
	pub out_commands_by_users: HashMap<UserPublicKey, VecDeque<S2CCommand>>,
}

pub trait RoomUserListener {
	fn register_user(&mut self, room_id: RoomId, template: &UserTemplate);
	fn connected_user(&mut self, room_id: RoomId, template: &UserTemplate);
	fn disconnected_user(&mut self, room_id: RoomId, template: &UserTemplate);
}
impl Debug for dyn RoomUserListener {
	fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
		Result::Ok(())
	}
}

#[derive(Debug)]
pub struct User {
	protocol: Option<RelayProtocol>,
	pub attached: bool,
	pub template: UserTemplate,
}

#[derive(Debug)]
pub enum RoomRegisterUserError {
	AlreadyRegistered,
}

impl User {
	pub fn attach_to_room(&mut self) {
		self.attached = true;
	}
}

impl Room {
	pub fn new(template: RoomTemplate, user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> Self {
		let mut room = Room {
			id: template.id,
			auto_create_user: template.auto_create_user,
			users: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_meta: Default::default(),
			current_user: Default::default(),
			user_listeners,
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			#[cfg(test)]
			out_commands_by_users: Default::default(),
		};

		template.objects.unwrap_or_default().into_iter().for_each(|object| {
			let game_object: GameObject = object.to_root_game_object();
			room.insert_object(game_object);
		});

		template.users.into_iter().for_each(|config| room.register_user(config).unwrap());
		room
	}

	pub fn send_to_group(&mut self, include_self: bool, access_groups: AccessGroups, command: S2CCommand) {
		#[cfg(test)]
		self.out_commands.push_front((access_groups, command.clone()));

		if self.current_user.is_none() {
			return;
		}

		let current_user_public_key = self.current_user.as_ref().unwrap();
		let meta = self.current_meta.as_ref().unwrap();
		let channel_type = self.current_channel.as_ref().unwrap();
		let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
			meta: S2CMetaCommandInformation::new(current_user_public_key.clone(), meta),
			command: command.clone(),
		});

		let room_id = self.id;
		self.users
			.values_mut()
			.filter(|user| include_self || user.template.public_key != *current_user_public_key)
			.filter(|user| user.attached)
			.filter(|user| user.protocol.is_some())
			.filter(|user| user.template.access_groups.contains_any(&access_groups))
			.for_each(|user| {
				let protocol = user.protocol.as_mut().unwrap();
				log::info!("[room({:?})] s -> u({:?}) {:?}", room_id, user.template.public_key, command);
				protocol
					.out_commands_collector
					.add_command(channel_type.clone(), application_command.clone())
			});
	}

	pub fn send_to_user(&mut self, user_public_key: &u32, command: S2CCommand) {
		#[cfg(test)]
		{
			let commands = self.out_commands_by_users.entry(user_public_key.clone()).or_insert(VecDeque::new());
			commands.push_front(command.clone());
		}

		match self.users.get_mut(user_public_key) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_public_key)
			}
			Some(user) => {
				if let Some(ref mut protocol) = user.protocol {
					if user.attached {
						log::info!("[room({:?})] s -> u({:?}) {:?}", self.id, user.template.public_key, command);
						let meta = self.current_meta.as_ref().unwrap();
						let channel = self.current_channel.as_ref().unwrap();
						let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
							meta: S2CMetaCommandInformation::new(user_public_key.clone(), meta),
							command,
						});
						protocol.out_commands_collector.add_command(channel.clone(), application_command.clone());
					}
				}
			}
		}
	}

	pub fn collect_out_frame(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		for (user_public_key, user) in self.users.iter_mut() {
			if let Some(ref mut protocol) = user.protocol {
				if let Some(frame) = protocol.build_next_frame(&now) {
					out_frames.push_front(OutFrame {
						user_public_key: user_public_key.clone(),
						frame,
					});
				}
			}
		}
	}

	pub fn process_in_frame(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		let user = self.users.get_mut(&user_public_key);
		let mut commands = Vec::new();
		match user {
			None => {
				log::error!("[room({:?})] user({:?}) not found for input frame", self.id, user_public_key);
			}
			Some(user) => {
				self.current_user.replace(user_public_key.clone());

				let mut new_user = false;
				let protocol = &mut user.protocol;
				if protocol.is_none() {
					protocol.replace(RelayProtocol::new(now));
					new_user = true;
				}

				let protocol = protocol.as_mut().unwrap();
				protocol.on_frame_received(frame, now);
				while let Some(application_command) = protocol.in_commands_collector.get_commands().pop_back() {
					commands.push(application_command);
				}

				if new_user {
					self.current_channel.replace(ApplicationCommandChannelType::ReliableSequenceByGroup(0));
					self.current_meta.replace(C2SMetaCommandInformation { timestamp: 0 });
					let template = user.template.clone();
					self.user_connected(template);
				}
			}
		}

		for application_command in commands.into_iter() {
			match application_command.command {
				ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
					self.current_channel.replace(From::from(&application_command.channel));
					self.current_meta.replace(command_with_meta.meta.clone());
					execute(command_with_meta.command, self, &user_public_key);
				}
				_ => {
					log::error!("[room({:?})] receive unsupported command {:?}", self.id, application_command)
				}
			}
		}

		self.current_user = None;
		self.current_channel = None;
		self.current_meta = None;
	}

	pub fn register_user(&mut self, template: UserTemplate) -> Result<(), RoomRegisterUserError> {
		if self.users.contains_key(&template.public_key) {
			return Result::Err(RoomRegisterUserError::AlreadyRegistered);
		}

		self.user_listeners.iter().cloned().for_each(|listener| {
			let mut listener = (*listener).borrow_mut();
			listener.register_user(self.id.clone(), &template);
		});

		let user = User {
			protocol: None,
			attached: false,
			template,
		};
		self.users.insert(user.template.public_key, user);
		Result::Ok(())
	}

	pub fn get_user(&self, user_public_key: &UserPublicKey) -> Option<&User> {
		self.users.get(user_public_key)
	}

	pub fn get_user_mut(&mut self, user_public_key: &UserPublicKey) -> Option<&mut User> {
		self.users.get_mut(user_public_key)
	}

	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_user(&mut self, user_public_key: &UserPublicKey) {
		log::info!("[room({:?})] disconnect user({:?})", self.id, user_public_key);
		match self.users.remove(user_public_key) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let ObjectOwner::User(owner) = o.id.owner {
						if owner == user.template.public_key {
							objects.push((o.id.clone(), o.access_groups.clone()));
						}
					}
				});

				for (id, access_groups) in objects {
					self.delete_object(&id);
					self.send_to_group(false, access_groups, S2CCommand::Delete(DeleteGameObjectCommand { object_id: id }));
				}

				self.user_listeners.iter().cloned().for_each(|listener| {
					let mut listener = (*listener).borrow_mut();
					listener.disconnected_user(self.id.clone(), &user.template);
				});

				if self.auto_create_user {
					self.register_user(user.template.clone()).unwrap();
				}
			}
		};
	}

	pub fn insert_object(&mut self, object: GameObject) {
		self.send_to_group(
			true,
			object.access_groups,
			S2CCommand::Create(CreateGameObjectCommand {
				object_id: object.id.clone(),
				template: object.template.clone(),
				access_groups: object.access_groups.clone(),
				fields: object.fields.clone(),
			}),
		);
		self.objects.insert(object.id.clone(), object);
	}

	pub fn get_object(&mut self, object_id: &GameObjectId) -> Option<&mut GameObject> {
		match self.objects.get_mut(object_id) {
			Some(object) => Option::Some(object),
			None => {
				log::error!("[room] game object not found {:?}", object_id);
				Option::None
			}
		}
	}

	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}

	pub fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
		self.objects.remove(object_id)
	}

	pub fn process_objects(&self, f: &mut dyn FnMut(&GameObject) -> ()) {
		self.objects.iter().for_each(|(_, o)| f(o));
	}

	///
	/// Тактируем протоколы пользователей и определяем дисконнекты
	///
	pub fn cycle(&mut self, now: &Instant) {
		let mut disconnected_user: [u32; 10] = [0; 10];
		let mut disconnected_users_count = 0;
		self.users.values_mut().for_each(|u| {
			if let Some(ref mut protocol) = u.protocol {
				if protocol.disconnected(now) && disconnected_users_count < disconnected_user.len() {
					disconnected_user[disconnected_users_count] = u.template.public_key.clone();
					disconnected_users_count += 1;
				}
			}
		});

		for i in 0..disconnected_users_count {
			self.disconnect_user(&disconnected_user[i]);
		}
	}
	fn user_connected(&mut self, template: UserTemplate) {
		self.user_listeners.iter().cloned().for_each(|listener| {
			let mut listener = (*listener).borrow_mut();
			listener.connected_user(self.id.clone(), &template);
		});
		let user_public_key = template.public_key;
		match template.objects {
			None => {}
			Some(ref objects) => {
				objects.iter().for_each(|object_template| {
					let object = object_template.to_user_game_object(user_public_key);
					self.insert_object(object);
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;
	use std::time::Instant;

	use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
	use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::object::GameObject;
	use crate::room::template::{GameObjectTemplate, RoomTemplate, UserTemplate};
	use crate::room::{Room, RoomUserListener};

	impl Room {
		pub fn create_object(&mut self, owner: &UserPublicKey) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, ObjectOwner::User(owner.clone()));
			let object = GameObject {
				id: id.clone(),
				template: 0,
				access_groups: Default::default(),
				fields: Default::default(),
			};
			self.insert_object(object);
			self.get_object(&id).unwrap()
		}

		pub fn create_object_with_access_groups(&mut self, access_groups: AccessGroups) -> &mut GameObject {
			let object = self.create_object(&0);
			object.access_groups = access_groups;
			object
		}
	}

	#[test]
	fn should_remove_objects_when_disconnect() {
		let mut template = RoomTemplate::default();
		let user_a = template.create_user(1, AccessGroups(0b111));
		let user_b = template.create_user(2, AccessGroups(0b111));

		let mut room = Room::new(template, Default::default());
		let object_a_1 = room.create_object(&user_a).id.clone();
		let object_a_2 = room.create_object(&user_a).id.clone();
		let object_b_1 = room.create_object(&user_b).id.clone();
		let object_b_2 = room.create_object(&user_b).id.clone();

		room.out_commands.clear();
		room.disconnect_user(&user_a);

		assert!(!room.contains_object(&object_a_1));
		assert!(!room.contains_object(&object_a_2));

		assert!(room.contains_object(&object_b_1));
		assert!(room.contains_object(&object_b_2));
		println!("{:?}", room.out_commands);

		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_1));
		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_2));
	}

	#[test]
	fn should_create_object_from_config() {
		let mut template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			access_groups: Default::default(),
			fields: Default::default(),
		};
		template.objects = Option::Some(vec![object_template.clone()]);

		let room = Room::new(template, Default::default());
		assert!(room.objects.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::Root)));
	}

	#[test]
	fn should_create_object_from_config_for_user() {
		let mut template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user_template = UserTemplate {
			public_key: 100,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Option::Some(vec![object_template.clone()]),
		};
		template.users.push(user_template.clone());

		let mut room = Room::new(template, Default::default());
		room.process_in_frame(&user_template.public_key, Frame::new(0), &Instant::now());
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::User(user_template.public_key))));
	}

	///
	/// Пользовательские объекты из шаблона должны загружаться на первый клиент при входе второго
	///
	#[test]
	fn should_load_user_object_when_connect_other_user() {
		let mut template = RoomTemplate::default();
		let object1_template = GameObjectTemplate {
			id: 100,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user1_template = UserTemplate {
			public_key: 1,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Option::Some(vec![object1_template.clone()]),
		};

		let object2_template = GameObjectTemplate {
			id: 200,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user2_template = UserTemplate {
			public_key: 2,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Option::Some(vec![object2_template.clone()]),
		};

		template.users.push(user1_template.clone());
		template.users.push(user2_template.clone());

		let mut room = Room::new(template, Default::default());

		room.process_in_frame(&user1_template.public_key, Frame::new(0), &Instant::now());
		let mut frame_with_attach_to_room = Frame::new(1);
		frame_with_attach_to_room.commands.reliable.push(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
				meta: C2SMetaCommandInformation { timestamp: 0 },
				command: C2SCommand::AttachToRoom,
			}),
		});
		room.process_in_frame(&user1_template.public_key, frame_with_attach_to_room, &Instant::now());

		let user1 = room.get_user_mut(&user1_template.public_key).unwrap();
		let protocol = user1.protocol.as_mut().unwrap();
		assert_eq!(
			protocol
				.out_commands_collector
				.commands
				.reliable
				.pop()
				.unwrap()
				.command
				.get_object_id()
				.unwrap(),
			&GameObjectId::new(object1_template.id, ObjectOwner::User(user1_template.public_key))
		);

		room.process_in_frame(&user2_template.public_key, Frame::new(0), &Instant::now());
		let user1 = room.get_user_mut(&user1_template.public_key).unwrap();
		let protocol = user1.protocol.as_mut().unwrap();
		assert_eq!(
			protocol
				.out_commands_collector
				.commands
				.reliable
				.pop()
				.unwrap()
				.command
				.get_object_id()
				.unwrap(),
			&GameObjectId::new(object2_template.id, ObjectOwner::User(user2_template.public_key))
		);
	}

	///
	/// Регистрация пользователя после разрыва соединения если выставлен флаг автосоздания
	///
	#[test]
	fn should_register_user_after_disconnect_when_auto_create() {
		let mut template = RoomTemplate::default();
		template.auto_create_user = true;
		let user_template = UserTemplate {
			public_key: 100,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Default::default(),
		};
		template.users.push(user_template.clone());

		let mut room = Room::new(template, Default::default());
		room.disconnect_user(&user_template.public_key);
		assert!(room.users.contains_key(&user_template.public_key));
	}

	pub fn from_vec(vec: Vec<u8>) -> heapless::Vec<u8, heapless::consts::U256> {
		let mut result = heapless::Vec::new();
		result.extend_from_slice(vec.as_slice()).unwrap();
		result
	}

	#[test]
	fn should_invoke_user_listeners() {
		struct TestUserListener {
			trace: String,
		}

		impl RoomUserListener for TestUserListener {
			fn register_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}r{:?}", self.trace, template.public_key);
			}

			fn connected_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}c{:?}", self.trace, template.public_key);
			}

			fn disconnected_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}d{:?}", self.trace, template.public_key);
			}
		}

		let mut template = RoomTemplate::default();
		let user_template = UserTemplate {
			public_key: 100,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Default::default(),
		};
		template.users.push(user_template.clone());

		let test_listener = Rc::new(RefCell::new(TestUserListener { trace: "".to_string() }));
		let mut room = Room::new(template, vec![test_listener.clone()]);
		room.process_in_frame(&user_template.public_key, Frame::new(0), &Instant::now());
		room.disconnect_user(&user_template.public_key);

		assert_eq!(test_listener.clone().borrow().trace, "r100c100d100".to_string());
	}
}
