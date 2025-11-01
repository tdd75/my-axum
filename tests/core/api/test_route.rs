use my_axum::core::api::route::get_route;

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_get_route_returns_router() {
    let test_app = TestApp::spawn_app().await;
    let app_state = test_app.create_app_state();

    let _router = get_route(app_state);
    // Just verify the router is created without errors - test passes if no panic occurs
}
