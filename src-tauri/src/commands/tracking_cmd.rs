use tauri::State;
use crate::AppState;
use crate::models::tracking::AIPickTracking;

#[tauri::command]
pub async fn add_tracking_stock(
    state: State<'_, AppState>,
    code: String,
    name: String,
    added_price: f64,
    rating: String,
    reason: String,
    sector: String,
) -> Result<(), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tracking = AIPickTracking {
        code,
        name,
        added_date: today,
        added_price,
        rating,
        reason,
        sector,
        created_at: now,
    };
    state.db.add_tracking_stock(&tracking).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tracking_stock(
    state: State<'_, AppState>,
    code: String,
    added_date: String,
) -> Result<(), String> {
    state.db.remove_tracking_stock(&code, &added_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tracking_stocks(
    state: State<'_, AppState>,
) -> Result<Vec<AIPickTracking>, String> {
    state.db.get_tracking_stocks().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_tracking_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<(), String> {
    state.db.clear_tracking_by_date(&date).map_err(|e| e.to_string())
}
