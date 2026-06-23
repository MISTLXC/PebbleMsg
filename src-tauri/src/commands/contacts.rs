use crate::state::AppState;
use pebble_core::{KnownContact, PebbleError};
use tauri::State;

#[tauri::command]
pub async fn search_contacts(
    state: State<'_, AppState>,
    account_id: String,
    query: String,
    limit: Option<i64>,
) -> std::result::Result<Vec<KnownContact>, PebbleError> {
    let limit = limit.unwrap_or(20);
    state.store.list_known_contacts(&account_id, &query, limit)
}
