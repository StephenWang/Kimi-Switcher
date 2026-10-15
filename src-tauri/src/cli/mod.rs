pub mod lock;
pub mod process;
use crate::model::{AppError, AppResult};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Clone)]
pub struct Credential {
    pub(crate) wire: Value,
}
impl Credential {
    pub fn parse(bytes: &[u8]) -> AppResult<Self> {
        let wire: Value = serde_json::from_slice(bytes)
            .map_err(|_| AppError::new("credential", "凭据格式无效，请重新登录"))?;
        if !wire.is_object()
            || !["access_token", "refresh_token"].iter().all(|k| {
                wire[*k]
                    .as_str()
                    .is_some_and(|s| !s.is_empty() && s.len() < 65536)
            })
            || wire["expires_at"].as_f64().is_none()
        {
            return Err(AppError::new(
                "credential",
                "不支持的凭据或登录已失效，请重新登录",
            ));
        }
        Ok(Self { wire })
    }
    pub fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(&self.wire).expect("JSON value")
    }
    pub fn access(&self) -> &str {
        self.wire["access_token"].as_str().unwrap_or_default()
    }
    pub fn refresh(&self) -> &str {
        self.wire["refresh_token"].as_str().unwrap_or_default()
    }
    pub fn expires_at(&self) -> f64 {
        self.wire["expires_at"].as_f64().unwrap_or_default()
    }
    pub fn identity(&self) -> Option<String> {
        let payload = self.access().split('.').nth(1)?;
        let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let claims: Value = serde_json::from_slice(&decoded).ok()?;
        claims
            .get("user_id")
            .or_else(|| claims.get("sub"))?
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    }
    pub fn device_id(&self) -> Option<String> {
        let claims: Value = serde_json::from_slice(
            &URL_SAFE_NO_PAD
                .decode(self.access().split('.').nth(1)?)
                .ok()?,
        )
        .ok()?;
        claims["device_id"].as_str().map(str::to_owned)
    }
    pub fn same_account(&self, other: &Self) -> bool {
        match (self.identity(), other.identity()) {
            (Some(a), Some(b)) => a == b,
            _ => self.refresh() == other.refresh(),
        }
    }
}
pub fn resolve_home(user: PathBuf, override_home: Option<PathBuf>) -> PathBuf {
    override_home
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| user.join(".kimi-code"))
}
pub fn supports_version(version: &str) -> bool {
    version.trim() == "2.1.1"
}
pub fn parse_config(text: &str) -> AppResult<toml_edit::DocumentMut> {
    text.parse()
        .map_err(|_| AppError::new("config", "CLI 配置不是有效 TOML，已停止操作"))
}
pub fn safe_key(key: &str) -> AppResult<String> {
    let name = key.strip_prefix("oauth/").unwrap_or(key);
    if name.starts_with("kimi-code") && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        Ok(name.into())
    } else {
        Err(AppError::new("credential_key", "不支持的 CLI 凭据引用"))
    }
}
#[derive(Clone)]
pub struct CliAdapter {
    pub home: PathBuf,
    pub version: String,
    pub launch: Option<Launch>,
}
#[derive(Clone)]
pub struct Launch {
    pub executable: PathBuf,
    pub prefix: Vec<String>,
}
impl Launch {
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.prefix);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        command
    }
}
impl CliAdapter {
    pub fn from_home(home: PathBuf, version: &str) -> AppResult<Self> {
        if !supports_version(version) {
            return Err(AppError::new(
                "version",
                "当前只支持已验证的 Kimi Code CLI 2.1.1",
            ));
        }
        Ok(Self {
            home,
            version: version.into(),
            launch: None,
        })
    }
    pub fn detect() -> AppResult<Self> {
        let user = std::env::var_os("USERPROFILE")
            .ok_or_else(|| AppError::new("home", "无法定位用户目录"))?;
        let home = resolve_home(
            PathBuf::from(user),
            std::env::var_os("KIMI_CODE_HOME").map(PathBuf::from),
        );
        let launch = detect_launch()?;
        let output = launch.command().arg("--version").output()?;
        let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let mut cli = Self::from_home(home, &version)?;
        cli.launch = Some(launch);
        Ok(cli)
    }
    pub fn config(&self) -> AppResult<String> {
        match fs::read_to_string(self.home.join("config.toml")) {
            Ok(s) => {
                parse_config(&s)?;
                Ok(s)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn region_and_key(&self) -> AppResult<(String, String)> {
        let doc = parse_config(&self.config()?)?;
        let provider = doc
            .get("providers")
            .and_then(|p| p.get("managed:kimi-code"));
        let key = provider
            .and_then(|p| p.get("oauth"))
            .and_then(|v| v.get("key"))
            .and_then(|v| v.as_str())
            .unwrap_or("oauth/kimi-code");
        let host = provider
            .and_then(|p| p.get("oauth"))
            .and_then(|v| v.get("oauth_host"))
            .and_then(|v| v.as_str());
        let base = provider
            .and_then(|p| p.get("base_url"))
            .and_then(|v| v.as_str());
        let region = if host == Some("https://auth.kimi.ai")
            || base == Some("https://api.kimi.ai/coding/v1")
        {
            "global"
        } else {
            "mainland-cn"
        };
        if host.is_some_and(|h| !matches!(h, "https://auth.kimi.ai" | "https://auth.kimi.com"))
            || base.is_some_and(|h| {
                !matches!(
                    h,
                    "https://api.kimi.ai/coding/v1" | "https://api.kimi.com/coding/v1"
                )
            })
        {
            return Err(AppError::new(
                "endpoint",
                "自定义 OAuth 地址不受支持，已停止发送凭据",
            ));
        }
        Ok((region.into(), safe_key(key)?))
    }
    pub fn credential_path(&self) -> AppResult<PathBuf> {
        Ok(self
            .home
            .join("credentials")
            .join(format!("{}.json", self.region_and_key()?.1)))
    }
    pub fn read_active(&self) -> AppResult<Option<Credential>> {
        match fs::read(self.credential_path()?) {
            Ok(b) => Ok(Some(Credential::parse(&b)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
pub fn detect_launch() -> AppResult<Launch> {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let entry =
            PathBuf::from(appdata).join("npm/node_modules/@moonshot-ai/kimi-code/dist/main.mjs");
        if entry.is_file() {
            if let Some(node) = find_executable("node.exe") {
                return Ok(Launch {
                    executable: node,
                    prefix: vec![entry.to_string_lossy().into_owned()],
                });
            }
        }
    }
    if let Some(executable) = find_executable("kimi.exe") {
        return Ok(Launch {
            executable,
            prefix: vec![],
        });
    }
    Err(AppError::new(
        "cli_missing",
        "未找到 Kimi Code CLI。请先安装 @moonshot-ai/kimi-code 2.1.1",
    ))
}
fn find_executable(name: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|p| p.is_absolute() && p != &cwd)
        .map(|p| p.join(name))
        .find(|p| p.is_file())
}
pub fn read_optional(path: &Path) -> AppResult<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
