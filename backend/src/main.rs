use async_nats::Client as NatsClient;
use axum::extract::{ConnectInfo, Path};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    routing::get,
    Error as AxumError, Json, Router, ServiceExt,
};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use mongodb::{options::ClientOptions, Client as MongoClient, Client};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: Use HashMap to map board id to all sockets listening for changes of that board
pub struct AppState {
    mongo: MongoClient,
    nats: NatsClient,
    frontend_app: String,
}

// TODO: use structopt for environment variable parsing
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanban_backend=trace,tower_http=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    println!("Starting server...");

    let mongo = connect_to_mongo().await?;
    let nats_connection_url = env::var("PUBSUB_CONNECTION_URL")
        .expect("Environment variable ´PUBSUB_CONNECTION_URL´ not set!");
    let nats = async_nats::connect(nats_connection_url).await?;
    let frontend_app = fs::read_to_string("static/board.html")?;
    let shared_state = Arc::new(AppState {
        mongo,
        nats,
        frontend_app,
    });

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

async fn connect_to_mongo() -> Result<Client, Box<dyn Error>> {
    let db_uri =
        env::var("DB_CONNECTION_URL").expect("Environment variable ´DB_CONNECTION_URL´ not set!");
    let mut client_options = ClientOptions::parse(db_uri)
        .await
        .expect("Failed to parse environment variable ´DB_CONNECTION_URL´");
    client_options.app_name = Some("kanban backend".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(10));
    let db = MongoClient::with_options(client_options)?;

    println!("Database names:");
    for db_name in db.list_database_names(None, None).await? {
        println!(" - {}", db_name);
    }
    Ok(db)
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
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<u128>,
) -> impl IntoResponse {
    println!("Received websocket request from {address}.");
    socket_upgrade.on_upgrade(move |socket| handle_websocket(socket, state, address, board_id))
}

async fn handle_websocket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    address: SocketAddr,
    board_id: u128,
) {
    println!("Upgraded websocket request from {address} at board {board_id}");

    match socket.send(Message::Ping(vec![1])).await {
        Ok(()) => println!("Pinged {address}..."),
        Err(err) => {
            println!("Failed to ping {address}: {err}");
            return;
        }
    }

    let (mut socket_sender, mut socket_receiver) = socket.split();

    let nats_subject = format!("board.{board_id}");
    let mut subscriber = state.nats.subscribe(nats_subject.clone()).await.unwrap();
    let subscriber_handle = tokio::spawn(handle_subscriber(subscriber, socket_sender));

    while let Some(message) = socket_receiver.next().await {
        if let Message::Text(message) = message.unwrap() {
            println!("Received message from websocket: {message}");
            state
                .nats
                .publish(nats_subject.clone(), message.into())
                .await
                .unwrap();
        }
    }
    println!("Websocket connection with {address} was closed.");
}

async fn handle_subscriber(
    mut subscriber: async_nats::Subscriber,
    mut socket_sender: SplitSink<WebSocket, Message>,
) {
    while let Some(message) = subscriber.next().await {
        let message = String::from_utf8(message.payload.to_vec()).unwrap();
        println!("Received message from Nats: {message}");
        socket_sender.send(Message::Text(message)).await.unwrap();
    }
}

async fn echo_message_back(
    socket: &mut WebSocket,
    message: Result<Message, AxumError>,
    address: &SocketAddr,
) -> Result<(), Box<dyn Error>> {
    match message.map_err(|err| format!("{err}"))? {
        Message::Text(t) => socket
            .send(Message::Text(format!("Hello {t}!")))
            .await
            .map_err(|err| err.into()),
        Message::Binary(b) => socket
            .send(Message::Binary(b))
            .await
            .map_err(|err| err.into()),
        Message::Close(close_frame) => Err(format!(
            "Websocket connection with {address} was closed. Close frame: {:?}",
            close_frame
        )
        .into()),
        Message::Ping(_) => Ok(()),
        Message::Pong(_) => Ok(()),
    }
}
