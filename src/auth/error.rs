use std::{
    error::Error,
    fmt::{self, Display},
};

use async_graphql::ErrorExtensions;

#[derive(Debug)]
pub enum AuthError {
    VerificationError(String),
    DuplicateUser(String),
    Unauthorized(String),
    ServerError(String),
    BadRequest(String),
    Forbidden,
}

impl Error for AuthError {}

impl Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Authentication or Authorization Error")
    }
}

impl ErrorExtensions for AuthError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| match self {
            AuthError::VerificationError(msg) => e.set("reason", msg.clone()),
            AuthError::DuplicateUser(msg) => e.set("reason", msg.clone()),
            AuthError::Unauthorized(msg) => e.set("reason", msg.clone()),
            AuthError::ServerError(msg) => e.set("reason", msg.clone()),
            AuthError::BadRequest(msg) => e.set("reason", msg.clone()),
            AuthError::Forbidden => e.set("code", 403),
        })
    }
}
