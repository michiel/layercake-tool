use async_graphql::*;

use crate::graphql::mutations::Mutation;
use crate::graphql::queries::Query;
use crate::graphql::subscriptions::Subscription;

pub type GraphQLSchema = Schema<Query, Mutation, Subscription>;
