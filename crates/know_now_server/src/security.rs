use axum::http::{HeaderName, HeaderValue};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;

pub fn cors_layer(origin: &str) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            origin.parse::<HeaderValue>().expect("valid origin"),
        ))
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::CONTENT_TYPE, http::header::COOKIE])
        .allow_credentials(true)
}

pub fn x_content_type_options() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    )
}

pub fn x_frame_options() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    )
}

pub fn content_security_policy() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'",
        ),
    )
}

pub fn referrer_policy() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    )
}
