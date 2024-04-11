use std::error::Error;

use chrono::{Days, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    pub sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    pub sub: String,
}

pub struct JwtManager;

impl JwtManager {
    pub fn new_access_token(id: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        let claims: AccessTokenClaims = AccessTokenClaims {
            aud: String::from("spadementalhealth.com"),
            exp: Local::now()
                .checked_add_days(Days::new(1))
                .unwrap()
                .timestamp() as usize,
            iat: chrono::offset::Utc::now().timestamp() as usize,
            iss: String::from("auth.spadementalhealth.com"),
            sub: id.to_string(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(dotenv::var("ACCESS_TOKEN_SECRET")?.as_ref()),
        )?;
        Ok(token)
    }
    pub fn new_refresh_token(id: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        let claims: RefreshTokenClaims = RefreshTokenClaims {
            aud: String::from("spadementalhealth.com"),
            exp: Local::now()
                .checked_add_days(Days::new(14))
                .unwrap()
                .timestamp() as usize,
            iat: chrono::offset::Utc::now().timestamp() as usize,
            iss: String::from("auth.spadementalhealth.com"),
            sub: id.to_string(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(dotenv::var("REFRESH_TOKEN_SECRET")?.as_ref()),
        )?;
        Ok(token)
    }

    pub fn decode_access_token(
        encoded_token: &str,
    ) -> Result<AccessTokenClaims, Box<dyn Error + Sync + Send>> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_audience(&["spadementalhealth.com"]);
        Ok(decode::<AccessTokenClaims>(
            encoded_token,
            &DecodingKey::from_secret(dotenv::var("ACCESS_TOKEN_SECRET")?.as_ref()),
            &validation,
        )?
        .claims)
    }

    pub fn decode_refresh_token(
        encoded_token: &str,
    ) -> Result<RefreshTokenClaims, Box<dyn Error + Sync + Send>> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_audience(&["spadementalhealth.com"]);
        Ok(decode::<RefreshTokenClaims>(
            encoded_token,
            &DecodingKey::from_secret(dotenv::var("REFRESH_TOKEN_SECRET")?.as_ref()),
            &validation,
        )?
        .claims)
    }
}
