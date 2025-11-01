use axum::{
    Json, Router,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use my_axum::{
    assistant::{
        dto::assistant_dto::AssistantPlannedApiCallDTO,
        service::assistant_service::execute_api_call,
    },
    config::{app::AppState, setting::Setting},
    core::db::connection::get_db,
};
use serde_json::json;

async fn test_app_state_with_base(base_url: &str) -> AppState {
    let db = get_db("sqlite::memory:").await.unwrap();
    let parsed = reqwest::Url::parse(base_url).unwrap();
    let mut setting = Setting::new();
    setting.app_host = parsed.host_str().unwrap().to_string();
    setting.app_port = parsed.port().unwrap_or(80);

    AppState {
        db,
        setting,
        producer: None,
    }
}

async fn spawn_test_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}", addr), handle)
}

#[tokio::test]
async fn execute_api_call_forwards_headers_and_json_body() {
    let app = Router::new().route(
        "/api/v1/echo/",
        post(
            |headers: HeaderMap, Json(payload): Json<serde_json::Value>| async move {
                let authorization = headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                let cookie = headers
                    .get("cookie")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                let language = headers
                    .get("accept-language")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                Json(json!({
                    "authorization": authorization,
                    "cookie": cookie,
                    "accept_language": language,
                    "payload": payload,
                }))
            },
        ),
    );

    let (base_url, handle) = spawn_test_server(app).await;
    let app_state = test_app_state_with_base(&base_url).await;
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static("Bearer abc"));
    headers.insert("cookie", HeaderValue::from_static("refresh_token=xyz"));
    headers.insert("accept-language", HeaderValue::from_static("en"));

    let api_call = AssistantPlannedApiCallDTO {
        method: "POST".to_string(),
        path: "/api/v1/echo/".to_string(),
        query: json!({}),
        body: json!({ "first_name": "Admin" }),
    };

    let result = execute_api_call(&app_state, &client, &headers, &api_call, "en")
        .await
        .unwrap();
    assert_eq!(result.status, 200);
    assert_eq!(result.body["authorization"], "Bearer abc");
    assert_eq!(result.body["cookie"], "refresh_token=xyz");
    assert_eq!(result.body["accept_language"], "en");
    assert_eq!(result.body["payload"]["first_name"], "Admin");

    handle.abort();
}

#[tokio::test]
async fn execute_api_call_wraps_text_response_and_query() {
    let app = Router::new().route(
        "/api/v1/text/",
        get(|request: Request| async move {
            let query = request.uri().query().unwrap_or_default().to_string();
            (StatusCode::ACCEPTED, format!("plain:{query}")).into_response()
        }),
    );

    let (base_url, handle) = spawn_test_server(app).await;
    let app_state = test_app_state_with_base(&base_url).await;
    let client = reqwest::Client::new();

    let api_call = AssistantPlannedApiCallDTO {
        method: "GET".to_string(),
        path: "/api/v1/text/".to_string(),
        query: json!({ "q": "test", "page": 1 }),
        body: json!({}),
    };

    let result = execute_api_call(&app_state, &client, &HeaderMap::new(), &api_call, "en")
        .await
        .unwrap();
    assert_eq!(result.status, 202);
    let text = result.body["text"].as_str().unwrap_or_default();
    assert!(text.starts_with("plain:"));
    assert!(text.contains("q=test"));
    assert!(text.contains("page=1"));

    handle.abort();
}

#[tokio::test]
async fn execute_api_call_rejects_invalid_http_method() {
    let app_state = test_app_state_with_base("http://127.0.0.1:12345").await;
    let client = reqwest::Client::new();
    let api_call = AssistantPlannedApiCallDTO {
        method: "".to_string(),
        path: "/api/v1/anything/".to_string(),
        query: json!({}),
        body: json!({}),
    };

    let error =
        match execute_api_call(&app_state, &client, &HeaderMap::new(), &api_call, "en").await {
            Ok(_) => panic!("expected execute_api_call to fail with invalid method"),
            Err(error) => error,
        };
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
}
