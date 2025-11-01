use my_axum::{
    core::context::Context,
    user::{dto::user_dto::UserCreateDTO, use_case::user::create_user_use_case},
};
use reqwest::{
    Client, Response, StatusCode,
    header::{ACCEPT, CONTENT_TYPE, HeaderMap},
};
use sea_orm::{DbErr, TransactionTrait};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::time::{Duration, timeout};

use crate::setup::{app::TestApp, fixture::login_admin_user};

const MCP_ACCEPT: &str = "application/json, text/event-stream";
const MCP_SESSION_ID: &str = "mcp-session-id";

async fn mcp_response_json(response: Response) -> Value {
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = response.text().await.unwrap();

    if content_type.starts_with("text/event-stream") {
        let data = body
            .lines()
            .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
            .find(|data| !data.is_empty())
            .expect("SSE response should contain a data line");
        serde_json::from_str(data).unwrap_or_else(|error| {
            panic!("failed to parse SSE data as JSON: {error}; body={body:?}")
        })
    } else {
        serde_json::from_str(&body).unwrap()
    }
}

async fn mcp_response_text(response: Response) -> String {
    assert!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .starts_with("text/event-stream")
    );
    response.text().await.unwrap()
}

async fn register_user_and_get_token(test_app: &TestApp, email: &str) -> String {
    let client = Client::new();
    let payload = json!({
        "email": email,
        "password": "password123@",
        "first_name": "Mcp",
        "last_name": "User",
        "phone": "1234567890"
    });

    let response = client
        .post(format!(
            "http://{}/api/v1/auth/register/",
            &test_app.base_url
        ))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    response
        .json::<Value>()
        .await
        .unwrap()
        .get("access")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

async fn admin_token(test_app: &TestApp) -> String {
    test_app
        .db
        .transaction::<_, String, DbErr>(|txn| {
            Box::pin(async move {
                let mut context = Context::builder(Arc::new(txn.begin().await?)).build();
                let (access_token, _) = login_admin_user(&mut context).await;
                context.commit().await?;
                Ok(access_token)
            })
        })
        .await
        .unwrap()
}

async fn initialize_mcp_session(client: &Client, test_app: &TestApp, access_token: &str) -> String {
    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "initialize",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "my-axum-test",
                    "version": "0.1.0"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    session_id(response.headers())
}

fn session_id(headers: &HeaderMap) -> String {
    headers
        .get(MCP_SESSION_ID)
        .expect("initialize should return an MCP session id")
        .to_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn initializes_mcp_server_for_authenticated_user() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.init@example.com").await;

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "my-axum-test",
                    "version": "0.1.0"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(!session_id(response.headers()).is_empty());
    let body_text = mcp_response_text(response).await;
    assert!(!body_text.lines().any(|line| line == "data: "));
    let data = body_text
        .lines()
        .find_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .expect("SSE response should contain a data line");
    let body: Value = serde_json::from_str(data).unwrap();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(body["result"]["serverInfo"]["name"], "my-axum");
    assert!(body["result"]["capabilities"]["tools"].is_object());
    assert!(body["result"]["capabilities"]["resources"].is_null());
}

#[tokio::test]
async fn calls_current_user_profile_tool() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.profile@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "profile",
            "method": "tools/call",
            "params": {
                "name": "get_current_user_profile",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    assert_eq!(body["id"], "profile");
    assert_eq!(
        body["result"]["structuredContent"]["email"],
        "mcp.profile@example.com"
    );
    assert_eq!(body["result"]["content"][0]["type"], "text");
}

#[tokio::test]
async fn calls_admin_search_users_tool() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = admin_token(&test_app).await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    test_app
        .db
        .transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let context = Context::builder(Arc::new(txn.begin().await?)).build();
                create_user_use_case::execute(
                    &context,
                    UserCreateDTO {
                        email: "mcp.search@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Search".to_string()),
                        last_name: Some("Target".to_string()),
                        phone: None,
                    },
                )
                .await
                .map_err(|error| DbErr::Custom(error.to_string()))?;
                context.commit().await?;
                Ok(())
            })
        })
        .await
        .unwrap();

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "search_users",
                "arguments": {
                    "email": "mcp.search",
                    "page": 1,
                    "page_size": 10,
                    "order_by": "+id"
                }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    assert_eq!(body["result"]["structuredContent"]["count"], 1);
    assert_eq!(
        body["result"]["structuredContent"]["items"][0]["email"],
        "mcp.search@example.com"
    );
}

