use std::path::Path;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::cookie::CookieJar;
use know_now_core::project_loader;
use know_now_metadata::authoring::AuthoringMetadata;
use know_now_metadata::budgets::ParserBudgets;
use know_now_writer::manifest::ManifestV1;
use serde::Deserialize;

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
        .route("/api/v1/generation-status", get(handle_generation_status))
        .route("/api/v1/manifest", get(handle_manifest))
        .route("/api/v1/docs", get(handle_docs_list))
        .route("/api/v1/docs/content", get(handle_docs_content))
        .route("/api/v1/review-state", get(handle_get_review_state))
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

async fn handle_generation_status(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let manifest_path = state.config.project_root.join("generated/manifest.json");
    let manifest = load_manifest(&manifest_path);
    let generated_dir = state.config.project_root.join("generated");

    let last_modified = std::fs::metadata(&manifest_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    let empty_warnings: Vec<String> = Vec::new();
    let (has_generation, engine_version, artifact_count, warnings) = manifest
        .as_ref()
        .map_or((false, "", 0, &empty_warnings), |m| {
            (true, m.engine_version.as_str(), m.artifacts.len(), &m.warnings)
        });

    axum::Json(serde_json::json!({
        "has_generation": has_generation,
        "engine_version": engine_version,
        "artifact_count": artifact_count,
        "warnings": warnings,
        "last_generated_epoch": last_modified,
        "generated_dir_exists": generated_dir.is_dir(),
    }))
    .into_response()
}

async fn handle_manifest(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let manifest_path = state.config.project_root.join("generated/manifest.json");
    let manifest = load_manifest(&manifest_path);
    axum::Json(serde_json::json!({ "manifest": manifest })).into_response()
}

#[derive(Deserialize)]
struct DocsContentQuery {
    path: String,
}

async fn handle_docs_list(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let docs_dir = state.config.project_root.join("generated/docs");
    let files = list_doc_files(&docs_dir);

    axum::Json(serde_json::json!({ "docs": files })).into_response()
}

async fn handle_docs_content(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<DocsContentQuery>,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let docs_dir = state.config.project_root.join("generated/docs");
    let requested = Path::new(&query.path);

    if requested.components().any(|c| c == std::path::Component::ParentDir) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let full_path = docs_dir.join(requested);
    if !full_path.starts_with(&docs_dir) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match std::fs::read_to_string(&full_path) {
        Ok(content) => axum::Json(serde_json::json!({
            "path": query.path,
            "content": content,
        }))
        .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn load_manifest(path: &Path) -> Option<ManifestV1> {
    let content = std::fs::read_to_string(path).ok()?;
    ManifestV1::from_json(&content).ok()
}

fn list_doc_files(docs_dir: &Path) -> Vec<serde_json::Value> {
    let mut files = Vec::new();
    if !docs_dir.is_dir() {
        return files;
    }
    collect_docs(docs_dir, docs_dir, &mut files);
    files
}

fn collect_docs(base: &Path, dir: &Path, out: &mut Vec<serde_json::Value>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_docs(base, &path, out);
        } else if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            if ext_str == "md" || ext_str == "html" || ext_str == "svg" {
                if let Ok(rel) = path.strip_prefix(base) {
                    out.push(serde_json::json!({
                        "path": rel.to_string_lossy(),
                        "kind": ext_str,
                    }));
                }
            }
        }
    }
}

async fn handle_get_review_state(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Response {
    if let Err(e) = require_session(&state, &jar) {
        return e;
    }

    let review_path = state
        .config
        .project_root
        .join(".knownow")
        .join("review_state.json");

    let content = std::fs::read_to_string(&review_path).unwrap_or_else(|_| "{}".to_string());
    let value: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

    axum::Json(serde_json::json!({ "review_state": value })).into_response()
}
