use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Instant;

use crate::commands::hash::{UserPrivateKey, UserPublicKey};
use crate::udp::channel::Channel;
use crate::udp::protocol::codec::cipher::Cipher;
use crate::udp::protocol::codec::decoder::UdpFrameDecodeError;
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::relay::RelayProtocol;

pub struct UdpServer<PeerAddress> {
	pub channel: Rc<RefCell<dyn Channel<PeerAddress>>>,
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
	pub fn new(channel: Rc<RefCell<dyn Channel<PeerAddress>>>) -> UdpServer<PeerAddress> {
		UdpServer {
			channel,
			sessions: Default::default(),
		}
	}
	
	pub fn cycle(&mut self, now: &Instant) {
		self.do_read(now);
		self.do_write(now)
	}
	
	fn do_write(&mut self, now: &Instant) {
		let channel = &self.channel.clone();
		self.sessions.iter_mut().for_each(|(_, session)| {
			match session.address {
				None => {}
				Some(ref address) => {
					match session.protocol.build_next_frame(now) {
						None => {}
						Some(mut frame) => {
							let (binary, commands) = frame.encode(&mut Cipher::new(&session.private_key));
							session.protocol.out_commands_collector.add_unsent_commands(commands);
							let address = session.address.as_ref().unwrap();
							channel.borrow_mut().send(address, binary);
						}
					}
				}
			}
		})
	}
	
	fn do_read(&mut self, now: &Instant) {
		loop {
			
			// read
			let channel = self.channel.clone();
			let channel = channel.borrow();
			match channel.try_recv() {
				None => { break; }
				Some((address, data)) => {
					let mut cursor = Cursor::new(data.as_slice());
					let headers = Frame::decode_headers(&mut cursor);
					match headers {
						Ok((header, additional_headers)) => {
							let user_public_key_header: Option<&UserPublicKey> = additional_headers.first(Header::predicate_UserPublicKey);
							match user_public_key_header {
								None => {
									log::error!("public key header not found, peer address {:?}", address);
								}
								Some(public_key) => {
									match self.sessions.get_mut(public_key) {
										None => {
											log::error!("user session not found, peer address {:?}", address);
										}
										Some(session) => {
											match Frame::decode_frame(cursor, Cipher::new(&session.private_key), header, additional_headers) {
												Ok(frame) => {
													session.protocol.on_frame_received(frame, now);
													session.address = Option::Some(address);
												}
												Err(e) => {
													log::error!("error decode frame {:?}", e)
												}
											}
										}
									}
								}
							}
						}
						Err(e) => {
							log::error!("protocol skip by header {:?}", e)
						}
					}
				}
			}
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