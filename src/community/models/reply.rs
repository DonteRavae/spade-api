use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user_profile::UserProfile;

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
pub struct Reply {
    id: String,
    author: UserProfile,
    parent: String, // Identification of parent object
    content: String,
    created_at: DateTime<Utc>,
    last_modified: DateTime<Utc>,
}

impl Reply {
    pub fn new(
        id: String,
        author: UserProfile,
        parent: String,
        content: String,
        created_at: DateTime<Utc>,
        last_modified: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            author,
            parent,
            content,
            created_at,
            last_modified,
        }
    }
}

/********** REQUEST OBJECT **********/

#[derive(Debug, InputObject)]
pub struct NewReplyRequest {
    pub content: String,
    pub parent: String,
}
