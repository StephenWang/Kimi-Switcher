pub mod cli;
#[cfg(feature = "desktop")]
mod commands;
pub mod login;
pub mod model;
pub mod oauth;
pub mod refresh_recovery;
pub mod storage;
pub mod switch;
pub mod transaction;
pub mod usage;
#[cfg(feature = "desktop")]
pub fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .setup(commands::setup)
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::import_current,
            commands::rename_account,
            commands::delete_account,
            commands::switch_account,
            commands::refresh_account,
            commands::refresh_all,
            commands::start_login,
            commands::get_login_status,
            commands::cancel_login
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle().clone();
                let login = app.state::<commands::AppState>().login.clone();
                login.shutdown();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<commands::AppState>();
                    let guard = state.core.lock().await;
                    let cleaned = login.shutdown_and_wait().await.is_ok();
                    drop(guard);
                    drop(state);
                    if cleaned {
                        app.exit(0);
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("无法启动桌面应用");
}
