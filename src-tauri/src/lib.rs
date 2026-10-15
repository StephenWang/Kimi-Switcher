pub mod cli;
pub mod model;
pub mod storage;
pub mod usage;
pub mod oauth;
pub mod switch;
pub mod transaction;
pub mod login;
#[cfg(feature = "desktop")]
pub fn run() { tauri::Builder::default().run(tauri::generate_context!()).expect("无法启动桌面应用"); }
