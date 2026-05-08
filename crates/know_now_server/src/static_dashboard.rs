//! SPA static-file router for the embedded dashboard.
//!
//! Active only with the `serve-dashboard` feature. Serves `index.html` at `/`
//! and hashed assets at `/assets/{*path}`. Explicit API/auth routes
//! (`/__open`, `/__health`, `/api/*`) take priority via axum's matcher.

#[cfg(feature = "serve-dashboard")]
mod inner {
    use axum::{
        extract::Path,
        http::{header, StatusCode},
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use rust_embed::RustEmbed;

    use crate::AppState;

    #[derive(RustEmbed)]
    #[folder = "../../web/dist"]
    struct Dashboard;

    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/", get(serve_index))
            .route("/assets/{*path}", get(serve_asset))
    }

    async fn serve_index() -> Response {
        match Dashboard::get("index.html") {
            Some(file) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                file.data,
            )
                .into_response(),
            None => (
                StatusCode::SERVICE_UNAVAILABLE,
                "dashboard not built — run `pnpm build` in web/ and rebuild",
            )
                .into_response(),
        }
    }

    async fn serve_asset(Path(path): Path<String>) -> Response {
        let full = format!("assets/{path}");
        match Dashboard::get(&full) {
            Some(file) => {
                let mime = mime_guess::from_path(&full).first_or_octet_stream();
                let mime_str = mime.as_ref().to_owned();
                ([(header::CONTENT_TYPE, mime_str)], file.data).into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn router_has_routes() {
            let _r: Router<AppState> = router();
        }
    }
}

#[cfg(feature = "serve-dashboard")]
pub use inner::router;

#[cfg(not(feature = "serve-dashboard"))]
pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
}
