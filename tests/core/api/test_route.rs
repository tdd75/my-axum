use my_axum::core::api::route::get_route;

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_get_route_builds_router_from_app_state() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let _router = get_route(app_state);
}
