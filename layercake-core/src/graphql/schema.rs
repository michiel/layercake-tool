use async_graphql::*;

use crate::graphql::queries::Query;
use crate::graphql::mutations::Mutation;

pub type GraphQLSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema() -> GraphQLSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .finish()
}