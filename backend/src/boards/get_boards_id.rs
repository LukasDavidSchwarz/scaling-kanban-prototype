use crate::{AppState, Board};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use mongodb::bson::{doc, Uuid};
use std::sync::Arc;
use tracing::{error, instrument};

#[instrument(skip(app_state))]
pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Path(board_id): Path<Uuid>,
) -> Result<Json<Board>, StatusCode> {
    let board = app_state
        .boards_table
        .find_one(doc! { "id": board_id }, None)
        .await
        .map_err(|error| {
            error!(?error, "Failed to query board");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match board {
        Some(board) => Ok(Json(board)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
