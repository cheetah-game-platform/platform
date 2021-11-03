use tonic::transport::Server;
use tonic::{Request, Response, Status};

use cheetah_matches_registry::registry::RegistryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_microservice::init("matches.registry");

	let registry_service = RegistryService {};

	let grpc_service = cheetah_matches_registry::proto::internal::registry_server::RegistryServer::new(registry_service);
	Server::builder()
		.add_service(grpc_service)
		.serve(cheetah_microservice::get_internal_service_binding_addr())
		.await
		.unwrap();

	Result::Ok(())
}
