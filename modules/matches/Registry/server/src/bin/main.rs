use std::env;

use tonic::transport::Server;
use tonic_health::ServingStatus;

use cheetah_matches_registry::proto::matches::registry::internal::registry_server::RegistryServer;
use cheetah_matches_registry::registry::service::RegistryService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	cheetah_libraries_microservice::init("matches.registry");

	let redis_host = cheetah_libraries_microservice::get_env("REDIS_HOST");
	let redis_port: u16 = cheetah_libraries_microservice::get_env("REDIS_PORT").parse().unwrap();
	let redis_auth = env::var("REDIS_AUTH").ok();
	let redis_dsn = match redis_auth {
		Some(ref password) => {
			format!("redis://:{}@{}:{}", password, redis_host, redis_port)
		}
		None => {
			format!("redis://{}:{}", redis_host, redis_port)
		}
	};

	let registry_service = RegistryService::new(&redis_dsn).await?;
	let grpc_service = RegistryServer::new(registry_service);

	let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
	health_reporter.set_service_status("", ServingStatus::Serving).await;

	Server::builder()
		.add_service(health_service)
		.add_service(grpc_service)
		.serve(cheetah_libraries_microservice::get_internal_grpc_service_default_address())
		.await?;
	Ok(())
}
