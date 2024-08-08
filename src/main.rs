use std::{collections::HashMap, fs::File, usize};

use axum::{extract::Query, http::StatusCode, routing::get, Router};
use log_file::tail_file;

mod log;
mod log_file;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "healthy" }))
        .route("/health", get(|| async { "healthy" }))
        .route("/logs", get(read_logs));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn read_logs(Query(params): Query<HashMap<String, String>>) -> (StatusCode, String) {
    let user_id = match params.get("user_id") {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, "no user_id provided in query string".to_string()),
    };
    let default_limit = String::from("32");
    let limit = params.get("limit").unwrap_or(&default_limit);
    let limit: usize = match limit.parse() {
        Ok(val) => val,
        Err(e) => return (StatusCode::OK, format!("error parsing limit query param: {e}")),
    };

    let f = match File::open(user_id) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error opening file for {user_id}: {e}")),
    };

    let lines = match tail_file(&f, limit) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error tailing file for {user_id}: {e}")),
    };

    for line in lines {
        println!("{line}");
    }

    return (StatusCode::OK, user_id.to_string() + &limit.to_string());
}

