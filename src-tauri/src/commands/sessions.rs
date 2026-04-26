use tauri::{ipc::Channel, State};

use crate::{
    app_state::AppState,
    error::AppError,
    events::SessionIndexEvent,
    sessions::indexer::{self, SessionIndexSummary},
};

#[tauri::command]
pub async fn index_sessions(
    state: State<'_, AppState>,
    on_event: Channel<SessionIndexEvent>,
) -> Result<SessionIndexSummary, AppError> {
    index_sessions_for_app(&state, move |event| {
        on_event.send(event).map_err(AppError::from)
    })
    .await
}

pub async fn index_sessions_for_app(
    state: &AppState,
    on_event: impl Fn(SessionIndexEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<SessionIndexSummary, AppError> {
    indexer::index_session_roots(state.pool.clone(), state.home_dir.clone(), on_event).await
}
