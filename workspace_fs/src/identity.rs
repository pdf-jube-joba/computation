use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

#[derive(Debug, Clone)]
pub struct IdentityConfig;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestIdentity {
    pub user: String,
}

impl IdentityConfig {
    pub fn load() -> Self {
        Self
    }
}

pub async fn capture_identity(
    State(_identity): State<IdentityConfig>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let user = request
        .headers()
        .get("user-identity")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .map(ToOwned::to_owned);

    let Some(user) = user.filter(|value| !value.is_empty()).or_else(|| {
        if method == Method::GET {
            Some(String::new())
        } else {
            None
        }
    }) else {
        return (
            StatusCode::BAD_REQUEST,
            "missing user-identity header",
        )
            .into_response();
    };

    request.extensions_mut().insert(RequestIdentity { user });
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        extract::Extension,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
    };
    use tower::ServiceExt;

    async fn identity_handler(Extension(identity): Extension<RequestIdentity>) -> impl IntoResponse {
        identity.user
    }

    fn app() -> Router {
        Router::new()
            .route("/", get(identity_handler).post(identity_handler))
            .layer(middleware::from_fn_with_state(
                IdentityConfig::load(),
                capture_identity,
            ))
    }

    #[tokio::test]
    async fn get_allows_missing_user_identity() {
        let response = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_requires_user_identity() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
