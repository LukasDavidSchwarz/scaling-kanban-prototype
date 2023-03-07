use axum::extract::{ConnectInfo, Path};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    routing::get,
    Json, Router, ServiceExt,
};
use mongodb::{options::ClientOptions, Client};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct AppState {
    db: Client,
    frontend_app: String,
}

// TODO: use structopt for environment variable parsing
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanban_backend=trace,tower_http=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
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
        .route("/board/:board_id", get(board_handler))
        .route("/board/:board_id/ws", get(websocket_handler))
        .with_state(shared_state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::default().include_headers(true),
            ),
        );

    let address = env::var("BACKEND_ADDRESS")
        .expect("Environment variable `BACKEND_ADDRESS` not set!")
        .parse()
        .expect("Failed to parse `BACKEND_ADDRESS`!");

    println!("Listening on http://{}...", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    println!("Server stopped.");
    Ok(())
}

async fn index_handler() -> &'static str {
    println!("Sending greetings from index...");
    "Hello, please navigate to /board/0"
}

async fn board_handler(
    State(state): State<Arc<AppState>>,
    Path(_board_id): Path<u128>,
) -> Html<String> {
    println!("Sending frontend...");
    Html(state.frontend_app.clone())
}

async fn websocket_handler(
    socket_upgrade: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("Received websocket request from {address}.");
    socket_upgrade.on_upgrade(move |socket| handle_websocket(socket, address))
}

async fn handle_websocket(mut socket: WebSocket, address: SocketAddr) {
    println!("Upgraded websocket request from {address}");

    match socket.send(Message::Ping(vec![1])).await {
        Ok(()) => println!("Pinged {address}..."),
        Err(err) => {
            println!("Failed to ping {address}: {err}");
            return;
        }
    }

    match socket.send(Message::Text("Hello World!".to_string())).await {
        Ok(()) => println!("Sent greetings to frontend!"),
        Err(err) => println!("Failed to send greetings to frontend: {err}"),
    }
    socket.close().await.ok();
}
