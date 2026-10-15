use crate::{
    cli::{lock::OperationLock, CliAdapter, Launch},
    model::{AppError, AppResult, LoginStatus},
    storage::Repository,
};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::watch,
};
pub fn authorization_url(line: &str) -> Option<String> {
    for word in line.split_whitespace() {
        let text =
            word.trim_matches(|c: char| matches!(c, '"' | '\'' | '(' | ')' | '<' | '>' | ','));
        if let Ok(url) = reqwest::Url::parse(text) {
            if url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.port().is_none()
                && matches!(
                    url.host_str(),
                    Some(
                        "auth.kimi.com"
                            | "auth.kimi.ai"
                            | "www.kimi.com"
                            | "www.kimi.ai"
                            | "kimi.com"
                            | "kimi.ai"
                    )
                )
            {
                return Some(url.into());
            }
        }
    }
    None
}
pub fn login_arguments(region: &str) -> AppResult<Vec<String>> {
    crate::usage::endpoints(region)?;
    Ok(vec!["login".into(), "--region".into(), region.into()])
}
struct ActiveLogin {
    status: Arc<Mutex<LoginStatus>>,
    cancel: watch::Sender<bool>,
}
#[derive(Default)]
pub struct LoginManager {
    active: Mutex<Option<ActiveLogin>>,
}
fn update(status: &Arc<Mutex<LoginStatus>>, stage: &str, error: Option<String>) {
    if let Ok(mut s) = status.lock() {
        s.stage = stage.into();
        s.error = error;
    }
}
impl LoginManager {
    pub fn start(
        &self,
        launch: Launch,
        region: String,
        name: String,
        root: PathBuf,
        timeout: Duration,
    ) -> AppResult<LoginStatus> {
        login_arguments(&region)?;
        if name.trim().is_empty() || name.trim().chars().count() > 40 {
            return Err(AppError::new("name", "账号名称需要 1–40 个字符"));
        }
        let mut active = self
            .active
            .lock()
            .map_err(|_| AppError::new("login", "登录任务状态不可用"))?;
        if active.as_ref().is_some_and(|j| {
            j.status.lock().is_ok_and(|s| {
                matches!(
                    s.stage.as_str(),
                    "starting" | "waiting" | "cancelling" | "saving"
                )
            })
        }) {
            return Err(AppError::new("login_busy", "已有登录任务，请先完成或取消"));
        }
        let value = LoginStatus {
            task_id: uuid::Uuid::new_v4().to_string(),
            stage: "starting".into(),
            verification_url: None,
            error: None,
        };
        let status = Arc::new(Mutex::new(value.clone()));
        let (cancel, rx) = watch::channel(false);
        *active = Some(ActiveLogin {
            status: status.clone(),
            cancel,
        });
        tokio::spawn(async move {
            let result = run_login(launch, region, name, root, timeout, status.clone(), rx).await;
            if let Err(error) = result {
                update(
                    &status,
                    if error.code == "cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    },
                    Some(error.message),
                );
            } else {
                update(&status, "complete", None);
            }
        });
        Ok(value)
    }
    pub fn status(&self, id: &str) -> AppResult<LoginStatus> {
        let active = self
            .active
            .lock()
            .map_err(|_| AppError::new("login", "登录任务状态不可用"))?;
        let job = active
            .as_ref()
            .ok_or_else(|| AppError::new("login_missing", "登录任务不存在"))?;
        let status = job
            .status
            .lock()
            .map_err(|_| AppError::new("login", "登录任务状态不可用"))?
            .clone();
        if status.task_id != id {
            return Err(AppError::new("login_missing", "登录任务已更新"));
        }
        Ok(status)
    }
    pub fn cancel(&self, id: &str) -> AppResult<LoginStatus> {
        let status = self.status(id)?;
        let active = self
            .active
            .lock()
            .map_err(|_| AppError::new("login", "登录任务状态不可用"))?;
        if let Some(job) = active.as_ref() {
            let _ = job.cancel.send(true);
        }
        Ok(status)
    }
    pub fn shutdown(&self) {
        if let Ok(active) = self.active.lock() {
            if let Some(job) = active.as_ref() {
                let _ = job.cancel.send(true);
            }
        }
    }
    pub async fn shutdown_and_wait(&self) -> AppResult<()> {
        self.shutdown();
        for _ in 0..500 {
            let complete = {
                let active = self
                    .active
                    .lock()
                    .map_err(|_| AppError::new("login", "无法读取登录清理状态"))?;
                active.as_ref().is_none_or(|job| {
                    job.status.lock().is_ok_and(|s| {
                        matches!(s.stage.as_str(), "complete" | "failed" | "cancelled")
                    })
                })
            };
            if complete {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Err(AppError::new(
            "cleanup_pending",
            "登录进程仍在清理，请稍后再关闭助手",
        ))
    }
}
async fn run_login(
    launch: Launch,
    region: String,
    name: String,
    root: PathBuf,
    timeout: Duration,
    status: Arc<Mutex<LoginStatus>>,
    mut cancel: watch::Receiver<bool>,
) -> AppResult<()> {
    let tmp_root = root.join("login-tmp");
    std::fs::create_dir_all(&tmp_root)?;
    let dir = tempfile::tempdir_in(&tmp_root)?;
    let mut command = tokio::process::Command::from(launch.command());
    command
        .args(login_arguments(&region)?)
        .env("KIMI_CODE_HOME", dir.path())
        .env_remove("KIMI_CODE_OAUTH_HOST")
        .env_remove("KIMI_OAUTH_HOST")
        .env_remove("KIMI_CODE_BASE_URL")
        .env_remove("KIMI_CODE_CUSTOM_HEADERS")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|_| AppError::new("login_spawn", "无法启动官方 CLI 登录进程"))?;
    let mut stderr = BufReader::new(
        child
            .stderr
            .take()
            .ok_or_else(|| AppError::new("login_pipe", "无法读取登录状态"))?,
    )
    .lines();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);
    let mut stderr_open = true;
    let outcome = loop {
        tokio::select! {
         _=cancel.changed()=>{let _=child.kill().await;break Err(AppError::new("cancelled","登录已取消"));},
         _=&mut deadline=>{let _=child.kill().await;break Err(AppError::new("login_timeout","登录等待超时，请重新添加账号"));},
         exit=child.wait()=>{break match exit{Ok(exit) if exit.success()=>Ok(()),_=>Err(AppError::new("login_failed","官方登录未完成或授权被拒绝，请重试"))};},
         line=stderr.next_line(),if stderr_open=>{match line {Ok(Some(line))=>{if let Some(url)=authorization_url(&line){if let Ok(mut s)=status.lock(){s.stage="waiting".into();s.verification_url=Some(url);}}},Ok(None)=>{stderr_open=false;},Err(_)=>{let _=child.kill().await;break Err(AppError::new("login_output","无法读取登录进程状态"));}}},
        }
    };
    let result = match outcome {
        Ok(()) => {
            if *cancel.borrow() {
                Err(AppError::new("cancelled", "登录已取消"))
            } else {
                update(&status, "saving", None);
                (|| {
                    let cli = CliAdapter::from_home(dir.path().to_owned(), "2.1.1")?;
                    let credential = cli.read_active()?.ok_or_else(|| {
                        AppError::new("login_credential", "登录完成但未生成有效凭据")
                    })?;
                    let (detected_region, key) = cli.region_and_key()?;
                    let _lock = OperationLock::acquire(&root)?;
                    let mut repo = Repository::open(root.clone())?;
                    let id =
                        repo.import_scoped(credential, &name, Some("new"), &detected_region)?;
                    repo.set_profile(&id, cli.config()?, detected_region, key)
                })()
            }
        }
        Err(e) => Err(e),
    };
    dir.close().map_err(|_| {
        AppError::new(
            "login_cleanup",
            "登录临时文件清理失败，请关闭助手后检查账号数据目录下 login-tmp",
        )
    })?;
    result
}
