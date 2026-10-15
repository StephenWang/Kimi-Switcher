use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}
pub type AppResult<T> = Result<T, AppError>;
impl AppError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for AppError {}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::new(
            "io",
            &format!(
                "文件访问失败（系统错误 {:?}），请检查权限或文件是否被占用",
                e.raw_os_error()
            ),
        )
    }
}
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub used_ratio: f64,
    pub reset_at: Option<String>,
}
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    pub week: Option<QuotaWindow>,
    pub five_hour: Option<QuotaWindow>,
    pub plan: Option<String>,
    pub fetched_at: Option<String>,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub is_current: bool,
    pub region: String,
    pub quota: QuotaSnapshot,
    pub status: String,
    pub error: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub accounts: Vec<AccountView>,
    pub cli_version: Option<String>,
    pub cli_home: String,
    pub active_state: String,
    pub error: Option<String>,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginStatus {
    pub task_id: String,
    pub stage: String,
    pub verification_url: Option<String>,
    pub error: Option<String>,
}
