use std::{collections::HashMap, env, fs::{self, File}, io::{Read, Seek, Write}, path, sync::Arc, usize};

use axum::{extract::{Query, State}, http::StatusCode, routing::{get, post}, Router};
use log::merge_logs;
use log_file::{log_file_string_to_logs, logs_to_file_string, tail_file};

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

    let default_offset = String::from("0");
    let offset = params.get("offset").unwrap_or(&default_offset);
    let offset: usize = match offset.parse() {
        Ok(val) => val,
        Err(e) => return (StatusCode::OK, format!("error parsing limit query param: {e}")),
    };

    let f = match File::open(user_id) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error opening file for {user_id}: {e}")),
    };

    let lines = match tail_file(&f, limit, offset) {
        Ok(val) => val,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error tailing file for {user_id}: {e}")),
    };

    let mut logs: Vec<log::Log> = Vec::new();
    for line in lines.iter() {
        match log::Log::from_string(line) {
            Ok(new_log) => logs.push(new_log),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error reading log file: {e}")),
        } 
    }

    return (StatusCode::OK, lines.join("\n"));
}

async fn write_log_file(State(state): State<Arc<AppState>>, Query(params): Query<HashMap<String, String>>, body: String) -> (StatusCode, String) {
    let user_id = match params.get("user_id") {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, "no user_id provided in query string".to_string()),
    };

    let user_file_path = std::path::Path::new(&state.log_dir).join(user_id);
    
    let mut user_file = match fs::File::open(&user_file_path) {
        Ok(file) => file,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("error opening log file for user {user_id} -- {e}")),
    };

    let new_logs = match log_file::log_file_string_to_logs(&body) {
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()),
        Ok(logs) => logs,
    };

    let mut current_logs_file_string = String::new();
    if let Err(e) = user_file.read_to_string(&mut current_logs_file_string) {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("error reading log file for user {user_id} -- {e}"));
    }

    let current_logs = match log_file_string_to_logs(&current_logs_file_string) {
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
