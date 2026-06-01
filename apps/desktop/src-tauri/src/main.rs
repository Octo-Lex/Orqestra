#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(commands::graph::GraphState {
            store: tokio::sync::Mutex::new(None),
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
