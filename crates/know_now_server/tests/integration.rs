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
