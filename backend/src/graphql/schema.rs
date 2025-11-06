use async_graphql::{Context, EmptySubscription, Object, Schema};
use conhub_models::auth::Claims;

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> String {
        "healthy".to_string()
    }

    async fn version(&self) -> String {
        // Keep simple; can be wired to config later
        "v1".to_string()
    }

    async fn me(&self, ctx: &Context<'_>) -> Option<CurrentUser> {
        let claims = ctx.data_opt::<Claims>()?;
        Some(CurrentUser::from(claims.clone()))
    }
}

#[derive(async_graphql::SimpleObject, Default)]
pub struct CurrentUser {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
}

impl From<Claims> for CurrentUser {
    fn from(c: Claims) -> Self {
        Self {
            user_id: Some(c.sub.to_string()),
            roles: c.roles,
        }
    }
}

pub type ConhubSchema = Schema<QueryRoot, async_graphql::EmptyMutation, EmptySubscription>;

pub fn build_schema() -> ConhubSchema {
    Schema::build(QueryRoot::default(), async_graphql::EmptyMutation, EmptySubscription)
        .finish()
}