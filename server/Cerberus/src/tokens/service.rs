use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use cheetah_microservice::jwt::{JWTTokenParser, SessionTokenClaims};

use super::storage::RedisRefreshTokenStorage;

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
	exp: usize,
	player: u64,
	device_id: String,
	version: u64,
}

#[derive(Debug)]
pub struct Tokens {
	pub session: String,
	pub refresh: String,
}

#[derive(Debug)]
pub enum JWTTokensServiceError {
	InvalidSignature,
	Expired,
	InvalidId,
	StorageError(String),
}

pub struct JWTTokensService {
	session_exp_in_sec: u64,
	refresh_exp_in_sec: u64,
	private_key: String,
	public_key: String,
	storage: RedisRefreshTokenStorage,
}

impl JWTTokensService {
	pub fn new(
		private_key: String,
		public_key: String,
		session_exp_in_sec: u64,
		refresh_exp_in_sec: u64,
		storage: RedisRefreshTokenStorage,
	) -> Self {
		Self {
			session_exp_in_sec,
			refresh_exp_in_sec,
			private_key,
			public_key,
			storage,
		}
	}

	pub async fn create(&self, player: u64, device_id: String) -> Result<Tokens, JWTTokensServiceError> {
		Result::Ok(Tokens {
			session: self.create_session_token(player),
			refresh: self.create_refresh_token(player, device_id).await?,
		})
	}

	async fn create_refresh_token(&self, player: u64, device_id: String) -> Result<String, JWTTokensServiceError> {
		let version = self
			.storage
			.new_version(player, &device_id)
			.await
			.map_err(|e| JWTTokensServiceError::StorageError(format!("{:?}", e)))?;

		let timestamp = JWTTokensService::get_time_stamp();
		let claims = RefreshTokenClaims {
			exp: (timestamp + self.refresh_exp_in_sec) as usize,
			player,
			device_id,
			version,
		};
		let token = encode(
			&Header::new(Algorithm::ES256),
			&claims,
			&EncodingKey::from_ec_pem(self.private_key.as_bytes()).unwrap(),
		)
		.unwrap();
		Result::Ok(JWTTokensService::remove_head(token))
	}

	fn create_session_token(&self, player: u64) -> String {
		let timestamp = JWTTokensService::get_time_stamp();
		let claims = SessionTokenClaims {
			exp: (timestamp + self.session_exp_in_sec) as usize,
			player,
		};

		let token = encode(
			&Header::new(Algorithm::ES256),
			&claims,
			&EncodingKey::from_ec_pem(self.private_key.as_bytes()).unwrap(),
		)
		.unwrap();
		JWTTokensService::remove_head(token)
	}

