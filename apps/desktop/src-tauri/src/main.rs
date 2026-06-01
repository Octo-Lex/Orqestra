#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(commands::graph::GraphState {
            store: tokio::sync::Mutex::new(None),
        })
        .manage(commands::sync::SyncState {
            engine: Mutex::new(None),
            token_manager: Mutex::new(
                loro_engine::sync::TokenManager::new("default-master-token")
            ),
        })
        .invoke_handler(tauri::generate_handler![
            commands::roadmap::index_roadmap_cmd,
            commands::roadmap::get_task,
            commands::roadmap::update_task_status_cmd,
            commands::git::git_pull_roadmap,
            commands::git::git_push_roadmap,
            commands::commit::semantic_commit_cmd,
            commands::commit::backfill_cmd,
            commands::agents::read_file_cmd,
            commands::agents::write_file_cmd,
            commands::agents::run_agent_cmd,
            commands::agents::list_workspaces_cmd,
            commands::graph::index_graph_cmd,
            commands::graph::query_graph_cmd,
            commands::graph::query_history_cmd,
            commands::graph::read_trace_cmd,
            commands::graph::read_commit_stub_cmd,
            commands::sync::init_sync_cmd,
            commands::sync::open_crdt_doc_cmd,
            commands::sync::set_crdt_field_cmd,
            commands::sync::get_crdt_field_cmd,
            commands::sync::get_all_fields_cmd,
            commands::sync::export_delta_cmd,
            commands::sync::import_delta_cmd,
            commands::sync::load_markdown_cmd,
            commands::sync::export_markdown_cmd,
            commands::sync::save_snapshot_cmd,
            commands::sync::sync_status_cmd,
            commands::sync::generate_token_cmd,
            commands::sync::validate_token_cmd,
            commands::credentials::save_github_token_cmd,
            commands::credentials::get_github_token_cmd,
            commands::credentials::get_github_token_status_cmd,
            commands::credentials::delete_github_token_cmd,
            commands::credentials::migrate_github_token_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
