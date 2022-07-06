use tonic::Status;

use cheetah_libraries_microservice::trace::ResultErrorTracer;
use proto::RefreshTokenRequest;

use crate::proto;
use crate::proto::SessionAndRefreshTokens;
use crate::tokens::TokensService;

pub struct TokensGrpcService {
	service: TokensService,
}

impl TokensGrpcService {
	pub fn new(service: TokensService) -> Self {
		Self { service }
	}
}
#[tonic::async_trait]
impl proto::tokens_server::Tokens for TokensGrpcService {
	async fn refresh(
		&self,
		request: tonic::Request<RefreshTokenRequest>,
	) -> Result<tonic::Response<SessionAndRefreshTokens>, Status> {
		let request = request.get_ref();

		let tokens = self
			.service
			.refresh(request.token.clone())
			.await
			.trace_and_map_msg(format!("Refresh jwt tokens {}", request.token), |_| {
				Status::internal("")
			})?;

		Ok(tonic::Response::new(SessionAndRefreshTokens {
			session: tokens.session,
			refresh: tokens.refresh,
		}))
	}
}
