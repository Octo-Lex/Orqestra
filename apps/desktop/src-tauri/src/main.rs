#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::roadmap::index_roadmap_cmd,
            commands::roadmap::get_task,
            commands::git::git_pull_roadmap,
            commands::git::git_push_roadmap,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
