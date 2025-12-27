use async_graphql::*;

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{SystemSetting, SystemSettingUpdateInput};

#[derive(Default)]
pub struct SystemMutation;

#[Object]
impl SystemMutation {
    /// Update a runtime system setting value
    async fn update_system_setting(
        &self,
        ctx: &Context<'_>,
        input: SystemSettingUpdateInput,
    ) -> Result<SystemSetting> {
        let context = ctx.data::<GraphQLContext>()?;
        let updated = context
            .system_settings
            .update_setting(&input.key, input.value)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(SystemSetting::from(updated))
    }
}
