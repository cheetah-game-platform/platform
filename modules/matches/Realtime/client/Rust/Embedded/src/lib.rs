use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use cheetah_matches_realtime::builder::ServerBuilder;
use cheetah_matches_realtime::server::manager::RoomsServerManager;

mod ffi;

///
/// Обертка для запуска сервера из so/dll.
/// - методы не могут быть async так как они будут вызываться как методы so/dll
///
pub struct EmbeddedServerWrapper {
	runtime: Runtime,
	pub manager: Arc<Mutex<RoomsServerManager>>,
	pub game_socket_addr: SocketAddr,
	pub internal_grpc_socket_addr: SocketAddr,
	pub admin_grpc_socket_addr: SocketAddr,
}

#[derive(thiserror::Error, Debug)]
pub enum EmbeddedServerWrapperError {
	#[error("GrpcServicesNotStarted")]
	GrpcServicesNotStarted,
}

impl EmbeddedServerWrapper {
	pub fn run_new_server(bind_address: [u8; 4]) -> anyhow::Result<Self> {
		let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_io().build()?;

		let bind_socket_address = SocketAddr::new(IpAddr::from(bind_address), 0);

		let server = runtime.block_on(async move {
			ServerBuilder::default()
				.set_internal_grpc_address(bind_socket_address)
				.set_admin_grpc_address(bind_socket_address)
				.set_game_address(bind_socket_address)
				.build()
				.await
		})?;
		let manager = Arc::clone(&server.manager);
		let game_socket_addr = server.game_socket_addr;
		let internal_grpc_socket_addr = server.internal_grpc_tcp_listener.local_addr()?;
		let admin_grpc_socket_addr = server.admin_grpc_tcp_listener.local_addr()?;
		runtime.spawn(async move {
			server.run().await;
		});

		Self::wait_open_grpc_ports(internal_grpc_socket_addr, admin_grpc_socket_addr)?;

		Ok(EmbeddedServerWrapper {
			runtime,
			manager,
			game_socket_addr,
			internal_grpc_socket_addr,
			admin_grpc_socket_addr,
		})
	}

	fn wait_open_grpc_ports(internal_grpc_socket_addr: SocketAddr, admin_grpc_socket_addr: SocketAddr) -> Result<(), EmbeddedServerWrapperError> {
		let mut counter = 0;
		while !port_scanner::scan_port_addr(internal_grpc_socket_addr) || !port_scanner::scan_port_addr(admin_grpc_socket_addr) {
			std::thread::sleep(Duration::from_millis(10));
			counter += 1;
			if counter > 100 {
				return Err(EmbeddedServerWrapperError::GrpcServicesNotStarted);
			}
		}
		Ok(())
	}

	pub fn shutdown(self) {
		let manager = Arc::clone(&self.manager);
		self.runtime.block_on(async move { manager.lock().await.shutdown() });
		self.runtime.shutdown_background();
	}
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	use crate::EmbeddedServerWrapper;

	#[test]
	fn should_open_tcp_ports_after_start() {
		let server = EmbeddedServerWrapper::run_new_server(Default::default()).unwrap();
		let admin_grpc_port = server.admin_grpc_socket_addr.port();
		let internal_grpc_port = server.internal_grpc_socket_addr.port();
		assert!(port_scanner::scan_port(admin_grpc_port));
		assert!(port_scanner::scan_port(internal_grpc_port));
	}

	#[test]
	fn should_use_different_port_for_different_server() {
		let server_a = EmbeddedServerWrapper::run_new_server(Default::default()).unwrap();
		let server_b = EmbeddedServerWrapper::run_new_server(Default::default()).unwrap();
		assert_ne!(server_a.game_socket_addr, server_b.game_socket_addr);
		assert_ne!(server_a.admin_grpc_socket_addr, server_b.admin_grpc_socket_addr);
		assert_ne!(server_a.internal_grpc_socket_addr, server_b.internal_grpc_socket_addr);
	}

	#[test]
	fn should_shutdown_server() {
		let server = EmbeddedServerWrapper::run_new_server(Default::default()).unwrap();
		let admin_grpc_port = server.admin_grpc_socket_addr.port();
		let internal_grpc_port = server.internal_grpc_socket_addr.port();
		server.shutdown();
		std::thread::sleep(Duration::from_millis(100));
		assert!(!port_scanner::scan_port(admin_grpc_port));
		assert!(!port_scanner::scan_port(internal_grpc_port));
	}
}
