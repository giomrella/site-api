use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use lambda_http::{run, tracing, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::Layer;
use tower_http::{add_extension::AddExtensionLayer, normalize_path::NormalizePathLayer};
use std::env::set_var;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
struct Params {
    first: Option<String>,
    second: Option<String>,
}
#[derive(Default)]
struct ApiState {
    counter: i32,
}
type SharedState = Arc<RwLock<ApiState>>;

async fn root() -> Json<Value> {
    Json(json!({ "msg": "I am GET /" }))
}
async fn increment(Extension(state): Extension<SharedState>) -> String {
    let counter = state.read().unwrap().counter + 1;
    state.write().unwrap().counter = counter;
    format!("{counter}")
}

async fn echo(Path(echo): Path<String>) -> String {
    echo
}

async fn get_foo() -> Json<Value> {
    Json(json!({ "msg": "I am GET /foo" }))
}

async fn post_foo() -> Json<Value> {
    Json(json!({ "msg": "I am POST /foo" }))
}

async fn post_foo_name(Path(name): Path<String>) -> Json<Value> {
    Json(json!({ "msg": format!("I am POST /foo/:name, name={name}") }))
}

async fn get_parameters(Query(params): Query<Params>) -> Json<Value> {
    Json(json!({ "request parameters": params }))
}

/// Example on how to return status codes and data from an Axum function
async fn health_check() -> (StatusCode, String) {
    let health = true;
    match health {
        true => (StatusCode::OK, "Healthy!".to_string()),
        false => (StatusCode::INTERNAL_SERVER_ERROR, "Not healthy!".to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "false");
    set_var("RUST_BACKTRACE", "FULL");

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();
    let router = Router::new()
        .route("/", get(root))
        .route("/echo/:echo", get(echo))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/:name", post(post_foo_name))
        .route("/parameters", get(get_parameters))
        .route("/health", get(health_check))
        .route("/increment", get(increment))
        .layer(AddExtensionLayer::new(SharedState::default()))
        ;
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    run(app).await
}
