use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use crate::{common::use_case::mcp::mcp_use_case::McpServer, config::app::AppState};

pub fn service(app_state: &AppState) -> StreamableHttpService<McpServer, LocalSessionManager> {
    let mut session_manager = LocalSessionManager::default();
    session_manager.session_config.sse_retry = None;
    let api_base_url = api_base_url(app_state);
    let mut config = StreamableHttpServerConfig::default()
        .with_stateful_mode(true)
        .with_sse_retry(None)
        .with_json_response(false)
        .with_cancellation_token(app_state.shutdown_token.clone());

    if let Some(allowed_hosts) = allowed_hosts(app_state) {
        config = config.with_allowed_hosts(allowed_hosts);
    } else {
        config = config.disable_allowed_hosts();
    }

    StreamableHttpService::new(
        move || Ok(McpServer::new(api_base_url.clone())),
        session_manager.into(),
        config,
    )
}

fn api_base_url(app_state: &AppState) -> String {
    let host = match app_state.setting.app_host.as_str() {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "[::1]".to_string(),
        host if host.contains(':') && !host.starts_with('[') => format!("[{host}]"),
        host => host.to_string(),
    };

    format!("http://{}:{}", host, app_state.setting.app_port)
}

fn allowed_hosts(app_state: &AppState) -> Option<Vec<String>> {
    let mut hosts = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    for origin in &app_state.setting.allowed_origins {
        let origin = origin.trim();
        if origin == "*" || origin.contains('*') {
            return None;
        }

        if let Some(authority) = origin_authority(origin) {
            hosts.push(authority);
        }
    }

    if !matches!(app_state.setting.app_host.as_str(), "0.0.0.0" | "::") {
        hosts.push(app_state.setting.app_host.clone());
    }

    hosts.sort();
    hosts.dedup();
    Some(hosts)
}

fn origin_authority(origin: &str) -> Option<String> {
    let uri = origin.parse::<http::Uri>().ok()?;
    let authority = uri.authority()?;
    Some(authority.as_str().to_string())
}
