use crate::{
    common::api::runbook_api,
    user::api::{auth_api, user_api},
};
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
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        auth_api::change_password,
        auth_api::forgot_password,
        auth_api::reset_password,
        auth_api::login,
        auth_api::register,
        auth_api::refresh_token,
        auth_api::logout,
        runbook_api::list_runbooks,
        runbook_api::run_runbook,
        user_api::search_user,
        user_api::get_user,
        user_api::create_user,
        user_api::update_user,
        user_api::delete_user,
        user_api::get_profile,
        user_api::update_profile,
        user_api::upload_avatar,
    ),
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::ApiDoc;
    use utoipa::OpenApi;
    use utoipa::openapi::security::{HttpAuthScheme, SecurityScheme};

    #[test]
    fn generates_openapi_document_with_paths() {
        let api_doc = ApiDoc::openapi();

        assert!(!api_doc.info.title.is_empty());
        assert!(!api_doc.info.version.is_empty());
        assert!(!api_doc.paths.paths.is_empty());
    }

    #[test]
    fn includes_expected_security_schemes() {
        let api_doc = ApiDoc::openapi();
        let components = api_doc.components.expect("components should exist");

        let bearer = components
            .security_schemes
            .get("bearer_auth")
            .expect("bearer_auth should exist");
        match bearer {
            SecurityScheme::Http(http) => {
                assert!(matches!(http.scheme, HttpAuthScheme::Bearer));
                assert_eq!(http.bearer_format.as_deref(), Some("JWT"));
            }
            _ => panic!("expected bearer_auth to use HTTP bearer"),
        }

        assert_eq!(components.security_schemes.len(), 1);
    }

    #[test]
    fn serializes_document_to_json() {
        let api_doc = ApiDoc::openapi();
        let json = serde_json::to_string(&api_doc).unwrap();

        assert!(json.contains("bearer_auth"));
        assert!(!json.contains("runbook_token"));
    }
}
