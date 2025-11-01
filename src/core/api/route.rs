use axum::{
    Router,
    routing::{any, get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::{
    common::api::mcp_api,
    common::api::{runbook_api, task_ws},
    core::api::openapi::ApiDoc,
    user::api::user_ws,
};
use crate::{
    config::app::AppState,
    core::layer::{
        auth_layer::auth_middleware, lang_layer::lang_middleware,
        page_size_limit_layer::page_size_limit_middleware,
        transaction_layer::transaction_middleware,
    },
    user::api::{auth_api, user_api},
};

pub const SWAGGER_UI_PATH: &str = "/docs";
pub const OPENAPI_JSON_PATH: &str = "/docs/openapi.json";

pub fn get_route(app_state: AppState) -> Router<AppState> {
    let swagger_route = Router::new().merge(
        SwaggerUi::new(SWAGGER_UI_PATH)
            .url(OPENAPI_JSON_PATH, ApiDoc::openapi())
            .config(Config::default().persist_authorization(true)),
    );

    let runbook_route = Router::new()
        .route("/api/v1/runbook/", get(runbook_api::list_runbooks))
        .route("/api/v1/runbook/run/", post(runbook_api::run_runbook))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            transaction_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    let mcp_route = Router::new()
        .nest_service("/mcp", mcp_api::service(&app_state))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn(lang_middleware));

    let no_auth_route = Router::new()
        .route("/api/v1/auth/login/", post(auth_api::login))
        .route("/api/v1/auth/register/", post(auth_api::register))
        .route("/api/v1/auth/refresh-token/", post(auth_api::refresh_token))
        .route("/api/v1/auth/logout/", post(auth_api::logout))
        .route(
            "/api/v1/auth/forgot-password/",
            post(auth_api::forgot_password),
        )
        .route(
            "/api/v1/auth/reset-password/",
            post(auth_api::reset_password),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            transaction_middleware,
        ))
        .route_layer(axum::middleware::from_fn(lang_middleware))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            page_size_limit_middleware,
        ));

    let auth_route = Router::new()
        .route("/ws/v1/task/{task_id}/", any(task_ws::get_task_progress))
        .route("/ws/v1/user/", any(user_ws::sync_user_data))
        .route(
            "/api/v1/user/profile/",
            get(user_api::get_profile).patch(user_api::update_profile),
        )
        .route(
            "/api/v1/auth/change-password/",
            post(auth_api::change_password),
        )
        .route(
            "/api/v1/user/",
            get(user_api::search_user).post(user_api::create_user),
        )
        .route("/api/v1/user/upload-avatar/", post(user_api::upload_avatar))
        .route(
            "/api/v1/user/{id}/",
            get(user_api::get_user)
                .patch(user_api::update_user)
                .delete(user_api::delete_user),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            transaction_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn(lang_middleware))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            page_size_limit_middleware,
        ));

    swagger_route
        .merge(runbook_route)
        .merge(mcp_route)
        .merge(no_auth_route)
        .merge(auth_route)
}
