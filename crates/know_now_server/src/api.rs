use std::path::Path;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::cookie::CookieJar;
use know_now_core::project_loader;
use know_now_metadata::authoring::AuthoringMetadata;
use know_now_metadata::budgets::ParserBudgets;

use crate::AppState;

const SESSION_COOKIE: &str = "kn_session";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/version", get(handle_version))
        .route("/api/v1/project", get(handle_project))
        .route("/api/v1/domains", get(handle_domains))
        .route("/api/v1/modules", get(handle_modules))
        .route("/api/v1/entities", get(handle_entities))
        .route("/api/v1/relationships", get(handle_relationships))
        .route("/api/v1/open-questions", get(handle_open_questions))
        .route("/api/v1/graph", get(handle_graph))
}

#[allow(clippy::result_large_err)]
fn require_session(state: &AppState, jar: &CookieJar) -> Result<(), Response> {
    let valid = jar
        .get(SESSION_COOKIE)
        .is_some_and(|c| state.sessions.validate(c.value()));
    if valid {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED.into_response())
    }
}

#[allow(clippy::result_large_err)]
fn load_metadata(project_root: &Path) -> Result<AuthoringMetadata, Response> {
    let metadata_dir = project_root.join("metadata");
    if !metadata_dir.is_dir() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "no metadata/ directory found",
        )
            .into_response());
    }
    project_loader::load_project(&metadata_dir, &ParserBudgets::default())
        .map(|p| p.metadata)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load project: {e}"),
            )
                .into_response()
        })
}

async fn handle_version(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    axum::Json(serde_json::json!({
        "engine_version": env!("CARGO_PKG_VERSION"),
        "api_contract_version": "1",
        "compatibility": "current",
    }))
    .into_response()
}

async fn handle_project(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({
        "project": metadata.project,
        "version": metadata.version,
    }))
    .into_response()
}

async fn handle_domains(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({ "domains": metadata.domains })).into_response()
}

async fn handle_modules(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({ "modules": metadata.modules })).into_response()
}

async fn handle_entities(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({ "entities": metadata.entities })).into_response()
}

async fn handle_relationships(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({ "relationships": metadata.relationships })).into_response()
}

async fn handle_open_questions(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    axum::Json(serde_json::json!({ "open_questions": metadata.open_questions })).into_response()
}

async fn handle_graph(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let metadata = match load_metadata(&state.config.project_root) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let nodes: Vec<serde_json::Value> = metadata
        .entities
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "name": e.name,
                "domain": e.domain,
            })
        })
        .collect();

    let edges: Vec<serde_json::Value> = metadata
        .relationships
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "from": r.from_entity,
                "to": r.to_entity,
            })
        })
        .collect();

    axum::Json(serde_json::json!({ "nodes": nodes, "edges": edges })).into_response()
}
