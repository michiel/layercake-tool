use async_graphql::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    LoginInput, LoginResponse, RegisterResponse, RegisterUserInput, UpdateUserInput, User,
};
use layercake_core::database::entities::{user_sessions, users};
use layercake_core::services::auth_service::AuthService;

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Register a new user
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: RegisterUserInput,
    ) -> Result<RegisterResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Validate input
        AuthService::validate_email(&input.email)
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;
        AuthService::validate_username(&input.username)
            .map_err(|e| crate::graphql::errors::core_error_to_graphql_error(e))?;
        AuthService::validate_display_name(&input.display_name)
            .map_err(|e| crate::graphql::errors::core_error_to_graphql_error(e))?;

        // Check if user already exists
        let existing_user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::find (email)", e))?;

        if existing_user.is_some() {
            return Err(StructuredError::conflict(
                "User",
                "User with this email already exists",
            ));
        }

        let existing_username = users::Entity::find()
            .filter(users::Column::Username.eq(&input.username))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::find (username)", e))?;

        if existing_username.is_some() {
            return Err(StructuredError::conflict("User", "Username already taken"));
        }

        // Hash password using bcrypt
        let password_hash = AuthService::hash_password(&input.password)
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        // Create user
        let mut user = users::ActiveModel::new();
        user.email = Set(input.email);
        user.username = Set(input.username);
        user.display_name = Set(input.display_name);
        user.password_hash = Set(password_hash);
        user.avatar_color = Set(AuthService::generate_avatar_color());

        let user = user
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::insert", e))?;

        // Create session for the new user
        let session = user_sessions::ActiveModel::new(user.id, user.username.clone(), 1); // Assuming project ID 1 for now
        let session = session
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("user_sessions::Entity::insert", e))?;

        Ok(RegisterResponse {
            user: User::from(user),
            session_id: session.session_id,
            expires_at: session.expires_at,
        })
    }

    /// Login user
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::find (login email)", e))?
            .ok_or_else(|| StructuredError::unauthorized("Invalid email or password"))?;

        // Verify password using bcrypt
        let is_valid = AuthService::verify_password(&input.password, &user.password_hash)
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        if !is_valid {
            return Err(StructuredError::unauthorized("Invalid email or password"));
        }

        // Check if user is active
        if !user.is_active {
            return Err(StructuredError::forbidden("Account is deactivated"));
        }

        // Create new session
        let session = user_sessions::ActiveModel::new(user.id, user.username.clone(), 1); // Assuming project ID 1 for now
        let session = session
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("user_sessions::Entity::insert", e))?;

        // Update last login
        let mut user_active: users::ActiveModel = user.clone().into();
        user_active.last_login_at = Set(Some(Utc::now()));
        user_active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::update (last_login)", e))?;

        Ok(LoginResponse {
            user: User::from(user),
            session_id: session.session_id,
            expires_at: session.expires_at,
        })
    }

    /// Logout user (deactivate session)
    async fn logout(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find and deactivate session
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(&session_id))
            .one(&context.db)
            .await
            .map_err(|e| {
                StructuredError::database("user_sessions::Entity::find (session_id)", e)
            })?;

        if let Some(session) = session {
            let mut session_active: user_sessions::ActiveModel = session.into();
            session_active = session_active.deactivate();
            session_active
                .update(&context.db)
                .await
                .map_err(|e| StructuredError::database("user_sessions::Entity::update", e))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update user profile
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        user_id: i32,
        input: UpdateUserInput,
    ) -> Result<User> {
        let context = ctx.data::<GraphQLContext>()?;

        let user = users::Entity::find_by_id(user_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("User", user_id))?;

        let mut user_active: users::ActiveModel = user.into();

        if let Some(display_name) = input.display_name {
            user_active.display_name = Set(display_name);
        }

        if let Some(email) = input.email {
            // Check if email is already taken by another user
            let existing = users::Entity::find()
                .filter(users::Column::Email.eq(&email))
                .filter(users::Column::Id.ne(user_id))
                .one(&context.db)
                .await
                .map_err(|e| StructuredError::database("users::Entity::find (email check)", e))?;

            if existing.is_some() {
                return Err(StructuredError::conflict("User", "Email already taken"));
            }

            user_active.email = Set(email);
        }

        user_active = user_active.set_updated_at();
        let updated_user = user_active.update(&context.db).await?;

        Ok(User::from(updated_user))
    }
}
