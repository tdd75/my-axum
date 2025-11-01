use std::collections::HashMap;

use axum::http::{HeaderMap, StatusCode, header::HeaderValue};
use chrono::{Duration, Utc};
use http::header::{AUTHORIZATION, COOKIE};
use rust_i18n::t;
use sea_orm::entity::*;

use crate::{
    config::setting::Setting,
    core::{
        context::Context,
        db::entity::{refresh_token, user},
        dto::error_dto::ErrorDTO,
    },
    pkg::jwt::{decode_token, encode_token},
    user::repository::{refresh_token_repository, user_repository},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

// ------------------------------------------------
// Token
// ------------------------------------------------

pub async fn generate_token_pair(user_id: i32) -> Result<(String, String), ErrorDTO> {
    let setting = Setting::new();

    let access_token = encode_token(
        user_id,
        Duration::seconds(setting.jwt_access_token_expires),
        &setting.jwt_secret,
    )
    .map_err(ErrorDTO::map_internal_error)?;

    let refresh_token = encode_token(
        user_id,
        Duration::seconds(setting.jwt_refresh_token_expires),
        &setting.jwt_secret,
    )
    .map_err(ErrorDTO::map_internal_error)?;

    Ok((access_token, refresh_token))
}

pub async fn create_refresh_token_record(
    context: &Context<'_>,
    user_id: i32,
    token: &str,
    headers: &HeaderMap,
) -> Result<refresh_token::Model, ErrorDTO> {
    let setting = Setting::new();
    let device_info = get_device_info(headers);
    let ip_address = get_client_ip(headers);
    let expires_at = Utc::now().naive_utc() + Duration::seconds(setting.jwt_refresh_token_expires);

    let refresh_token_record = refresh_token::ActiveModel {
        user_id: Set(user_id),
        token: Set(token.to_string()),
        device_info: Set(device_info),
        ip_address: Set(ip_address),
        expires_at: Set(expires_at),
        ..Default::default()
    };

    refresh_token_repository::create(context, refresh_token_record)
        .await
        .map_err(ErrorDTO::map_internal_error)
}

pub async fn get_current_user(
    context: &Context<'_>,
    access_token: &str,
) -> Result<user::Model, ErrorDTO> {
    let setting = Setting::new();

    let claims = decode_token(access_token, setting.jwt_secret.as_str()).map_err(|_| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("authorization.invalid_token").to_string(),
        )
    })?;

    let user = user_repository::find_by_id(context, claims.sub)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("authorization.user_not_found").to_string(),
            )
        })?;

    Ok(user)
}

// ------------------------------------------------
// Tracking
// ------------------------------------------------

pub fn get_device_info(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub fn get_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded) = headers.get("x-forwarded-for")
        && let Ok(forwarded_str) = forwarded.to_str()
    {
        return Some(
            forwarded_str
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string(),
        );
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip")
        && let Ok(ip_str) = real_ip.to_str()
    {
        return Some(ip_str.to_string());
    }

    None
}

// ------------------------------------------------
// Cookie & Header Token
// ------------------------------------------------

pub fn set_auth_cookies(headers: &mut HeaderMap, access_token: &str, refresh_token: &str) {
    let setting = Setting::new();

    let cookie_attributes = "HttpOnly; SameSite=Strict; Secure";

    // Set access token cookie
    let access_cookie = format!(
        "access_token={}; Max-Age={}; {}; Path=/",
        access_token, setting.jwt_access_token_expires, cookie_attributes
    );

    // Set refresh token cookie
    let refresh_cookie = format!(
        "refresh_token={}; Max-Age={}; {}; Path=/",
        refresh_token,
        setting.jwt_refresh_token_expires * 24 * 60 * 60, // convert days to seconds
        cookie_attributes
    );

    if let Ok(access_header_value) = HeaderValue::from_str(&access_cookie) {
        headers.append("set-cookie", access_header_value);
    }

    if let Ok(refresh_header_value) = HeaderValue::from_str(&refresh_cookie) {
        headers.append("set-cookie", refresh_header_value);
    }
}

pub async fn extract_token_from_header_or_cookie(
    header_map: &HeaderMap,
    token_type: TokenType,
) -> Result<String, ErrorDTO> {
    let token_name = match token_type {
        TokenType::Access => "access_token",
        TokenType::Refresh => "refresh_token",
    };

    // Priority 1: Try to get token from cookie
    if let Some(token) = get_token_from_cookies(header_map, token_name)
        && !token.trim().is_empty()
    {
        return Ok(token);
    }

    // Priority 2: Try to get token from Authorization header (only for access tokens)
    if token_type == TokenType::Access
        && let Some(token) = get_token_from_headers(header_map)?
        && !token.trim().is_empty()
    {
        return Ok(token);
    }

    // No token found anywhere
    let error_key = match token_type {
        TokenType::Access => "authorization.token_not_found",
        TokenType::Refresh => "auth.refresh_token_required",
    };

    Err(ErrorDTO::new(
        StatusCode::UNAUTHORIZED,
        t!(error_key).to_string(),
    ))
}

pub fn get_token_from_headers(header_map: &HeaderMap) -> Result<Option<String>, ErrorDTO> {
    if let Some(authorization) = header_map.get(AUTHORIZATION) {
        let auth_str = authorization.to_str().map_err(|_| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("authorization.invalid_header").to_string(),
            )
        })?;

        if let Some(access_token) = auth_str.strip_prefix("Bearer ") {
            return Ok(Some(access_token.to_string()));
        }
    }

    Ok(None)
}

pub fn get_token_from_cookies(headers: &HeaderMap, token_name: &str) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .map(|cookie| cookie.trim())
                .find(|cookie| cookie.starts_with(&format!("{}=", token_name)))
                .map(|cookie| {
                    cookie
                        .trim_start_matches(&format!("{}=", token_name))
                        .to_string()
                })
        })
}

pub fn get_token_from_query_params(query_params: &str, token_name: &str) -> Option<String> {
    serde_urlencoded::from_str::<HashMap<String, String>>(query_params)
        .ok()
        .and_then(|params| params.get(token_name).cloned())
}
