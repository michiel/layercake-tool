use async_graphql::*;

use crate::graphql::queries::Query;
use crate::graphql::mutations::Mutation;
use crate::graphql::subscriptions::Subscription;

pub type GraphQLSchema = Schema<Query, Mutation, Subscription>;

pub fn build_schema() -> GraphQLSchema {
    Schema::build(Query, Mutation, Subscription)
        .finish()
}