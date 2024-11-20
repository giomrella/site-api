// use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
//
// /// This is the main body for the function.
// /// Write your code inside it.
// /// There are some code example in the following URLs:
// /// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
// async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
//     // Extract some useful information from the request
//     let who = event
//         .query_string_parameters_ref()
//         .and_then(|params| params.first("name"))
//         .unwrap_or("world");
//     let message = format!("Hello {who}, this is an AWS Lambda HTTP request");
//
//     // Return something that implements IntoResponse.
//     // It will be serialized to the right response event automatically by the runtime
//     let resp = Response::builder()
//         .status(200)
//         .header("content-type", "text/html")
//         .body(message.into())
//         .map_err(Box::new)?;
//     Ok(resp)
// }
//
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     tracing::init_default_subscriber();
//
//     run(service_fn(function_handler)).await
// }

//! This is an example function that leverages the Lambda Rust runtime HTTP support
//! and the [axum](https://docs.rs/axum/latest/axum/index.html) web framework.  The
//! runtime HTTP support is backed by the [tower::Service](https://docs.rs/tower-service/0.3.2/tower_service/trait.Service.html)
//! trait.  Axum's applications are also backed by the `tower::Service` trait.  That means
//! that it is fairly easy to build an Axum application and pass the resulting `Service`
//! implementation to the Lambda runtime to run as a Lambda function.  By using Axum instead
//! of a basic `tower::Service` you get web framework niceties like routing, request component
//! extraction, validation, etc.
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{
    extract::Path,
    response::Json,
    routing::{get, post},
    Router,
};
use lambda_http::{run, tracing, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::set_var;

#[derive(Deserialize, Serialize)]
struct Params {
    first: Option<String>,
    second: Option<String>,
}

async fn root() -> Json<Value> {
    Json(json!({ "msg": "I am GET /" }))
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

    let app = Router::new()
        .route("/", get(root))
        .route("/echo/:echo", get(echo))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/:name", post(post_foo_name))
        .route("/parameters", get(get_parameters))
        .route("/health/", get(health_check));

    run(app).await
}
