use crate::{app_state::AppState, error::AppError};

pub async fn request_tray_refresh(state: &AppState) -> Result<(), AppError> {
    state.request_tray_refresh();
    Ok(())
}
