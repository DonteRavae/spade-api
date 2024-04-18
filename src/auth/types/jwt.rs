use std::error::Error;

use chrono::{Days, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(id: &str) -> Result<Self, Box<dyn Error + Sync + Send>> {
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
        Ok(Self(token))
    }

    pub fn decode(encoded_token: &str) -> Result<AccessTokenClaims, Box<dyn Error + Sync + Send>> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_audience(&["spadementalhealth.com"]);
        Ok(decode::<AccessTokenClaims>(
            encoded_token,
            &DecodingKey::from_secret(dotenv::var("ACCESS_TOKEN_SECRET")?.as_ref()),
            &validation,
        )?
        .claims)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct RefreshToken(String);

impl RefreshToken {
    pub fn new(id: &str) -> Result<Self, Box<dyn Error + Sync + Send>> {
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

        Ok(Self(token))
    }

    pub fn decode(encoded_token: &str) -> Result<RefreshTokenClaims, Box<dyn Error + Sync + Send>> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_audience(&["spadementalhealth.com"]);
        Ok(decode::<RefreshTokenClaims>(
            encoded_token,
            &DecodingKey::from_secret(dotenv::var("REFRESH_TOKEN_SECRET")?.as_ref()),
            &validation,
        )?
        .claims)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
