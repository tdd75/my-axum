use axum::{
    Router,
    routing::{any, get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

use crate::{common::api::task_ws, core::api::openapi::ApiDoc, user::api::user_ws};
use crate::{
    config::app::AppState,
    core::layer::{auth_layer::auth_middleware, lang_layer::lang_middleware},
    user::api::{auth_api, user_api},
};

pub fn get_route(app_state: AppState) -> Router<AppState> {
    let no_auth_route = Router::new()
        .merge(
            SwaggerUi::new("/docs")
                .url("/docs/openapi.json", ApiDoc::openapi())
                .config(Config::default().persist_authorization(true)),
        )
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
        .route_layer(axum::middleware::from_fn(lang_middleware));

    let auth_route = Router::new()
        .route("/ws/task/{task_id}/", any(task_ws::get_task_progress))
        .route("/ws/user/", any(user_ws::sync_user_data))
        .route(
            "/api/v1/auth/me/",
            get(auth_api::get_profile).patch(auth_api::update_profile),
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
            auth_middleware,
        ))
        .route_layer(axum::middleware::from_fn(lang_middleware));

    no_auth_route.merge(auth_route)
}
