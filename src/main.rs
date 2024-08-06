use axum::{routing::get, Router};

pub mod log;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "healthy" }))
        .route("/log", get(get_log));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_log() -> String {
    let new_log = crate::log::Log {
        timestamp: 0,
        payload: String::from("Test"),
    };
    return new_log.to_string();
}


