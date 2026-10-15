use crate::{
    cli::{read_optional, CliAdapter, Credential},
    model::{AppError, AppResult},
    storage::{atomic, crypto, Repository},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Pending {
    home: std::path::PathBuf,
    key: String,
    id: String,
    before: Vec<u8>,
    after: Vec<u8>,
}
pub fn stage(
    repo: &Repository,
    cli: &CliAdapter,
    id: &str,
    before: &Credential,
    after: &Credential,
) -> AppResult<()> {
    if repo.root.join("refresh.dpapi").exists() {
        return Err(AppError::new(
            "refresh_pending",
            "存在未完成的令牌更新，请重启助手",
        ));
    }
    let pending = Pending {
        home: cli.home.clone(),
        key: cli.region_and_key()?.1,
        id: id.into(),
        before: before.bytes(),
        after: after.bytes(),
    };
    atomic::write(
        &repo.root.join("refresh.dpapi"),
        &crypto::protect(&serde_json::to_vec(&pending).expect("JSON value"))?,
    )
}
pub fn complete(repo: &mut Repository, cli: &CliAdapter) -> AppResult<()> {
    let path = repo.root.join("refresh.dpapi");
    let Some(bytes) = read_optional(&path)? else {
        return Ok(());
    };
    let pending: Pending = serde_json::from_slice(&crypto::unprotect(&bytes)?)
        .map_err(|_| AppError::new("refresh_recovery", "令牌恢复记录损坏，请保留数据"))?;
    if pending.home != cli.home || pending.key != cli.region_and_key()?.1 {
        return Err(AppError::new(
            "refresh_changed",
            "CLI 登录位置已变化，已停止恢复令牌",
        ));
    }
    let before = Credential::parse(&pending.before)?;
    let after = Credential::parse(&pending.after)?;
    let current = cli
        .read_active()?
        .ok_or_else(|| AppError::new("refresh_changed", "CLI 已退出登录，已停止恢复令牌"))?;
    if current.bytes() != before.bytes() && current.bytes() != after.bytes() {
        return Err(AppError::new(
            "refresh_changed",
            "CLI 凭据已被外部更新，未覆盖现有登录。请保留恢复记录并查看恢复说明",
        ));
    }
    // Refresh token rotation cannot be rolled back: recover forward to the new token.
    repo.save_credential(&pending.id, &after)?;
    let _guard = crate::switch::guard_files(&[cli.credential_path()?])?;
    atomic::write(&cli.credential_path()?, &after.bytes())?;
    std::fs::remove_file(path)?;
    Ok(())
}
