mod commands;
mod state;

use commands::{get_status, start_stream, stop_stream};
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_stream, stop_stream, get_status])
        .run(tauri::generate_context!())
        .expect("Erreur lors du démarrage de Tauri");
}
