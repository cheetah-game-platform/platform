use std::net::SocketAddr;
use std::sync::Arc;

use futures::join;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic_health::ServingStatus;

use grpc::proto::internal::realtime_server::RealtimeServer;

use crate::agones::run_agones_sdk;
use crate::debug::dump::DumpGrpcService;
use crate::debug::grpc::RealtimeAdminGRPCService;
use crate::debug::proto::admin;
use crate::debug::tracer::grpc::CommandTracerGRPCService;
use crate::grpc::RealtimeInternalService;
use crate::server::manager::{RoomsServerManager, RoomsServerManagerError};

pub mod agones;
pub mod builder;
pub mod debug;
pub mod grpc;
pub mod room;
pub mod server;

///
/// Server = Agones + Grpc + `NetworkRoomsServerManager`
/// `NetworkRoomsServerManager` = Manager + `NetworkRoomsServer`
///
pub struct Server {
	pub game_socket_addr: SocketAddr,
	pub internal_webgrpc_listener: TcpListener,
	pub internal_grpc_listener: TcpListener,
	pub admin_webgrpc_listener: TcpListener,
	pub is_agones_enabled: bool,
	pub manager: Arc<Mutex<RoomsServerManager>>,
}

impl Server {
	pub async fn run(self) {
		let internal_grpc_future = Self::new_internal_grpc_service(self.internal_grpc_listener, self.manager.clone());
		let internal_webgrpc_future = Self::new_internal_webgrpc_service(self.internal_webgrpc_listener, self.manager.clone());
		let admin_grpc = Self::configure_admin_grpc_service(self.admin_webgrpc_listener, self.manager.clone());
		if self.is_agones_enabled {
			let agones = run_agones_sdk(self.manager.clone());
			join!(internal_grpc_future, internal_webgrpc_future, admin_grpc, agones);
		} else {
			join!(internal_grpc_future, internal_webgrpc_future, admin_grpc);
		}
	}

	async fn new_internal_grpc_service(listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let service = RealtimeServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		tonic::transport::server::Server::builder()
			.add_service(health_service.clone())
			.add_service(service.clone())
			.serve_with_incoming(TcpListenerStream::new(listener))
			.await
			.unwrap();
	}

	async fn new_internal_webgrpc_service(listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let service = RealtimeServer::new(RealtimeInternalService::new(manager));

		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		tonic::transport::server::Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(service))
			.serve_with_incoming(TcpListenerStream::new(listener))
			.await
			.unwrap();
	}

	async fn configure_admin_grpc_service(tcp_listener: TcpListener, manager: Arc<Mutex<RoomsServerManager>>) {
		let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
		health_reporter.set_service_status("", ServingStatus::Serving).await;

		let admin = admin::realtime_server::RealtimeServer::new(RealtimeAdminGRPCService::new(manager.clone()));
		let tracer = admin::command_tracer_server::CommandTracerServer::new(CommandTracerGRPCService::new(manager.clone()));
		let dumper = admin::dump_server::DumpServer::new(DumpGrpcService::new(manager));

		tonic::transport::Server::builder()
			.accept_http1(true)
			.add_service(tonic_web::enable(health_service))
			.add_service(tonic_web::enable(dumper))
			.add_service(tonic_web::enable(admin))
			.add_service(tonic_web::enable(tracer))
			.serve_with_incoming(TcpListenerStream::new(tcp_listener))
			.await
			.unwrap();
	}
}
