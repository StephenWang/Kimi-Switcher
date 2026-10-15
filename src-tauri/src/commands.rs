use crate::{
    cli::{lock::OperationLock, CliAdapter},
    login::LoginManager,
    model::{AppError, AppResult, AppSnapshot, LoginStatus},
    storage::Repository,
    switch::SwitchService,
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::{Manager, State};
pub struct Core {
    service: Option<SwitchService>,
    error: Option<AppError>,
}
pub struct AppState {
    pub core: tokio::sync::Mutex<Core>,
    pub login: Arc<LoginManager>,
    pub root: PathBuf,
}
impl AppState {
    pub fn initialize(root: PathBuf) -> Self {
        let result = (|| {
            let cli = CliAdapter::detect()?;
            let repo = Repository::open(root.clone())?;
            let mut service = SwitchService::new(repo, cli);
            service.recover()?;
            Ok(service)
        })();
        let (service, error) = match result {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e)),
        };
        Self {
            core: tokio::sync::Mutex::new(Core { service, error }),
            login: Arc::new(LoginManager::default()),
            root,
        }
    }
}
impl Core {
    fn service(&mut self) -> AppResult<&mut SwitchService> {
        self.service.as_mut().ok_or_else(|| {
            self.error
                .clone()
                .unwrap_or_else(|| AppError::new("init", "初始化失败"))
        })
    }
}
#[tauri::command]
pub async fn get_snapshot(state: State<'_, AppState>) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    if core.service.is_none() {
        return Ok(AppSnapshot {
            accounts: vec![],
            cli_version: None,
            cli_home: String::new(),
            active_state: "unavailable".into(),
            error: core.error.as_ref().map(|e| e.message.clone()),
        });
    }
    core.service()?.snapshot()
}
#[tauri::command]
pub async fn import_current(
    state: State<'_, AppState>,
    name: String,
    replace_id: Option<String>,
) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    svc.import_current(&name, replace_id.as_deref())?;
    svc.snapshot()
}
#[tauri::command]
pub async fn rename_account(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    let _lock = OperationLock::acquire(&svc.repo.root)?;
    svc.ensure_ready()?;
    svc.repo.reload()?;
    svc.repo.rename(&id, &name)?;
    svc.snapshot()
}
#[tauri::command]
pub async fn delete_account(state: State<'_, AppState>, id: String) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    let _lock = OperationLock::acquire(&svc.repo.root)?;
    svc.ensure_ready()?;
    svc.repo.reload()?;
    svc.repo.remove(&id)?;
    svc.snapshot()
}
#[tauri::command]
pub async fn switch_account(state: State<'_, AppState>, id: String) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    svc.switch_to(&id)?;
    svc.snapshot()
}
#[tauri::command]
pub async fn refresh_account(state: State<'_, AppState>, id: String) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    svc.refresh_account(&id).await?;
    svc.snapshot()
}
#[tauri::command]
pub async fn refresh_all(state: State<'_, AppState>) -> AppResult<AppSnapshot> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    let ids = svc
        .snapshot()?
        .accounts
        .into_iter()
        .map(|a| a.id)
        .collect::<Vec<_>>();
    let total = ids.len();
    let mut failures = vec![];
    for id in ids {
        if let Err(error) = svc.refresh_account(&id).await {
            failures.push(error);
        }
    }
    crate::usage::summarize_refresh(total, &failures)?;
    svc.snapshot()
}
#[tauri::command]
pub async fn start_login(
    state: State<'_, AppState>,
    region: String,
    name: String,
) -> AppResult<LoginStatus> {
    let mut core = state.core.lock().await;
    let svc = core.service()?;
    if svc
        .snapshot()?
        .accounts
        .iter()
        .any(|a| a.name == name.trim())
    {
        return Err(AppError::new("name", "账号名称已存在，请使用其他名称"));
    }
    let launch = svc
        .cli
        .launch
        .clone()
        .ok_or_else(|| AppError::new("cli_missing", "找不到官方 CLI 启动入口"))?;
    state.login.start(
        launch,
        region,
        name,
        state.root.clone(),
        Duration::from_secs(600),
    )
}
#[tauri::command]
pub async fn get_login_status(
    state: State<'_, AppState>,
    task_id: String,
) -> AppResult<LoginStatus> {
    state.login.status(&task_id)
}
#[tauri::command]
pub async fn cancel_login(state: State<'_, AppState>, task_id: String) -> AppResult<LoginStatus> {
    state.login.cancel(&task_id)
}
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let root = app.path().app_data_dir()?;
    app.manage(AppState::initialize(root));
    Ok(())
}
