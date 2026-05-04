use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

use crate::AppState;

const SESSION_COOKIE: &str = "kn_session";

pub fn router(_state: AppState) -> Router<AppState> {
    let router = Router::new()
        .route("/__open", get(handle_open))
        .route("/__health", get(handle_health))
        .route("/api/v1/status", get(handle_status));

    #[cfg(feature = "allow-generate")]
    let router = router.route("/api/v1/generate", axum::routing::post(handle_generate));

    router
}

#[derive(Deserialize)]
struct OpenQuery {
    launch_token: String,
}

async fn handle_open(
    State(state): State<AppState>,
    Query(query): Query<OpenQuery>,
    jar: CookieJar,
) -> Response {
    if !state.launch_token.try_consume(&query.launch_token) {
        return (StatusCode::BAD_REQUEST, "invalid or already-used launch token")
            .into_response();
    }

    let session_id = state.sessions.create_session();
    let cookie = Cookie::build((SESSION_COOKIE, session_id))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax);

    let jar = jar.add(cookie);
    (jar, Redirect::to("/")).into_response()
}

async fn handle_health() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_status(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if !is_authenticated(&state, &jar) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    axum::Json(serde_json::json!({
        "server": "know-now",
        "version": env!("CARGO_PKG_VERSION"),
        "write_mode": state.config.allow_generate,
    }))
    .into_response()
}

#[cfg(feature = "allow-generate")]
async fn handle_generate(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if !is_authenticated(&state, &jar) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    if !state.config.allow_generate {
        return StatusCode::NOT_FOUND.into_response();
    }

    (StatusCode::NOT_IMPLEMENTED, "generate endpoint stub").into_response()
}

fn is_authenticated(state: &AppState, jar: &CookieJar) -> bool {
    jar.get(SESSION_COOKIE)
        .is_some_and(|c| state.sessions.validate(c.value()))
}
