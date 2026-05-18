pub mod commands;
mod models;
mod process;
mod state;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(std::sync::Mutex::new(state::AppState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::add_files,
            commands::remove_files,
            commands::get_files,
            commands::scan_metadata,
            commands::transcribe,
            commands::split_video,
            commands::organize,
            commands::execute_pipeline,
            commands::cancel,
            commands::check_binary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