#[tokio::test]
async fn lists_tools() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.discovery@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "tools/list",
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    assert!(body["result"]["tools"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn describes_tool_input_and_output_schemas() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.schema@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "tools/list",
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    let tools = body["result"]["tools"].as_array().unwrap();
    let search_users = tools
        .iter()
        .find(|tool| tool["name"] == "search_users")
        .unwrap();
    let properties = &search_users["inputSchema"]["properties"];

    assert_eq!(
        properties["email"]["description"],
        "Filter users by an email substring."
    );
    assert_eq!(
        properties["order_by"]["description"],
        "Sort expression such as '+id', '-created_at', or '+email'. Prefix '+' for ascending and '-' for descending."
    );
    assert_eq!(
        search_users["outputSchema"]["properties"]["items"]["description"],
        "Users in the current page."
    );

    let get_user = tools
        .iter()
        .find(|tool| tool["name"] == "get_user")
        .unwrap();
    assert_eq!(
        get_user["inputSchema"]["properties"]["id"]["description"],
        "Numeric user id to read."
    );
    assert_eq!(
        get_user["outputSchema"]["properties"]["email"]["description"],
        "User email address."
    );
}

#[tokio::test]
async fn calls_admin_get_user_tool() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = admin_token(&test_app).await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let user_id = test_app
        .db
        .transaction::<_, i32, DbErr>(|txn| {
            Box::pin(async move {
                let context = Context::builder(Arc::new(txn.begin().await?)).build();
                let user = create_user_use_case::execute(
                    &context,
                    UserCreateDTO {
                        email: "mcp.get@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Get".to_string()),
                        last_name: Some("Target".to_string()),
                        phone: None,
                    },
                )
                .await
                .map_err(|error| DbErr::Custom(error.to_string()))?
                .data;
                context.commit().await?;
                Ok(user.id)
            })
        })
        .await
        .unwrap();

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "get_user",
                "arguments": {
                    "id": user_id
                }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    assert_eq!(body["result"]["structuredContent"]["id"], user_id);
    assert_eq!(
        body["result"]["structuredContent"]["email"],
        "mcp.get@example.com"
    );
}

#[tokio::test]
async fn rejects_admin_tool_for_regular_user() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.regular@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "search_users",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = mcp_response_json(response).await;
    assert_eq!(body["error"]["code"], -32000);
    assert!(body["error"]["message"].as_str().unwrap().contains("Admin"));
}

#[tokio::test]
async fn returns_json_rpc_errors_for_bad_requests() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.errors@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let invalid_json_response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .header(CONTENT_TYPE, "application/json")
        .body("{")
        .send()
        .await
        .unwrap();
    assert_eq!(
        invalid_json_response.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );

    let unknown_method_response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "unknown/method"
        }))
        .send()
        .await
        .unwrap();
    let unknown_method = mcp_response_json(unknown_method_response).await;
    assert_eq!(unknown_method["error"]["code"], -32601);

    let removed_resource_response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "resources/read",
            "params": {
                "uri": "my-axum://users"
            }
        }))
        .send()
        .await
        .unwrap();
    let removed_resource = mcp_response_json(removed_resource_response).await;
    assert_eq!(removed_resource["error"]["code"], -32601);
}

#[tokio::test]
async fn accepts_notifications_and_opens_sse_get_stream() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.notify@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let notification_response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(&access_token)
        .header(ACCEPT, MCP_ACCEPT)
        .header(MCP_SESSION_ID, &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(notification_response.status(), StatusCode::ACCEPTED);

    let get_response = client
        .get(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, "text/event-stream")
        .header(MCP_SESSION_ID, session_id)
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn closes_sse_get_stream_when_mcp_shutdown_is_cancelled() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();
    let access_token = register_user_and_get_token(&test_app, "mcp.shutdown@example.com").await;
    let session_id = initialize_mcp_session(&client, &test_app, &access_token).await;

    let mut get_response = client
        .get(format!("http://{}/mcp/", &test_app.base_url))
        .bearer_auth(access_token)
        .header(ACCEPT, "text/event-stream")
        .header(MCP_SESSION_ID, session_id)
        .send()
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    test_app.shutdown_token.cancel();

    let next_chunk = timeout(Duration::from_secs(2), get_response.chunk())
        .await
        .expect("MCP SSE stream should close after shutdown token is cancelled")
        .unwrap();
    assert!(next_chunk.is_none());
}

#[tokio::test]
async fn requires_authentication_for_mcp_endpoint() {
    let test_app = TestApp::spawn_app().await;
    let client = Client::new();

    let response = client
        .post(format!("http://{}/mcp/", &test_app.base_url))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
