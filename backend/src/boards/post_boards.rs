use crate::{AppState, Board};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardDTO {
    name: String,
}

pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Json(new_board_dto): Json<BoardDTO>,
) -> Result<Json<Board>, StatusCode> {
    let new_board = Board::new(new_board_dto.name);

    let board_id = app_state
        .boards_table
        .insert_one(new_board, None)
        .await
        .map_err(|error| {
            error!(?error, "Failed to create new board");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .inserted_id;

    let new_board = app_state
        .boards_table
        .find_one(doc! {"_id": &board_id}, None)
        .await
        .map_err(|error| {
            error!(?error, ?board_id, "Failed to find newly created board",);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            error!(?board_id, "Failed to find newly created board");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(new_board))
}
