pub mod atomic;
pub mod crypto;
use crate::{
    cli::Credential,
    model::{AccountView, AppError, AppResult, QuotaSnapshot},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredAccount {
    pub id: String,
    pub name: String,
    pub credential: Vec<u8>,
    pub config: String,
    pub region: String,
    pub key: String,
    pub quota: QuotaSnapshot,
    pub error: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Vault {
    version: u32,
    accounts: Vec<StoredAccount>,
}
pub struct Repository {
    pub root: PathBuf,
    data: Vault,
}
impl Repository {
    pub fn open(root: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&root)?;
        let data = if let Some(bytes) = crate::cli::read_optional(&root.join("accounts.dpapi"))? {
            let data: Vault = serde_json::from_slice(&crypto::unprotect(&bytes)?)
                .map_err(|_| AppError::new("vault", "账号仓库已损坏，未覆盖原文件"))?;
            if data.version != 1 {
                return Err(AppError::new("vault_version", "不支持的账号仓库版本"));
            }
            data
        } else {
            Vault {
                version: 1,
                accounts: vec![],
            }
        };
        Ok(Self { root, data })
    }
    pub fn reload(&mut self) -> AppResult<()> {
        self.data = Self::open(self.root.clone())?.data;
        Ok(())
    }
    pub fn import(
        &mut self,
        credential: Credential,
        name: &str,
        replace_id: Option<&str>,
    ) -> AppResult<String> {
        self.import_scoped(credential, name, replace_id, "mainland-cn")
    }
    pub fn import_scoped(
        &mut self,
        credential: Credential,
        name: &str,
        replace_id: Option<&str>,
        region: &str,
    ) -> AppResult<String> {
        crate::usage::endpoints(region)?;
        let existing = self
            .data
            .accounts
            .iter()
            .find(|a| {
                a.region == region
                    && Credential::parse(&a.credential).is_ok_and(|c| c.same_account(&credential))
            })
            .map(|a| a.id.clone());
        let target = replace_id
            .filter(|v| *v != "new")
            .map(str::to_owned)
            .or(existing);
        if let Some(id) = target {
            let idx = self.index(&id)?;
            self.data.accounts[idx].credential = credential.bytes();
            self.data.accounts[idx].error = None;
            self.save()?;
            return Ok(id);
        }
        if credential.identity().is_none()
            && !self.data.accounts.is_empty()
            && replace_id != Some("new")
        {
            return Err(AppError::new(
                "identity_choice",
                "无法确认账号身份。请选择更新已有账号或明确新建",
            ));
        }
        let name = self.validate_name(name, None)?;
        let id = uuid::Uuid::new_v4().to_string();
        self.data.accounts.push(StoredAccount {
            id: id.clone(),
            name,
            credential: credential.bytes(),
            config: String::new(),
            region: region.into(),
            key: "kimi-code".into(),
            quota: QuotaSnapshot::default(),
            error: None,
        });
        self.save()?;
        Ok(id)
    }
    fn index(&self, id: &str) -> AppResult<usize> {
        self.data
            .accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AppError::new("account_missing", "账号已被删除，请刷新列表"))
    }
    fn validate_name(&self, name: &str, except: Option<&str>) -> AppResult<String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 40 {
            return Err(AppError::new("name", "账号名称需要 1–40 个字符"));
        }
        if self
            .data
            .accounts
            .iter()
            .any(|a| Some(a.id.as_str()) != except && a.name == name)
        {
            return Err(AppError::new("name", "该名称已存在，请使用其他名称"));
        }
        Ok(name.into())
    }
    pub fn account(&self, id: &str) -> AppResult<StoredAccount> {
        Ok(self.data.accounts[self.index(id)?].clone())
    }
    pub fn credential(&self, id: &str) -> AppResult<Credential> {
        Credential::parse(&self.data.accounts[self.index(id)?].credential)
    }
    pub fn save_credential(&mut self, id: &str, credential: &Credential) -> AppResult<()> {
        let idx = self.index(id)?;
        self.data.accounts[idx].credential = credential.bytes();
        self.save()
    }
    pub fn set_profile(
        &mut self,
        id: &str,
        config: String,
        region: String,
        key: String,
    ) -> AppResult<()> {
        let idx = self.index(id)?;
        let a = &mut self.data.accounts[idx];
        a.config = config;
        a.region = region;
        a.key = key;
        self.save()
    }
    pub fn update_quota(
        &mut self,
        id: &str,
        quota: Option<QuotaSnapshot>,
        error: Option<String>,
    ) -> AppResult<()> {
        let idx = self.index(id)?;
        let a = &mut self.data.accounts[idx];
        if let Some(q) = quota {
            a.quota = q;
        }
        a.error = error;
        self.save()
    }
    pub fn rename(&mut self, id: &str, name: &str) -> AppResult<()> {
        let name = self.validate_name(name, Some(id))?;
        let idx = self.index(id)?;
        self.data.accounts[idx].name = name;
        self.save()
    }
    pub fn remove(&mut self, id: &str) -> AppResult<()> {
        let idx = self.index(id)?;
        self.data.accounts.remove(idx);
        self.save()
    }
    pub fn views(&self, active: Option<&Credential>) -> Vec<AccountView> {
        self.data
            .accounts
            .iter()
            .map(|a| AccountView {
                id: a.id.clone(),
                name: a.name.clone(),
                is_current: active.is_some_and(|v| {
                    Credential::parse(&a.credential).is_ok_and(|c| c.same_account(v))
                }),
                region: a.region.clone(),
                quota: a.quota.clone(),
                status: if a.error.is_some() { "error" } else { "ready" }.into(),
                error: a.error.clone(),
            })
            .collect()
    }
    pub fn save(&self) -> AppResult<()> {
        let bytes = serde_json::to_vec(&self.data)
            .map_err(|_| AppError::new("vault", "账号数据编码失败"))?;
        atomic::write(&self.root.join("accounts.dpapi"), &crypto::protect(&bytes)?)
    }
}
