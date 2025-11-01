use crate::user::api::{auth_api, user_api};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        auth_api::get_profile,
        auth_api::update_profile,
        auth_api::change_password,
        auth_api::forgot_password,
        auth_api::reset_password,
        auth_api::login,
        auth_api::register,
        auth_api::refresh_token,
        auth_api::logout,
        user_api::search_user,
        user_api::get_user,
        user_api::create_user,
        user_api::update_user,
        user_api::delete_user,
        user_api::upload_avatar,
    ),
)]
pub struct ApiDoc;
