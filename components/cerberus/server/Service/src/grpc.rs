use crate::proto;
use crate::storage::RedisRefreshTokenStorage;
use crate::token::JWTTokensService;
use tonic::Request;

pub struct Cerberus {
    service: JWTTokensService,
}

impl Cerberus {
    pub fn new(
        private_key: String,
        public_key: String,
        redis_host: String,
        redis_port: u16,
    ) -> Self {
        let storage =
            RedisRefreshTokenStorage::new(redis_host, redis_port, 31 * 24 * 60 * 60).unwrap();
        Self {
            service: JWTTokensService::new(
                private_key,
                public_key,
                5 * 60 * 60,
                30 * 24 * 60 * 60,
                storage,
            ),
        }
    }
}

#[tonic::async_trait]
impl proto::internal::cerberus_server::Cerberus for Cerberus {
    async fn create(
        &self,
        request: Request<proto::internal::CreateTokenRequest>,
    ) -> Result<tonic::Response<proto::types::TokensReply>, tonic::Status> {
        let request = request.get_ref();
        match self
            .service
            .create(request.user_id.clone(), request.device_id.clone())
            .await
        {
            Ok(tokens) => Result::Ok(tonic::Response::new(proto::types::TokensReply {
                session: tokens.session,
                refresh: tokens.refresh,
            })),
            Err(e) => Result::Err(tonic::Status::failed_precondition(format!("{:?}", e))),
        }
    }
}

#[tonic::async_trait]
impl proto::external::cerberus_server::Cerberus for Cerberus {
    async fn refresh(
        &self,
        request: tonic::Request<proto::external::RefreshTokenRequest>,
    ) -> Result<tonic::Response<proto::types::TokensReply>, tonic::Status> {
        let request = request.get_ref();

        match self.service.refresh(request.token.clone()).await {
            Ok(tokens) => Result::Ok(tonic::Response::new(proto::types::TokensReply {
                session: tokens.session,
                refresh: tokens.refresh,
            })),
            Err(e) => Result::Err(tonic::Status::failed_precondition(format!("{:?}", e))),
        }
    }
}