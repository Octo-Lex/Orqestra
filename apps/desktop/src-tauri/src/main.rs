#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

mod commands;
mod diagnostics;
mod security;

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
        // v1.0.3: Onboarding state
        .manage(commands::onboarding::OnboardingStateManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::roadmap::index_roadmap_cmd,
            commands::roadmap::get_task,
            commands::roadmap::update_task_status_cmd,
            commands::git::git_pull_roadmap,
            commands::git::git_push_roadmap,
            commands::git::git_status_cmd,
            commands::git::git_repository_snapshot_cmd,
            commands::git::git_recent_commits_cmd,
            commands::git::git_diff_stat_cmd,
            commands::git::prepare_semantic_commit_cmd,
            commands::git::git_provider_diagnostics_cmd,
            commands::git::git_recent_commits_with_provider_cmd,
            commands::git::git_diff_stat_with_provider_cmd,
            commands::commit::semantic_commit_cmd,
            commands::commit::backfill_cmd,
            commands::agents::read_file_cmd,
            commands::agents::write_file_cmd,
            commands::agents::run_agent_cmd,
            commands::agents::run_docs_agent_cmd,
            commands::agents::list_workspaces_cmd,
            // v1.0.2: Bugfix agent commands
            commands::agents::run_bugfix_agent_cmd,
            commands::agents::read_project_file_cmd,
            // v1.7.0: Patch governance
            commands::agents::apply_agent_patch_cmd,
            commands::agents::reject_agent_patch_cmd,
            // Graph commands
            commands::graph::index_graph_cmd,
            commands::graph::query_graph_cmd,
            commands::graph::query_history_cmd,
            commands::graph::read_trace_cmd,
            commands::graph::read_commit_stub_cmd,
            // Sync commands
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
            // v1.0.2: Secure credential commands (OS-keychain backed)
            commands::credentials::bootstrap_credential_vault_cmd,
            commands::credentials::save_github_token_cmd,
            commands::credentials::get_github_token_status_cmd,
            commands::credentials::delete_github_token_cmd,
            commands::credentials::test_github_connection_cmd,
            commands::credentials::migrate_legacy_credential_cmd,
            commands::credentials::rotate_vault_unlock_secret_cmd,
            // v1.0.3: Onboarding commands
            commands::onboarding::get_onboarding_state_cmd,
            commands::onboarding::set_onboarding_state_cmd,
            commands::onboarding::reset_onboarding_cmd,
            // v1.0.3: Project validation and sample project
            commands::project_validation::validate_project_cmd,
            commands::project_validation::create_sample_project_cmd,
            // v1.0.3: Environment readiness
            commands::readiness::get_readiness_cmd,
            // v1.0.3: Diagnostics and recovery
            commands::diagnostics::export_diagnostics_cmd,
            commands::diagnostics::get_recovery_advice_cmd,
            commands::diagnostics::get_structured_error_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
