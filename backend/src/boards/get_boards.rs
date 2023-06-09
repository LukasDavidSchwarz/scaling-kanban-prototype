use crate::{AppState, Board};

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use futures_util::TryStreamExt;
use std::sync::Arc;
use tracing::{error, instrument};

#[instrument(skip(app_state))]
pub async fn handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<Board>>, StatusCode> {
    let mut boards = app_state
        .boards_table
        .find(None, None)
        .await
        .map_err(|error| {
            error!(?error, "Failed to query boards");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| {
            error!(?error, "Failed to collect boards");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    boards.sort_unstable_by_key(|board| board.created_at);
    Ok(Json(boards))
}
