use std::{
    error::Error,
    fmt::{self, Display},
};

use async_graphql::ErrorExtensions;

#[derive(Debug)]
pub enum CommunityError {
    DuplicateProfile(String),
    Unauthorized,
}

impl Error for CommunityError {}

impl Display for CommunityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Community Error")
    }
}

impl ErrorExtensions for CommunityError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| match self {
            CommunityError::DuplicateProfile(msg) => e.set("reason", msg.clone()),
            CommunityError::Unauthorized => e.set("code", 401),
        })
    }
}
