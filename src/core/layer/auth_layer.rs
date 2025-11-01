use crate::config::app::AppState;
use crate::core::context::Context;
use crate::core::db::uow::new_transaction;
use crate::core::dto::error_dto::ErrorDTO;
use crate::core::layer::lang_layer::RequestLocale;
use crate::user::entity::sea_orm_active_enums::UserRole;
use crate::user::entity::user;
use crate::user::service::auth_service::{self, TokenType};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{extract::Request, middleware::Next, response::Response};
use rust_i18n::t;

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorDTO> {
    let headers = req.headers().clone();
    let uri_query = req.uri().query().map(|q| q.to_string());
    let locale = req
        .extensions()
        .get::<RequestLocale>()
        .map(|l| l.as_str().to_string());

    let current_user = new_transaction(&app_state, None, locale, move |context| {
        Box::pin(async move {
            let access_token = auth_service::extract_token_from_header_or_cookie(
                &headers,
                TokenType::Access,
                &context.locale,
            )
            .await
            .or_else(|_| {
                // Try to get token from query parameters (for WebSocket connections)
                uri_query
                    .as_ref()
                    .and_then(|query| auth_service::get_token_from_query_params(query, "token"))
                    .ok_or_else(|| {
                        ErrorDTO::new(
                            axum::http::StatusCode::UNAUTHORIZED,
                            t!("auth.access_token_not_found", locale = &context.locale).to_string(),
                        )
                    })
            })?;

            auth_service::get_current_user(context, &access_token).await
        })
    })
    .await?;

    req.extensions_mut().insert(current_user);

    Ok(next.run(req).await)
}

pub fn authorize_role(
    context: &Context,
    current_user: &user::Model,
    required_role: UserRole,
) -> Result<(), ErrorDTO> {
    if current_user.role != required_role {
        let message = t!(
            "authorization.role_required",
            role = localized_role(required_role, &context.locale),
            locale = &context.locale
        )
        .to_string();
        return Err(ErrorDTO::new(StatusCode::FORBIDDEN, message));
    }

    Ok(())
}

fn localized_role(role: UserRole, locale: &str) -> String {
    let key = match role {
        UserRole::Admin => "authorization.role.admin",
        UserRole::User => "authorization.role.user",
    };

    t!(key, locale = locale).to_string()
}
