use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Cursor;
use std::time::Instant;

use crate::commands::hash::{UserPrivateKey, UserPublicKey};
use crate::udp::channel::Channel;
use crate::udp::protocol::codec::cipher::Cipher;
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::relay::RelayProtocol;

pub struct UdpServer<PeerAddress> {
	pub channel: Box<dyn Channel<PeerAddress>>,
	pub sessions: HashMap<UserPublicKey, UserSession<PeerAddress>>,
}

pub struct UserSession<PeerAddress> {
	pub public_key: UserPublicKey,
	pub private_key: UserPrivateKey,
	pub protocol: RelayProtocol,
	pub address: Option<PeerAddress>,
	pub state: UserSessionState,
}

pub enum UserSessionState {
	CONNECTING,
	CONNECTED,
}

impl<PeerAddress: Hash + Debug> UdpServer<PeerAddress> {
	pub fn new(channel: Box<dyn Channel<PeerAddress>>) -> UdpServer<PeerAddress> {
		UdpServer {
			channel,
			sessions: Default::default(),
		}
	}
	
	pub fn cycle(&mut self, now: Instant) {
		loop {
			// read
			match self.channel.try_recv() {
				None => { break; }
				Some((address, data)) => {
					let mut cursor = Cursor::new(data.as_slice());
					let headers = Frame::decode_headers(&mut cursor);
					match headers {
						Ok((header, additional_headers)) => {
							let session_found = false;
							// for i in 0..additional_headers.len() {
							// 	if let AdditionalUdpHeader::UserPublicKeyC2S(public_key) = additional_headers.get(i).unwrap() {
							// 		let session = self.sessions.get_mut(public_key).unwrap();
							// 		let protocol = UdpFrame::decode_frame(cursor, Cipher::new(&session.private_key), header, additional_headers);
							// 		match protocol {
							// 			Ok(protocol) => {
							// 				println!("server recv {:?}", protocol);
							// 				session.handlers.on_frame_recv(protocol);
							// 				session.address = Option::Some(address)
							// 			}
							// 			Err(e) => {
							// 				log::error!("protocol skip {:?}", e)
							// 			}
							// 		}
							// 		session_found = true;
							// 		break;
							// 	}
							// }
							if !session_found {
								log::error!("protocol skip - session not found")
							}
						}
						Err(e) => {
							log::error!("protocol skip by header {:?}", e)
						}
					}
				}
			}
			
			
			// write
			let channel = &self.channel;
			self.sessions.iter_mut().for_each(|(_, session)| {
				let mut frame = session.protocol.build_next_frame(&now).unwrap();
				let (binary, commands) = frame.encode(&mut Cipher::new(&session.private_key));
				session.protocol.out_commands_collector.add_unsent_commands(commands);
				let address = session.address.as_ref().unwrap();
				channel.send(address, binary);
			})
			
			// client timeout(?)
		}
	}
	
	
	pub fn get_user_sessions<'a>(&'a mut self, public_key: &UserPublicKey) -> &'a mut UserSession<PeerAddress> {
		self.sessions.get_mut(public_key).unwrap()
	}
	
	
	pub fn add_allowed_user(&mut self, private_key: UserPrivateKey, public_key: UserPublicKey) {
		self.sessions.insert(public_key, UserSession {
			public_key,
			private_key,
			protocol: RelayProtocol::new(),
			address: Option::None,
			state: UserSessionState::CONNECTING,
		});
	}
}