use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use know_now_server::{ServerConfig, start_server};

fn localhost_config() -> ServerConfig {
    ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 0,
        allow_generate: false,
        project_root: PathBuf::from("/tmp"),
    }
}

fn project_config(root: PathBuf) -> ServerConfig {
    ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 0,
        allow_generate: false,
        project_root: root,
    }
}

fn create_test_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let meta = dir.path().join("metadata");
    std::fs::create_dir(&meta).unwrap();
    std::fs::write(
        meta.join("project.yml"),
        r#"version: "1.0"
project:
  name: test-project
  owner: team-data
domains:
  - id: dom_sales
    name: sales
entities:
  - id: ent_customer
    name: customer
    domain: dom_sales
    description: A customer entity
    attributes:
      - id: attr_id
        name: id
        logical_type: integer
        description: PK
relationships:
  - id: rel_self
    from_entity: customer
    to_entity: customer
open_questions:
  - id: oq_1
    question: How do we handle archival?
"#,
    )
    .unwrap();
    dir
}

async fn authenticated_client(
    handle: &know_now_server::ServerHandle,
) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    client.get(&handle.launch_url).send().await.unwrap();
    client
}

#[tokio::test]
async fn health_endpoint_works_without_session() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/{}", handle.url, "__health"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    handle.shutdown();
}

#[tokio::test]
async fn status_requires_session() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/api/v1/status", handle.url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    handle.shutdown();
}

#[tokio::test]
async fn launch_token_exchange_creates_session() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let resp = client.get(&handle.launch_url).send().await.unwrap();
    assert_eq!(resp.status(), 303);

    let cookie_header = resp
        .headers()
        .get("set-cookie")
        .expect("session cookie should be set");
    let cookie_str = cookie_header.to_str().unwrap();
    assert!(cookie_str.contains("kn_session"));
    assert!(cookie_str.contains("HttpOnly"));

    handle.shutdown();
}

#[tokio::test]
async fn launch_token_single_use() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client.get(&handle.launch_url).send().await.unwrap();
    assert_eq!(resp.status(), 303);

    let resp = client.get(&handle.launch_url).send().await.unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown();
}

#[tokio::test]
async fn wrong_launch_token_rejected() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client
        .get(format!("{}/__open?launch_token=wrong", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    handle.shutdown();
}

#[tokio::test]
async fn authenticated_status_returns_ok() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    client.get(&handle.launch_url).send().await.unwrap();

    let resp = client
        .get(format!("{}/api/v1/status", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["server"], "know-now");
    assert_eq!(body["write_mode"], false);

    handle.shutdown();
}

#[tokio::test]
async fn security_headers_present() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/{}", handle.url, "__health"))
        .await
        .unwrap();

    assert_eq!(
        resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    assert!(resp
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("default-src 'self'"));
    assert_eq!(resp.headers().get("referrer-policy").unwrap(), "no-referrer");

    handle.shutdown();
}

#[tokio::test]
async fn query_string_token_rejected_on_api_routes() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!(
        "{}/api/v1/status?token=fake",
        handle.url
    ))
    .await
    .unwrap();
    assert_eq!(resp.status(), 401);
    handle.shutdown();
}

// ─── API /api/v1 endpoints ───────────────────────────────────────────────────

#[tokio::test]
async fn api_version_returns_engine_info() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/version", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["api_contract_version"], "1");
    assert!(body["engine_version"].is_string());

    handle.shutdown();
}

#[tokio::test]
async fn api_project_returns_metadata() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/project", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["project"]["name"], "test-project");

    handle.shutdown();
}

#[tokio::test]
async fn api_entities_returns_list() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/entities", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let entities = body["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0]["name"], "customer");

    handle.shutdown();
}

#[tokio::test]
async fn api_domains_returns_list() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/domains", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let domains = body["domains"].as_array().unwrap();
    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0]["name"], "sales");

    handle.shutdown();
}

#[tokio::test]
async fn api_graph_returns_nodes_and_edges() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/graph", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(body["edges"].as_array().unwrap().len(), 1);

    handle.shutdown();
}

#[tokio::test]
async fn api_open_questions_returns_list() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();
    let client = authenticated_client(&handle).await;

    let resp = client
        .get(format!("{}/api/v1/open-questions", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let questions = body["open_questions"].as_array().unwrap();
    assert_eq!(questions.len(), 1);

    handle.shutdown();
}

#[tokio::test]
async fn api_endpoints_require_session() {
    let project = create_test_project();
    let handle = start_server(project_config(project.path().to_path_buf()))
        .await
        .unwrap();

    let endpoints = [
        "/api/v1/version",
        "/api/v1/project",
        "/api/v1/domains",
        "/api/v1/entities",
        "/api/v1/relationships",
        "/api/v1/graph",
        "/api/v1/open-questions",
    ];

    for endpoint in endpoints {
        let resp = reqwest::get(format!("{}{endpoint}", handle.url))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            401,
            "{endpoint} should require authentication"
        );
    }

    handle.shutdown();
}
