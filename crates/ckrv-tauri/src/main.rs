//! ckrv-tauri - Chakravarti Desktop Application
//!
//! This crate provides the Tauri v2 desktop wrapper for the Chakravarti CLI.
//! It reuses the React frontend from ckrv-ui and the transport handlers from
//! ckrv-transport to provide a native desktop experience.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

// ============================================================
// Imports
// ============================================================

use ckrv_transport::AppState;
use commands::terminal::TerminalSessions;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ============================================================
// Application
// ============================================================

/// Shared application state type for Tauri commands.
pub type SharedState = Arc<RwLock<AppState>>;

/// Application entry point.
fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ckrv_tauri=debug,ckrv_transport=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Chakravarti desktop app");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_pty::init()) // PTY for interactive terminals
        .plugin(tauri_plugin_process::init()) // Process for app restart
        .plugin(tauri_plugin_updater::Builder::new().build()) // Auto-update from GitHub Releases
        .setup(|app| {
            // Initialize app state - load project root from saved config or use cwd as fallback
            let project_root = commands::project::TauriConfig::load()
                .project_root
                .filter(|p| p.exists())
                .unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| {
                        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                    })
                });

            tracing::info!("Initializing with project root: {:?}", project_root);

            let state = AppState::new(project_root);
            let shared_state: SharedState = Arc::new(RwLock::new(state));
            app.manage(shared_state);

            // Initialize terminal sessions state (using parking_lot::Mutex for sync access)
            let terminal_sessions: TerminalSessions =
                Arc::new(parking_lot::Mutex::new(HashMap::new()));
            app.manage(terminal_sessions);

            // Open DevTools in debug builds
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            // Background update check on startup
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Delay to let the app finish initializing
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                match handle.updater() {
                    Ok(updater) => match updater.check().await {
                        Ok(Some(update)) => {
                            tracing::info!("Update available: v{}", update.version);
                            let _ = handle.emit(
                                "update-available",
                                serde_json::json!({
                                    "version": update.version,
                                    "body": update.body,
                                }),
                            );
                        }
                        Ok(None) => {
                            tracing::debug!("App is up to date");
                        }
                        Err(e) => {
                            tracing::warn!("Failed to check for updates: {}", e);
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Updater not available: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Status commands
            commands::status::get_status,
            commands::status::check_docker,
            commands::status::get_cloud_status,
            // Agent commands
            commands::agents::list_agents,
            commands::agents::get_openrouter_models,
            commands::agents::upsert_agent,
            commands::agents::delete_agent,
            commands::agents::set_default_agent,
            commands::agents::set_qa_agent,
            commands::agents::set_test_writer_agent,
            commands::agents::test_agent,
            commands::agents::get_kilo_models,
            commands::agents::get_glm_models,
            // Spec commands
            commands::specs::list_specs,
            commands::specs::get_spec,
            commands::specs::create_spec,
            commands::specs::update_spec,
            commands::specs::delete_spec,
            commands::specs::validate_spec,
            commands::specs::generate_design,
            commands::specs::generate_tasks,
            // Plan commands
            commands::plans::list_plans,
            commands::plans::get_plan,
            commands::plans::save_plan,
            commands::plans::delete_plan,
            // Diff commands
            commands::diff::get_branches,
            commands::diff::get_default_branch,
            commands::diff::get_diff,
            // History commands
            commands::history::list_history,
            commands::history::get_run,
            commands::history::create_run,
            commands::history::update_run,
            commands::history::delete_run,
            // QA commands
            commands::qa::get_qa_agent,
            commands::qa::run_review,
            commands::qa::run_bugs,
            commands::qa::run_report,
            // Test commands
            commands::test::get_test_agent,
            commands::test::run_tests,
            commands::test::generate_tests,
            commands::test::plan_tests,
            commands::test::write_tests,
            commands::test::get_coverage,
            commands::test::fix_tests,
            commands::test::get_plan_status,
            commands::test::get_write_status,
            // CLI commands
            commands::cli::run_init,
            commands::cli::run_git_init,
            commands::cli::run_spec_new,
            commands::cli::run_spec_tasks,
            commands::cli::run_plan,
            commands::cli::run_execute,
            commands::cli::run_diff,
            commands::cli::run_verify,
            commands::cli::run_promote,
            commands::cli::run_fix,
            // Terminal commands
            commands::terminal::terminal_start,
            commands::terminal::terminal_stop,
            commands::terminal::terminal_write,
            commands::terminal::terminal_read,
            commands::terminal::terminal_is_running,
            commands::terminal::terminal_list,
            // Execution commands
            commands::execution::start_execution,
            commands::execution::stop_execution,
            commands::execution::get_execution_status,
            commands::execution::get_execution_logs,
            commands::execution::list_execution_branches,
            // Project commands
            commands::project::get_project_root,
            commands::project::set_project_root,
            commands::project::get_recent_projects,
            commands::project::open_project_dialog,
            // Update commands
            commands::update::check_for_updates,
            commands::update::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
