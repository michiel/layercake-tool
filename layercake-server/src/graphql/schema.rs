use async_graphql::*;

use crate::graphql::mutations::Mutation;
use crate::graphql::queries::Query;
use crate::graphql::subscriptions::Subscription;

pub type GraphQLSchema = Schema<Query, Mutation, Subscription>;

/// Build the GraphQL schema without a request/database context.
///
/// The type system (SDL / introspection) does not depend on runtime data, so
/// this is enough for tooling like `layercake schema dump`. Do NOT use the
/// result to execute real operations — resolvers expect a `GraphQLContext` in
/// the schema data, which is only wired up in `create_app`.
pub fn build_schema_for_introspection() -> GraphQLSchema {
    Schema::build(Query, Mutation::default(), Subscription).finish()
}

/// The GraphQL SDL for the API (no server or database required).
pub fn sdl() -> String {
    build_schema_for_introspection().sdl()
}

/// Run the standard introspection query and return the JSON result string.
pub async fn introspection_json() -> anyhow::Result<String> {
    let schema = build_schema_for_introspection();
    let response = schema.execute(INTROSPECTION_QUERY).await;
    let value = serde_json::to_value(&response)?;
    Ok(serde_json::to_string_pretty(&value)?)
}

const INTROSPECTION_QUERY: &str = r#"
query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types { ...FullType }
    directives { name description locations args { ...InputValue } }
  }
}
fragment FullType on __Type {
  kind name description
  fields(includeDeprecated: true) {
    name description
    args { ...InputValue }
    type { ...TypeRef }
    isDeprecated deprecationReason
  }
  inputFields { ...InputValue }
  interfaces { ...TypeRef }
  enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason }
  possibleTypes { ...TypeRef }
}
fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue }
fragment TypeRef on __Type {
  kind name
  ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name } } } } } } }
}
"#;
