mod misc;

use crate::misc::ConnectionInfo;
use async_nats::{Client as NatsClient, Subscriber};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    extract::{ConnectInfo, Path},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use clap::Parser;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, event, info, instrument, trace, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Provides a websocket interface for the frontend to be updated about changes made by other users
pub struct Cli {
    #[arg(env)]
    /// The address at which this server should listen
    backend_address: SocketAddr,

    #[arg(env)]
    /// The connection url of the Publish/Subscriber server
    pubsub_connection_url: String,

    #[arg(env)]
    /// The path to the .html file that contains the frontend
    frontend_file: PathBuf,
}

pub struct AppState {
    nats: NatsClient,
    frontend_app: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanban_backend=trace,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Starting server");

    let nats = async_nats::connect(cli.pubsub_connection_url).await?;
    let frontend_app = fs::read_to_string(cli.frontend_file)?;
    let shared_state = Arc::new(AppState { nats, frontend_app });

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

    info!("Listening on http://{}", cli.backend_address);
    info!("Open http://{}/board/0 to get started", cli.backend_address);
    axum::Server::bind(&cli.backend_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    info!("Server stopped");
    Ok(())
}

async fn index_handler() -> &'static str {
    trace!("Sending greetings from index...");
    "Hello, please navigate to /board/0"
}

async fn board_handler(
    State(state): State<Arc<AppState>>,
    Path(_board_id): Path<u128>,
) -> Html<String> {
    trace!("Sending frontend");
    Html(state.frontend_app.clone())
}

async fn websocket_handler(
    socket_upgrade: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(app_state): State<Arc<AppState>>,
    Path(board_id): Path<u128>,
) -> impl IntoResponse {
    let connection_info = ConnectionInfo { address, board_id };
    socket_upgrade
        .on_upgrade(move |socket| connection_state_machine(socket, app_state, connection_info))
}

#[instrument(skip(socket, app_state))]
async fn connection_state_machine(
    mut socket: WebSocket,
    app_state: Arc<AppState>,
    connection_info: ConnectionInfo,
) {
    info!("Upgraded websocket");
    match socket.send(Message::Ping(vec![1])).await {
        Ok(()) => event!(Level::TRACE, "Sending websocket ping..."),
        Err(err) => {
            info!(?err, "Failed to ping websocket");
            return;
        }
    }

    let nats = app_state.nats.clone();
    let nats_subject = connection_info.nats_subject();
    let subscriber = match nats.subscribe(nats_subject).await {
        Ok(subscriber) => subscriber,
        Err(err) => {
            error!(err, "Failed to subscribe to Nats ");
            if let Err(err) = socket.close().await {
                error!(?err, "Failed to close websocket");
            }
            return;
        }
    };

    let (socket_sender, socket_receiver) = socket.split();

    let mut nats_subscriber_task = tokio::spawn(handle_nats_subscriber(
        subscriber,
        socket_sender,
        connection_info.clone(),
    ));
    let mut socket_receiver_task = tokio::spawn(handle_socket_receiver(
        nats,
        socket_receiver,
        connection_info.clone(),
    ));

    tokio::select! {
        nats_subscriber_result = (&mut nats_subscriber_task) => {
            error!(?nats_subscriber_result, "Nats subscriber task was aborted unexpectedly");
            socket_receiver_task.abort();
        },
        socket_receiver_result = (&mut socket_receiver_task) => {
            info!(?socket_receiver_result, "Websocket receiver task was aborted");
            nats_subscriber_task.abort();
        }
    }

    info!("Destroyed connection context");
}

#[instrument(skip(subscriber, socket_sender))]
async fn handle_nats_subscriber(
    mut subscriber: Subscriber,
    mut socket_sender: SplitSink<WebSocket, Message>,
    connection: ConnectionInfo,
) -> Result<(), String> {
    loop {
        let nats_message = subscriber
            .next()
            .await
            .ok_or(format!("Nats subscriber for {connection} was closed."))?;

        let nats_message =
            String::from_utf8(nats_message.payload.to_vec()).map_err(|e| e.to_string())?;
        trace!(nats_message, "Received Nats message");
        socket_sender
            .send(Message::Text(nats_message))
            .await
            .map_err(|e| e.to_string())?;
    }
}

#[instrument(skip(nats, socket_receiver))]
async fn handle_socket_receiver(
    nats: NatsClient,
    mut socket_receiver: SplitStream<WebSocket>,
    connection: ConnectionInfo,
) -> Result<(), String> {
    loop {
        let socket_message = socket_receiver
            .next()
            .await
            .ok_or(format!("Websocket for {connection} was closed."))?
            .map_err(|e| e.to_string())?;

        if let Message::Text(socket_message) = socket_message {
            trace!(socket_message, "Received text message from websocket");

            trace!(socket_message, "Publishing message via Nats");
            let message = socket_message.into();
            nats.publish(connection.nats_subject(), message)
                .await
                .map_err(|e| e.to_string())?;
        } else if let Message::Binary(_) = socket_message {
            return Err(format!(
                "Received binary message via websocket {connection}. Closing connection..."
            ));
        } else if let Message::Close(_) = socket_message {
            return Err(format!(
                "Websocket closed gracefully by {connection}! Cleaning up connection..."
            ));
        } else {
            trace!(?socket_message, "Received message from websocket");
        }
    }
}
