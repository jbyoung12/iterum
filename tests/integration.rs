use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use std::sync::Arc;

fn build_app() -> axum::Router {
    let store = iterum::store::memory::InMemoryStore::new(604800);
    let retriever = Arc::new(iterum::retrieval::ContextRetriever::new(store, 3));
    iterum::routes::build_router(retriever)
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

#[tokio::test]
async fn health_check() {
    let app = build_app();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn store_and_retrieve_fact() {
    let app = build_app();

    // Store a fact
    let store_req = json_request(
        "POST",
        "/v1/context/facts",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "sqlite:~/test.db",
            "topic": "schema:users",
            "title": "Schema for users",
            "content": "CREATE TABLE users (id INTEGER, name TEXT)",
            "confidence": 0.95
        }),
    );
    let resp = app.clone().oneshot(store_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let fact: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(fact["id"].as_str().unwrap().starts_with("fact_"));

    // Retrieve it
    let retrieve_req = json_request(
        "POST",
        "/v1/context/retrieve",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "sqlite:~/test.db"
        }),
    );
    let resp = app.oneshot(retrieve_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let ctx: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ctx["facts"].as_array().unwrap().len(), 1);
    assert!(ctx["prompt_context"]
        .as_str()
        .unwrap()
        .contains("Schema for users"));
}

#[tokio::test]
async fn tool_result_event_extracts_schema() {
    let app = build_app();

    // Send a tool-result event with .schema output
    let event_req = json_request(
        "POST",
        "/v1/events/tool-result",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "bash",
            "command": "sqlite3 ~/test.db '.schema orders'",
            "resource_id": "sqlite:~/test.db",
            "stdout": "CREATE TABLE orders (id INTEGER PRIMARY KEY, amount REAL, created_at TEXT);",
            "is_error": false
        }),
    );
    let resp = app.clone().oneshot(event_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let stored = result["stored"].as_array().unwrap();
    assert!(
        stored.iter().any(|s| s["record_type"] == "fact"),
        "should extract a fact from .schema output"
    );

    // Verify it's retrievable
    let retrieve_req = json_request(
        "POST",
        "/v1/context/retrieve",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "sqlite:~/test.db"
        }),
    );
    let resp = app.oneshot(retrieve_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let ctx: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(!ctx["facts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn tool_result_event_detects_error_and_creates_constraint() {
    let app = build_app();

    let event_req = json_request(
        "POST",
        "/v1/events/tool-result",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "bash",
            "command": "sqlite3 ~/test.db 'SELECT bad_col FROM users'",
            "resource_id": "sqlite:~/test.db",
            "stdout": "",
            "stderr": "Error: no such column: bad_col",
            "is_error": true
        }),
    );
    let resp = app.clone().oneshot(event_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let stored = result["stored"].as_array().unwrap();
    assert!(
        stored.iter().any(|s| s["record_type"] == "constraint"),
        "should create a constraint from column error"
    );
    assert!(
        stored
            .iter()
            .any(|s| s["record_type"] == "failure_pattern"),
        "should create a failure pattern"
    );
}

#[tokio::test]
async fn failure_pattern_increments_on_repeat() {
    let app = build_app();

    let make_event = || {
        json_request(
            "POST",
            "/v1/events/tool-result",
            serde_json::json!({
                "namespace": "default",
                "tool_name": "bash",
                "command": "sqlite3 ~/test.db 'SELECT nope FROM t'",
                "resource_id": "sqlite:~/test.db",
                "stdout": "",
                "stderr": "Error: no such column: nope",
                "is_error": true
            }),
        )
    };

    // First occurrence
    let resp = app.clone().oneshot(make_event()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second occurrence
    let resp = app.clone().oneshot(make_event()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Check debug endpoint for failure patterns
    let debug_req = Request::builder()
        .uri("/v1/debug/failure-patterns?namespace=default&tool_name=sqlite3&resource_id=sqlite:~/test.db")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(debug_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let patterns: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!patterns.is_empty());
    let count = patterns[0]["occurrence_count"].as_u64().unwrap();
    assert_eq!(count, 2, "occurrence_count should be 2 after two events");
}

#[tokio::test]
async fn debug_constraints_endpoint() {
    let app = build_app();

    // Store via event
    let event_req = json_request(
        "POST",
        "/v1/events/tool-result",
        serde_json::json!({
            "namespace": "default",
            "tool_name": "bash",
            "command": "sqlite3 ~/test.db 'SELECT missing FROM t'",
            "resource_id": "sqlite:~/test.db",
            "stderr": "Error: no such column: missing",
            "is_error": true
        }),
    );
    app.clone().oneshot(event_req).await.unwrap();

    let debug_req = Request::builder()
        .uri("/v1/debug/constraints?namespace=default&tool_name=sqlite3")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(debug_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let constraints: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!constraints.is_empty());
}
