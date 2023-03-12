use crate::AppState;

use async_nats::{Client as NatsClient, Subscriber};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    extract::{ConnectInfo, Path},
    response::IntoResponse,
};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, instrument, trace};

#[derive(Clone, Copy, Debug)]
struct ConnectionInfo {
    pub address: SocketAddr,
    pub board_id: u128,
}

impl ConnectionInfo {
    fn nats_subject(&self) -> String {
        format!("board.{}", self.board_id)
    }
}

impl Display for ConnectionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[board.{}@{}]", self.board_id, self.address)
    }
}

pub async fn websocket_handler(
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
        Ok(()) => trace!("Sending websocket ping..."),
        Err(err) => {
            info!(?err, "Failed to ping websocket.");

            socket.close().await.ok();
            info!("Destroyed connection context");
            return;
        }
    }

    let nats = app_state.nats.clone();
    let nats_subject = connection_info.nats_subject();
    let subscriber = match nats.subscribe(nats_subject).await {
        Ok(subscriber) => subscriber,
        Err(err) => {
            error!(err, "Failed to subscribe to Nats");
            if let Err(err) = socket.close().await {
                error!(?err, "Failed to close websocket");
            }
            info!("Destroyed connection context");
            return;
        }
    };

    let (socket_sender, socket_receiver) = socket.split();

    let mut nats_subscriber_task = tokio::spawn(handle_nats_subscriber(
        subscriber,
        socket_sender,
        connection_info,
    ));
    let mut socket_receiver_task = tokio::spawn(handle_socket_receiver(
        nats,
        socket_receiver,
        connection_info,
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