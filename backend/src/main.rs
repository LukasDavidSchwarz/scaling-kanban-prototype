use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;
use axum::{extract::State, routing::get, Json, Router, ServiceExt};
use mongodb::{options::ClientOptions, Client};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};

pub struct AppState {
    db: Client,
    frontend_app: String,
}

// TODO: use structopt for environment variable parsing
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server...");

    let db_uri =
        env::var("DB_CONNECTION_URI").expect("Environment variable ´DB_CONNECTION_URI´ not set!");
    let mut client_options = ClientOptions::parse(db_uri)
        .await
        .expect("Failed to parse environment variable ´DB_CONNECTION_URI´");
    client_options.app_name = Some("kanban backend".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(10));
    let db = Client::with_options(client_options)?;

    println!("Database names:");
    for db_name in db.list_database_names(None, None).await? {
        println!(" - {}", db_name);
    }

    let frontend_app = fs::read_to_string("static/board.html")?;

    let shared_state = Arc::new(AppState { db, frontend_app });
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/board/:board_id", get(serve_frontend))
        .with_state(shared_state);

    let port = env::var("BACKEND_PORT")
        .expect("Environment variable `BACKEND_PORT` not set!")
        .parse()
        .expect("Failed to parse `BACKEND_PORT`!");

    // TODO: parse ip address from environment
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Listening on http://{}...", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    println!("Server stopped.");
    Ok(())
}

async fn index_handler() -> &'static str {
    "Hello, World!"
}

async fn serve_frontend(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<u128>,
) -> Html<String> {
    Html(state.frontend_app.clone())
}
