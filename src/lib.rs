use std::sync::Arc;

use async_graphql::{http::GraphiQLSource, EmptySubscription, OutputType, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use community::{ExpressionPost, ExpressionPostAggregate, Reply, UserProfile};
use db::DbController;
use tower_cookies::Cookies;

mod auth;
mod community;
mod db;

pub struct ApplicationState {
    pub auth_schema: Schema<auth::Query, auth::Mutation, EmptySubscription>,
    pub community_schema: Schema<community::Query, community::Mutation, EmptySubscription>,
}

impl ApplicationState {
    pub async fn init() -> Arc<ApplicationState> {
        let db = Arc::new(
            DbController::init()
                .await
                .expect("Error initializing database"),
        );

        let auth_schema = Schema::build(auth::Query, auth::Mutation, EmptySubscription)
            .data(Arc::clone(&db))
            .finish();
        let community_schema =
            Schema::build(community::Query, community::Mutation, EmptySubscription)
                .data(Arc::clone(&db))
                .finish();

        Arc::new(ApplicationState {
            auth_schema,
            community_schema,
        })
    }
}

pub async fn auth_gateway(
    cookies: Cookies,
    State(state): State<Arc<ApplicationState>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    req = req.data(cookies);
    state.auth_schema.execute(req).await.into()
}

pub async fn auth_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/auth").finish())
}

pub async fn community_gateway(
    cookies: Cookies,
    State(state): State<Arc<ApplicationState>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    req = req.data(cookies);
    state.community_schema.execute(req).await.into()
}

pub async fn community_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/community").finish())
}

pub fn welcome() {
    println!("SPADE Mental Health API!");
}

#[derive(SimpleObject)]
#[graphql(concrete(name = "UserProfileResponse", params(UserProfile)))]
#[graphql(concrete(name = "ExpressionPostResponse", params(ExpressionPost)))]
#[graphql(concrete(
    name = "ExpressionPostAggregateResponse",
    params(ExpressionPostAggregate)
))]
#[graphql(concrete(name = "ReplyResponse", params(Reply)))]
pub struct GatewayResponse<T: OutputType> {
    pub success: bool,
    pub message: Option<String>,
    pub payload: Option<T>,
    pub status_code: u16,
}

impl<T: OutputType> GatewayResponse<T> {
    pub fn new(
        success: bool,
        message: Option<String>,
        payload: Option<T>,
        status_code: u16,
    ) -> Self {
        Self {
            success,
            message,
            payload,
            status_code,
        }
    }
}
