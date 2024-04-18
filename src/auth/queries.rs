use async_graphql::Object;

pub struct Query;

#[Object]
impl Query {
    // Placeholder query
    async fn add(&self, a: u32, b: u32) -> u32 {
        a + b
    }
}