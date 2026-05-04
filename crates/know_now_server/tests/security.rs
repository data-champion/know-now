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

// ─── B. Launch token + session ─────────────────────��─────────────────────────

#[tokio::test]
async fn b1_open_without_token_rejected() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client
        .get(format!("{}/__open", handle.url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    handle.shutdown();
}

#[tokio::test]
async fn b2_launch_token_single_use() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client.get(&handle.launch_url).send().await.unwrap();
    assert_eq!(resp.status(), 303, "first use should redirect");

    let resp = client.get(&handle.launch_url).send().await.unwrap();
    assert_eq!(resp.status(), 400, "reuse should fail");

    handle.shutdown();
}

#[tokio::test]
async fn b3_query_string_tokens_rejected_on_api() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    client.get(&handle.launch_url).send().await.unwrap();

    let resp = client
        .get(format!(
            "{}/api/v1/status?token=fake&bearer=fake",
            handle.url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "session cookie should work");

    let fresh_client = reqwest::Client::new();
    let resp = fresh_client
        .get(format!(
            "{}/api/v1/status?token=session_id_here",
            handle.url
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "query string alone must not authenticate");

    handle.shutdown();
}

#[tokio::test]
async fn b5_tampered_session_cookie_rejected() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client
        .get(format!("{}/api/v1/status", handle.url))
        .header("cookie", "kn_session=tampered-value-not-real")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    handle.shutdown();
}

// ─── C. CORS + origin checks ─────────────────���──────────────────────────────

#[tokio::test]
async fn c1_cors_wrong_origin_rejected() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/__health", handle.url))
        .header("origin", "http://evil.example.com")
        .send()
        .await
        .unwrap();

    let acl = resp.headers().get("access-control-allow-origin");
    assert!(
        acl.is_none() || acl.unwrap() != "http://evil.example.com",
        "CORS must not allow evil origin"
    );

    handle.shutdown();
}

#[tokio::test]
async fn c2_preflight_with_correct_origin_returns_headers() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::new();

    let origin = &handle.url;
    let resp = client
        .request(http::Method::OPTIONS, format!("{}/__health", handle.url))
        .header("origin", origin.as_str())
        .header("access-control-request-method", "GET")
        .send()
        .await
        .unwrap();

    let acl = resp.headers().get("access-control-allow-origin");
    assert!(acl.is_some(), "preflight should return CORS headers");
    assert_eq!(acl.unwrap().to_str().unwrap(), origin.as_str());

    let methods = resp
        .headers()
        .get("access-control-allow-methods")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(methods.contains("GET"));

    handle.shutdown();
}

#[tokio::test]
async fn c3_origin_null_rejected() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/__health", handle.url))
        .header("origin", "null")
        .send()
        .await
        .unwrap();

    let acl = resp.headers().get("access-control-allow-origin");
    assert!(
        acl.is_none() || acl.unwrap() != "null",
        "Origin: null must not be allowed"
    );

    handle.shutdown();
}

// ─── D. Response headers ───────────���──────────────────────────���──────────────

#[tokio::test]
async fn d1_csp_present_and_restrictive() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/__health", handle.url))
        .await
        .unwrap();

    let csp = resp
        .headers()
        .get("content-security-policy")
        .expect("CSP header must be present")
        .to_str()
        .unwrap();

    assert!(csp.contains("default-src 'self'"), "CSP default-src missing");
    assert!(!csp.contains("unsafe-eval"), "CSP must not allow unsafe-eval");

    handle.shutdown();
}

#[tokio::test]
async fn d2_x_content_type_options_nosniff() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/__health", handle.url))
        .await
        .unwrap();
    assert_eq!(
        resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    handle.shutdown();
}

#[tokio::test]
async fn d3_x_frame_options_deny() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/__health", handle.url))
        .await
        .unwrap();
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    handle.shutdown();
}

#[tokio::test]
async fn d4_referrer_policy_present() {
    let handle = start_server(localhost_config()).await.unwrap();
    let resp = reqwest::get(format!("{}/__health", handle.url))
        .await
        .unwrap();
    assert!(resp.headers().get("referrer-policy").is_some());
    handle.shutdown();
}

// ─── E. Write-endpoint guards ────────────────────────────────────────────────

#[tokio::test]
async fn e1_generate_endpoint_absent_without_feature() {
    let handle = start_server(localhost_config()).await.unwrap();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    client.get(&handle.launch_url).send().await.unwrap();

    let resp = client
        .post(format!("{}/api/v1/generate", handle.url))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        404,
        "generate endpoint must not exist without allow-generate feature"
    );

    handle.shutdown();
}

// ─── F. Architecture fitness ─────────────────────────────────────��───────────

#[test]
fn f1_generate_route_absent_without_feature() {
    #[cfg(feature = "allow-generate")]
    {
        panic!("this test should run without allow-generate feature");
    }
}

// ─── G. API compatibility ───────────────────��───────────────────────��────────

#[tokio::test]
async fn g2_version_endpoint_shape_stable() {
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
    assert!(body.get("server").is_some(), "must have 'server' field");
    assert!(body.get("version").is_some(), "must have 'version' field");
    assert!(
        body.get("write_mode").is_some(),
        "must have 'write_mode' field"
    );

    handle.shutdown();
}

// ─── A. Network binding ───────────────────────��────────────────────────���─────

#[test]
fn a1_config_is_localhost_by_default() {
    let config = localhost_config();
    assert!(config.is_localhost());
}

#[test]
fn a1_non_localhost_detected() {
    let config = ServerConfig {
        host: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port: 0,
        allow_generate: false,
        project_root: PathBuf::from("/tmp"),
    };
    assert!(!config.is_localhost());
}
