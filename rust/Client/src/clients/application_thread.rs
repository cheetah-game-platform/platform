use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use cheetah_common::commands::binary_value::BinaryValue;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::create::CreateGameObjectCommand;
use cheetah_common::commands::{FieldType, FieldValue};
use cheetah_common::network::client::ConnectionStatus;
use cheetah_common::protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use cheetah_common::protocol::frame::channel::ChannelType;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomMemberId;

use crate::clients::network_thread::C2SCommandWithChannel;
use crate::clients::{ClientRequest, SharedClientStatistics};
use crate::ffi::channel::Channel;
use crate::ffi::ForwardedCommandFFI;

///
/// Взаимодействие с сетевым потоком клиента, через Sender
///
pub struct ApplicationThreadClient {
	member_id: RoomMemberId,
	commands_from_server: Receiver<CommandWithChannel>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	server_time: Arc<Mutex<Option<u64>>>,
	request_to_client: Sender<ClientRequest>,
	channel: ChannelType,
	game_object_id_generator: u32,
	pub shared_statistics: SharedClientStatistics,
	pub listener_long_value: Option<extern "C" fn(RoomMemberId, &GameObjectId, FieldId, i64)>,
	pub listener_float_value: Option<extern "C" fn(RoomMemberId, &GameObjectId, FieldId, f64)>,
	pub listener_event: Option<extern "C" fn(RoomMemberId, &GameObjectId, FieldId, &BinaryValue)>,
	pub listener_structure: Option<extern "C" fn(RoomMemberId, &GameObjectId, FieldId, &BinaryValue)>,
	pub listener_delete_field: Option<extern "C" fn(RoomMemberId, &GameObjectId, FieldId, FieldType)>,
	pub listener_create_object: Option<extern "C" fn(&GameObjectId, u16)>,
	pub listener_delete_object: Option<extern "C" fn(&GameObjectId)>,
	pub listener_created_object: Option<extern "C" fn(&GameObjectId)>,
	pub listener_forwarded_command: Option<extern "C" fn(ForwardedCommandFFI)>,
	pub listener_member_connected: Option<extern "C" fn(RoomMemberId)>,
}

impl Drop for ApplicationThreadClient {
	fn drop(&mut self) {
		if self
			.request_to_client
			.send(ClientRequest::Close(DisconnectByCommandReason::ClientStopped))
			.is_ok()
		{
			self.handler.take().unwrap().join().unwrap();
		}
	}
}

impl ApplicationThreadClient {
	pub fn new(
		member_id: RoomMemberId,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Receiver<CommandWithChannel>,
		sender: Sender<ClientRequest>,
		shared_statistics: SharedClientStatistics,
		server_time: Arc<Mutex<Option<u64>>>,
	) -> Self {
		Self {
			member_id,
			commands_from_server: in_commands,
			handler: Some(handler),
			state,
			server_time,
			request_to_client: sender,
			channel: ChannelType::ReliableSequence(ChannelGroup(0)),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			shared_statistics,
			listener_long_value: None,
			listener_float_value: None,
			listener_event: None,
			listener_structure: None,
			listener_delete_object: None,
			listener_create_object: None,
			listener_created_object: None,
			listener_delete_field: None,
			listener_forwarded_command: None,
			listener_member_connected: None,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::SetProtocolTimeOffsetForTest(time_offset))
	}

	pub fn send(&mut self, command: C2SCommand) -> Result<(), SendError<ClientRequest>> {
		let out_command = C2SCommandWithChannel {
			channel_type: self.channel,
			command,
		};
		self.request_to_client.send(ClientRequest::SendCommandToServer(out_command))
	}

	pub fn get_connection_status(&self) -> Result<ConnectionStatus, PoisonError<MutexGuard<'_, ConnectionStatus>>> {
		Ok(self.state.lock()?.clone())
	}

	#[allow(clippy::unwrap_in_result)]
	pub fn get_server_time(&self) -> Option<u64> {
		*self.server_time.lock().unwrap()
	}

	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroup) {
		self.channel = match channel {
			Channel::ReliableUnordered => ChannelType::ReliableUnordered,
			Channel::UnreliableUnordered => ChannelType::UnreliableUnordered,
			Channel::ReliableOrdered => ChannelType::ReliableOrdered(group),
			Channel::UnreliableOrdered => ChannelType::UnreliableOrdered(group),
			Channel::ReliableSequence => ChannelType::ReliableSequence(group),
		}
	}

	pub fn receive(&mut self) {
		while let Ok(command) = self.commands_from_server.try_recv() {
			if let BothDirectionCommand::S2CWithCreator(member_with_creator) = command.both_direction_command {
				match member_with_creator.command {
					S2CCommand::Create(command) => {
						if let Some(ref listener) = self.listener_create_object {
							let object_id = command.object_id;
							listener(&object_id, command.template);
						}
					}
					S2CCommand::Created(command) => {
						if let Some(ref listener) = self.listener_created_object {
							let object_id = command.object_id;
							listener(&object_id);
						}
					}
					S2CCommand::SetField(command) => match command.value {
						FieldValue::Long(v) => {
							if let Some(ref listener) = self.listener_long_value {
								let object_id = command.object_id;
								listener(member_with_creator.creator, &object_id, command.field_id, v);
							}
						}
						FieldValue::Double(v) => {
							if let Some(ref listener) = self.listener_float_value {
								let object_id = command.object_id;
								listener(member_with_creator.creator, &object_id, command.field_id, v);
							}
						}
						FieldValue::Structure(s) => {
							if let Some(ref listener) = self.listener_structure {
								let object_id = command.object_id;
								listener(member_with_creator.creator, &object_id, command.field_id, &s.as_slice().into());
							}
						}
					},
					S2CCommand::Event(command) => {
						if let Some(ref listener) = self.listener_event {
							listener(member_with_creator.creator, &command.object_id, command.field_id, &command.event);
						}
					}
					S2CCommand::Delete(command) => {
						if let Some(ref listener) = self.listener_delete_object {
							listener(&command.object_id);
						}
					}
					S2CCommand::DeleteField(command) => {
						if let Some(ref listener) = self.listener_delete_field {
							listener(member_with_creator.creator, &command.object_id, command.field_id, command.field_type);
						}
					}
					S2CCommand::Forwarded(command) => {
						if let Some(ref listener) = self.listener_forwarded_command {
							listener((*command).into());
						}
					}
					S2CCommand::MemberConnected(command) => {
						if let Some(ref listener) = self.listener_member_connected {
							listener(command.member_id);
						}
					}
				}
			}
		}
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> Result<GameObjectId, SendError<ClientRequest>> {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, GameObjectOwner::Member(self.member_id));
		self.send(C2SCommand::CreateGameObject(CreateGameObjectCommand {
			object_id: game_object_id,
			template,
			access_groups: AccessGroups(access_group),
		}))?;

		Ok(game_object_id)
	}

	pub fn set_rtt_emulation(&mut self, rtt: Duration, rtt_dispersion: f64) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion))
	}

	pub fn set_drop_emulation(&mut self, drop_probability: f64, drop_time: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client
			.send(ClientRequest::ConfigureDropEmulation(drop_probability, drop_time))
	}

	pub fn reset_emulation(&mut self) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ResetEmulation)
	}

	pub fn attach_to_room(&mut self) -> Result<(), SendError<ClientRequest>> {
		// удаляем все пришедшие команды (ситуация возникает при attach/detach)
		while self.commands_from_server.try_recv().is_ok() {}
		self.send(C2SCommand::AttachToRoom)
	}
}
