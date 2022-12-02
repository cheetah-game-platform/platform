use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;

use fnv::FnvHashSet;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::{RoomsServerManager, RoomsServerManagerError, Server};

///
/// Паттерн Создатель для игрового сервера
/// - если адреса для udp/grpc не заданы - то в качестве адреса выбирается 127.0.0.1, в качестве
/// порта - свободный порт
///
pub struct ServerBuilder {
	game_bind_addr: SocketAddr,
	admin_webgrpc_service_bind_address: SocketAddr,
	internal_grpc_service_bind_address: SocketAddr,
	internal_webgrpc_service_bind_address: SocketAddr,
	is_agones_enabled: bool,
	plugin_names: FnvHashSet<String>,
}

impl Default for ServerBuilder {
	fn default() -> Self {
		Self {
			game_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			admin_webgrpc_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_grpc_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_webgrpc_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			is_agones_enabled: false,
			plugin_names: FnvHashSet::default(),
		}
	}
}

#[derive(Debug, Error)]
pub enum ServerBuilderError {
	#[error("RoomsServerManager {0}")]
	RoomsServerManager(RoomsServerManagerError),
	#[error("ErrorOpenGrpcSocket {0}")]
	ErrorOpenGrpcSocket(io::Error),
	#[error("ErrorBindUdpSocket {0}")]
	ErrorBindUdpSocket(io::Error),
	#[error("ErrorGetLocalAddrFromUdpSocket {0}")]
	ErrorGetLocalAddrFromUdpSocket(io::Error),
}

impl ServerBuilder {
	#[must_use]
	pub fn set_games_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.game_bind_addr = addr;
		self
	}

	#[must_use]
	pub fn set_admin_webgrpc_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.admin_webgrpc_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn set_internal_grpc_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.internal_grpc_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn set_internal_webgrpc_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.internal_webgrpc_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn enable_agones(mut self) -> Self {
		self.is_agones_enabled = true;
		self
	}

	#[must_use]
	pub fn set_plugin_names(mut self, plugin_names: FnvHashSet<String>) -> Self {
		self.plugin_names = plugin_names;
		self
	}

	pub async fn build(self) -> Result<Server, ServerBuilderError> {
		let game_socket = UdpSocket::bind(self.game_bind_addr).map_err(ServerBuilderError::ErrorBindUdpSocket)?;
		let game_socket_addr = game_socket.local_addr().map_err(ServerBuilderError::ErrorGetLocalAddrFromUdpSocket)?;
		let server_manager = RoomsServerManager::new(game_socket, self.plugin_names).map_err(ServerBuilderError::RoomsServerManager)?;
		let manager = Arc::new(Mutex::new(server_manager));

		let internal_grpc_listener = TcpListener::bind(self.internal_grpc_service_bind_address)
			.await
			.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;

		let internal_webgrpc_listener = TcpListener::bind(self.internal_webgrpc_service_bind_address)
			.await
			.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;

		let admin_webgrpc_listener = TcpListener::bind(self.admin_webgrpc_service_bind_address)
			.await
			.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;

		Ok(Server {
			game_socket_addr,
			internal_webgrpc_listener,
			internal_grpc_listener,
			admin_webgrpc_listener,
			is_agones_enabled: self.is_agones_enabled,
			manager,
		})
	}
}