	fn get_time_stamp() -> u64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards")
			.as_secs()
	}

	fn remove_head(token: String) -> String {
		let collect: Vec<_> = token.split('.').collect();
		format!("{}.{}", collect[1], collect[2])
	}

	pub async fn refresh(&self, refresh_token: String) -> Result<Tokens, JWTTokensServiceError> {
		let token = JWTTokenParser::add_head(refresh_token);
		match jsonwebtoken::decode::<RefreshTokenClaims>(
			token.as_str(),
			&DecodingKey::from_ec_pem(self.public_key.as_bytes()).unwrap(),
			&Validation::new(Algorithm::ES256),
		) {
			Ok(token) => {
				let player = token.claims.player;
				let device_id = token.claims.device_id;
				if self.storage.get_version(player, &device_id).await.unwrap() == token.claims.version {
					Result::Ok(Tokens {
						session: self.create_session_token(player),
						refresh: self.create_refresh_token(player, device_id).await?,
					})
				} else {
					Result::Err(JWTTokensServiceError::InvalidId)
				}
			}
			Err(error) => match error.kind() {
				ErrorKind::ExpiredSignature => Result::Err(JWTTokensServiceError::Expired),
				_ => Result::Err(JWTTokensServiceError::InvalidSignature),
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use std::thread;
	use std::time::Duration;

	use testcontainers::clients::Cli;
	use testcontainers::images::redis::Redis;
	use testcontainers::{images, Container, Docker};

	use cheetah_microservice::jwt::{JWTTokenParser, SessionTokenError};

	use crate::tokens::service::{JWTTokensService, JWTTokensServiceError};
	use crate::tokens::storage::RedisRefreshTokenStorage;

	#[tokio::test]
	async fn session_token_should_correct() {
		let (_node, service) = stub_token_service(1, 1);
		let player = 123;
		let tokens = service.create(player, "some-device-id".to_string()).await.unwrap();

		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = parser.get_player_id(tokens.session);

		assert!(matches!(user_id_from_token, Result::Ok(value) if value==player))
	}

	#[tokio::test]
	async fn session_token_should_exp() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		thread::sleep(Duration::from_secs(2));
		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = parser.get_player_id(tokens.session);
		assert!(matches!(user_id_from_token, Result::Err(SessionTokenError::Expired)))
	}

	#[tokio::test]
	async fn session_token_should_fail_if_not_correct() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
		let user_id_from_token = parser.get_player_id(tokens.session.replace("IzfQ", "ccoY"));
		assert!(matches!(user_id_from_token, Result::Err(SessionTokenError::InvalidSignature)))
	}

	pub const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

	pub const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";

	pub fn stub_token_service<'a>(session_exp: u64, refresh_exp: u64) -> (Container<'a, Cli, Redis>, JWTTokensService) {
		let (node, storage) = stub_storage(refresh_exp + 1);

		let service = JWTTokensService::new(
			PRIVATE_KEY.to_string(),
			PUBLIC_KEY.to_string(),
			session_exp,
			refresh_exp,
			storage,
		);
		(node, service)
	}

	lazy_static::lazy_static! {
		static ref CLI: Cli = Default::default();

	}
	fn stub_storage<'a>(time_of_life_in_sec: u64) -> (Container<'a, Cli, Redis>, RedisRefreshTokenStorage) {
		let node = (*CLI).run(images::redis::Redis::default());
		let port = node.get_host_port(6379).unwrap();
		(
			node,
			RedisRefreshTokenStorage::new("127.0.0.1".to_owned(), port, Option::None, time_of_life_in_sec).unwrap(),
		)
	}

	#[tokio::test]
	async fn should_refresh_token_different_for_players() {
		let (_node, service) = stub_token_service(1, 100);
		let tokens_for_player_a = service.create(123u64, "some-devicea-id".to_string()).await.unwrap();
		let tokens_for_player_b = service.create(124_u64, "some-deviceb-id".to_string()).await.unwrap();
		assert_ne!(tokens_for_player_a.refresh, tokens_for_player_b.refresh)
	}

	#[tokio::test]
	async fn should_refresh_token() {
		let (_node, service) = stub_token_service(1, 100);

		let tokens = service.create(123u64, "some-device-id".to_owned()).await.unwrap();

		let new_tokens = service.refresh(tokens.refresh.clone()).await.unwrap();
		// проверяем что это действительно новые токены
		assert_ne!(tokens.session, new_tokens.session);
		assert_ne!(tokens.refresh, new_tokens.refresh);
		// проверяем работоспособность новых токенов
		let get_player_id_result = JWTTokenParser::new(PUBLIC_KEY.to_owned()).get_player_id(new_tokens.session);
		assert!(matches!(get_player_id_result, Result::Ok(player) if player==123));

		// проверяем что новый refresh токен валидный
		service.refresh(new_tokens.refresh.clone()).await.unwrap();
	}

	///
	/// Проверяем время жизни refresh токена
	///
	#[tokio::test]
	async fn should_refresh_token_exp() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		thread::sleep(Duration::from_secs(2));
		let result = service.refresh(tokens.refresh).await;
		assert!(matches!(result, Result::Err(JWTTokensServiceError::Expired)));
	}

	///
	/// Проверяем реакцию на нарушение подписи
	///
	#[tokio::test]
	async fn should_refresh_token_fail() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		assert!(matches!(
			service.refresh(tokens.refresh.replace("eyJleHA", "eyJleHB")).await,
			Result::Err(JWTTokensServiceError::InvalidSignature)
		));
	}

	///
	/// Проверяем что refresh токен может быть использован один раз
	///
	#[tokio::test]
	async fn should_refresh_token_can_use_once() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		service.refresh(tokens.refresh.clone()).await.unwrap();
		assert!(matches!(
			service.refresh(tokens.refresh).await,
			Result::Err(JWTTokensServiceError::InvalidId)
		));
	}

	///
	/// Проверяем что выдача нового refresh токена инвалидирует старые
	///
	#[tokio::test]
	async fn should_refresh_token_can_invalidate_tokens() {
		let (_node, service) = stub_token_service(1, 1);
		let tokens_a = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		let tokens_b = service.create(123u64, "some-device-id".to_string()).await.unwrap();
		service.refresh(tokens_b.refresh.clone()).await.unwrap();
		assert!(matches!(
			service.refresh(tokens_a.refresh).await,
			Result::Err(JWTTokensServiceError::InvalidId)
		));
	}
}
