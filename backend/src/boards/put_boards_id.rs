use crate::boards::BoardId;
use crate::{AppState, Board};

use axum::extract;
use axum::http::StatusCode;
use axum::Json;
use mongodb::bson;
use mongodb::bson::doc;

use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{error, instrument};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardDTO {
    pub name: String,
    pub lists: Vec<TaskListDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskListDTO {
    pub name: String,
    pub tasks: Vec<TaskDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskDTO {
    pub name: String,
}

#[instrument(skip(app_state))]
pub async fn handler(
    extract::State(app_state): extract::State<Arc<AppState>>,
    extract::Path(board_id): extract::Path<BoardId>,
    extract::Json(new_board): extract::Json<BoardDTO>,
) -> Result<Json<Board>, StatusCode> {
    let new_board = bson::to_bson(&new_board).map_err(|error| {
        error!(?error, "Failed to serialize board DTO as bson");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let update = doc! {
        "$set": new_board,
        "$inc": {"version": 1}
    };

    let mut update_options = FindOneAndUpdateOptions::default();
    update_options.return_document = Some(ReturnDocument::After);

    let updated_board = app_state
        .boards_table
        .find_one_and_update(doc! {"id": board_id}, update, update_options)
        .await
        .map_err(|error| {
            error!(?error, "Failed to update board");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let board_json = serde_json::to_string(&updated_board).map_err(|error| {
        error!(
            ?error,
            ?updated_board,
            "Failed to serialize updated board to json"
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    app_state
        .nats
        .publish(updated_board.pubsub_subject(), board_json.into())
        .await
        .map_err(|error| {
            error!(?error, "Failed to publish updated board to Nats");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(updated_board))
}
