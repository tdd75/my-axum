use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use form_urlencoded::{Serializer, parse};
use http::{Uri, uri::PathAndQuery};

use crate::config::app::AppState;
use crate::core::db::pagination::clamp_page_size;

pub async fn page_size_limit_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(page_size_limit) = app_state.setting.page_size_limit
        && let Some(uri) = clamp_page_size_in_uri(req.uri(), page_size_limit)
    {
        *req.uri_mut() = uri;
    }

    next.run(req).await
}

fn clamp_page_size_in_uri(uri: &Uri, page_size_limit: u64) -> Option<Uri> {
    let mut changed = false;
    let mut has_page_size = false;
    let mut pairs = Vec::new();

    if let Some(query) = uri.query() {
        for (key, value) in parse(query.as_bytes()) {
            if key == "page_size" {
                has_page_size = true;

                if let Ok(page_size) = value.parse::<u64>()
                    && let Some(clamped_page_size) =
                        clamp_page_size(Some(page_size), Some(page_size_limit))
                    && clamped_page_size != page_size
                {
                    pairs.push((key.into_owned(), clamped_page_size.to_string()));
                    changed = true;
                    continue;
                }
            }

            pairs.push((key.into_owned(), value.into_owned()));
        }
    }

    if !has_page_size {
        pairs.push(("page_size".to_string(), page_size_limit.to_string()));
        changed = true;
    }

    if !changed {
        return None;
    }

    let path = uri.path();
    let query = Serializer::new(String::new()).extend_pairs(pairs).finish();
    let path_and_query = if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{query}")
    };

    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(PathAndQuery::try_from(path_and_query).ok()?);
    Uri::from_parts(parts).ok()
}

#[cfg(test)]
mod tests {
    use http::Uri;

    use super::clamp_page_size_in_uri;

    #[test]
    fn clamps_page_size_query_param() {
        let uri: Uri = "/api/v1/user/?email=test&page=1&page_size=50"
            .parse()
            .unwrap();

        let clamped = clamp_page_size_in_uri(&uri, 20).unwrap();

        assert_eq!(
            clamped.to_string(),
            "/api/v1/user/?email=test&page=1&page_size=20"
        );
    }

    #[test]
    fn leaves_uri_unchanged_when_page_size_is_under_limit() {
        let uri: Uri = "/api/v1/user/?page_size=10".parse().unwrap();

        assert!(clamp_page_size_in_uri(&uri, 20).is_none());
    }

    #[test]
    fn adds_page_size_limit_when_query_is_missing() {
        let uri: Uri = "/api/v1/user/".parse().unwrap();

        let clamped = clamp_page_size_in_uri(&uri, 20).unwrap();

        assert_eq!(clamped.to_string(), "/api/v1/user/?page_size=20");
    }

    #[test]
    fn adds_page_size_limit_when_page_size_query_param_is_missing() {
        let uri: Uri = "/api/v1/user/?email=test".parse().unwrap();

        let clamped = clamp_page_size_in_uri(&uri, 20).unwrap();

        assert_eq!(clamped.to_string(), "/api/v1/user/?email=test&page_size=20");
    }

    #[test]
    fn ignores_invalid_page_size_values() {
        let uri: Uri = "/api/v1/user/?page_size=abc".parse().unwrap();

        assert!(clamp_page_size_in_uri(&uri, 20).is_none());
    }

    #[test]
    fn clamps_page_size_when_another_query_param_is_not_valid_percent_encoding() {
        let uri: Uri = "/api/v1/user/?filter=%E0%A4%A&page_size=50"
            .parse()
            .unwrap();

        let clamped = clamp_page_size_in_uri(&uri, 20).unwrap();

        assert!(clamped.to_string().contains("page_size=20"));
    }
}
