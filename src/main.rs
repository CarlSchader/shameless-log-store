use std::{collections::HashMap, env, fs::{self, File}, io::{Read, Seek, Write}, path, sync::Arc, usize};

use axum::{extract::{Query, State}, http::{header, HeaderName, StatusCode}, response::IntoResponse, routing::{get, post}, Router};
use log::merge_logs;
use log_file::{file_string_to_logs, logs_to_file_string};

mod log;
mod log_file;

struct AppState {
    log_dir: String
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        log_dir: env::var("LOGS_DIR").unwrap_or("./logs/".to_string()),
    });

    if !path::Path::new(&shared_state.log_dir).try_exists().expect("failed to check if LOGS_DIR exists") {
        fs::create_dir_all(&shared_state.log_dir).expect("failed to make LOGS_DIR");
    }

    let app = Router::new()
        .route("/", get(|| async { "healthy" }))
        .route("/health", get(|| async { "healthy" }))
        .route("/logs", get(read_logs))
        .route("/log-file", post(write_log_file))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn read_logs(State(state): State<Arc<AppState>>, Query(params): Query<HashMap<String, String>>) ->  impl IntoResponse {
    const HEADERS: [(HeaderName, &str); 1] = [(header::CONTENT_TYPE, "application/json")];
    let user_id = match params.get("user_id") {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, HEADERS, format!("no user_id provided in query string")),
    };

    let default_limit = String::from("32");
    let limit = params.get("limit").unwrap_or(&default_limit);
    let limit: usize = match limit.parse() {
        Ok(val) => val,
        Err(e) => return (StatusCode::BAD_REQUEST, HEADERS, format!("error parsing limit query param: {e}")),
    };

    let default_offset = String::from("0");
    let offset = params.get("offset").unwrap_or(&default_offset);
    let offset: usize = match offset.parse() {
        Ok(val) => val,
        Err(e) => return (StatusCode::BAD_REQUEST, HEADERS, format!("error parsing offset query param: {e}")),
    };

    let user_file_path = std::path::Path::new(&state.log_dir).join(user_id);

    if !user_file_path.exists() {
        return (StatusCode::OK, HEADERS, "[]".to_string());
    }

    let f = match File::open(user_file_path) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, HEADERS, format!("error opening file for {user_id}: {e}")),
    };

    let logs = match log_file::read_logs(&f, limit, offset) {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, HEADERS, format!("error reading logs from file for {user_id}: {e}")),
    };

    return (StatusCode::OK, HEADERS, log::Log::format_vector_as_json(&logs));
}

async fn write_log_file(State(state): State<Arc<AppState>>, Query(params): Query<HashMap<String, String>>, body: String) -> (StatusCode, String) {
    let user_id = match params.get("user_id") {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, "no user_id provided in query string".to_string()),
    };

    let user_file_path = std::path::Path::new(&state.log_dir).join(user_id);

    let new_logs = match log_file::file_string_to_logs(&body) {
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
        Ok(logs) => logs,
    };
    
    let mut user_file: fs::File;
    if user_file_path.exists() {
        user_file = match fs::File::open(&user_file_path) {
            Ok(file) => file,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error opening log file for user {user_id} -- {e}")),
        };
    } else {
        match std::fs::File::create(&user_file_path) {
            Ok(mut file) => {
                match file.write(body.as_bytes()) {
                    Ok(_) => return (StatusCode::OK, format!("new log file created for {user_id}")),
                    Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error opening log file for user {user_id} -- {e}")),
                }
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error creating log file for user {user_id} -- {e}")),
        };
    }

    let mut current_logs_file_string = String::new();
    if let Err(e) = user_file.read_to_string(&mut current_logs_file_string) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("error reading log file for user {user_id} -- {e}"));
    }

    let current_logs = match file_string_to_logs(&current_logs_file_string) {
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error parsing logs from log file for user {user_id} -- {e}")),
        Ok(logs) => logs,
    };

    let merged_logs = match merge_logs(new_logs, current_logs) {
        Ok(logs) => logs,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error merging new logs for user {user_id} -- {e}")),
    };

    if let Err(e) = user_file.seek(std::io::SeekFrom::Start(0)) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("error seeking to the start of user file for {user_id} -- {e}"));
    }

    let merged_file_string = match logs_to_file_string(&merged_logs) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error converting merged logs to file string for user {user_id} -- {e}")),
    };

    if let Err(e) = user_file.write(merged_file_string.as_bytes()) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("error writing merged logs string to user file for {user_id} -- {e}"));
    }

    return (StatusCode::OK, String::new());
}
