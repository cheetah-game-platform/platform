use tonic::metadata::MetadataMap;

use crate::grpc::AuthorizationError::*;
use crate::token::SessionTokenError;

#[derive(Debug)]
pub enum AuthorizationError {
    MissingHeader,
    WrongHeader,
    Token(SessionTokenError),
}

pub fn get_player_id(
    metadata: &MetadataMap,
    public_key: String,
) -> Result<u64, AuthorizationError> {
    match metadata.get("authorization") {
        None => Result::Err(MissingHeader),
        Some(value) => {
            let value = value.to_str().unwrap().to_string();
            let splitted: Vec<_> = value.split(" ").collect();
            if splitted.len() != 2 {
                Result::Err(WrongHeader)
            } else {
                let token = splitted.get(1).unwrap().to_string();
                let result = crate::token::JWTTokenParser::new(public_key).get_player_id(token);
                result.map_err(AuthorizationError::Token)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use serde::Deserialize;
    use serde::Serialize;
    use tonic::metadata::{MetadataMap, MetadataValue};

    use crate::grpc::{get_player_id, AuthorizationError};
    use crate::token::SessionTokenError;

    pub const PUBLIC_KEY: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEVVHNXKxoUNkoX9hnOJpSz6K2KDfi
gxaSXu+FIpP32qvcDgZPZ01tjnGjOysyPxRoZaMu/d9rHi3ulbceoYwS+Q==
-----END PUBLIC KEY-----";

    pub const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgcg7dsJWSz8f40cEv
BTeGSzANXGlEzutd9IIm6/inl0ahRANCAARVUc1crGhQ2Shf2Gc4mlLPorYoN+KD
FpJe74Uik/faq9wOBk9nTW2OcaM7KzI/FGhloy7932seLe6Vtx6hjBL5
-----END PRIVATE KEY-----";

    #[test]
    fn should_missing_header() {
        let metadata = MetadataMap::new();
        assert!(matches!(
            get_player_id(&metadata, PUBLIC_KEY.to_string()),
            Result::Err(AuthorizationError::MissingHeader)
        ));
    }

    #[test]
    fn should_wrong_header() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "authorization",
            MetadataValue::from_str("wrong_authorization").unwrap(),
        );
        assert!(matches!(
            get_player_id(&metadata, PUBLIC_KEY.to_string()),
            Result::Err(AuthorizationError::WrongHeader)
        ));
    }

    #[test]
    fn should_wrong_token() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "authorization",
            MetadataValue::from_str("Bear eyJlbWFpbCI6ImFsZXhAa3ZpcmluZy5jb20iLCJhdWQiOiJzb21lLWNsaWVudC1pZCJ9.AcUzPLaDRYUZfpH5Q4xlC_xH9rwi_YefKwJT080dRyYgwPtaHYjygGjC2djhhvs1YjlQS59qf9NG5h_7qpk3_r1-S-UNIBMuB1Tkqu1YSJF1N2H6AuSkA4TQ4YE5mNHL3pudaD5vplfQa5KOqL1fgxekTQ2Rnkq90YuW_Xck0RgPqTDkso0kvHZcS5t5qyX_Rg2EieE6i73nZL3-B15BwRKl6NmaJZ1dTLn9IYpuM_TeapMmrQcqCIesqV4N9MlCbhawkKtbiaolTXET-ujFeDWnR3XFbxi_DxSMQ-Dwq1gGRzcxWK6xIrGqh02TVC2HUPvLefgRu9Mmky6igcYWJw").unwrap(),
        );

        assert!(matches!(
            get_player_id(&metadata, PUBLIC_KEY.to_string()),
            Result::Err(AuthorizationError::Token(
                SessionTokenError::InvalidSignature
            ))
        ));
    }

    #[derive(Serialize, Deserialize)]
    struct TokenClaims {
        pub exp: usize,
        pub player: u64,
    }

    #[test]
    fn should_correct_token() {
        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::ES256),
            &TokenClaims {
                exp: usize::max_value(),
                player: 10,
            },
            &EncodingKey::from_ec_pem(PRIVATE_KEY.as_bytes()).unwrap(),
        )
        .unwrap();
        let parts: Vec<_> = token.split(".").collect();
        let token = format!("{}.{}", parts.get(1).unwrap(), parts.get(2).unwrap());

        let mut metadata = MetadataMap::new();
        metadata.insert(
            "authorization",
            MetadataValue::from_str(format!("Bear {}", token).as_str()).unwrap(),
        );
        println!("{:?}", get_player_id(&metadata, PUBLIC_KEY.to_string()));
        assert!(matches!(
            get_player_id(&metadata, PUBLIC_KEY.to_string()),
            Result::Ok(_)
        ));
    }
}