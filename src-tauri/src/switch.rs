use crate::{cli::{CliAdapter,Credential,lock::OperationLock,parse_config,process::ensure_cli_stopped},storage::{Repository,atomic},model::{AppResult,AppError,AppSnapshot},transaction::{Transaction,FileChange},usage::{UsageClient,endpoints}};
pub struct SwitchService {pub repo:Repository,pub cli:CliAdapter}
impl SwitchService {
 pub fn new(repo:Repository,cli:CliAdapter)->Self{Self{repo,cli}}
 fn stopped(&self)->AppResult<()>{if self.cli.launch.is_some(){ensure_cli_stopped()?;}Ok(())}
 pub fn snapshot(&mut self)->AppResult<AppSnapshot>{self.repo.reload()?;let active=self.cli.read_active()?;let accounts=self.repo.views(active.as_ref());let active_state=if active.is_none(){"logged_out"}else if accounts.iter().any(|a|a.is_current){"known"}else{"unimported"};Ok(AppSnapshot{accounts,cli_version:Some(self.cli.version.clone()),cli_home:self.cli.home.to_string_lossy().into_owned(),active_state:active_state.into(),error:None})}
 pub fn import_current(&mut self,name:&str,replace:Option<&str>)->AppResult<()> {
  let _lock=OperationLock::acquire(&self.repo.root)?;self.repo.reload()?;
  let credential=self.cli.read_active()?.ok_or_else(||AppError::new("logged_out","CLI 尚未登录，请先添加账号或运行 kimi login"))?;
  let (region,key)=self.cli.region_and_key()?;
  let id=self.repo.import(credential,name,replace)?;self.repo.set_profile(&id,self.cli.config()?,region,key)
 }
 pub fn switch_to(&mut self,id:&str)->AppResult<()>{
  let _lock=OperationLock::acquire(&self.repo.root)?;self.stopped()?;self.repo.reload()?;
  Transaction::recover(&self.repo.root,&self.cli.home)?;
  let target=self.repo.account(id)?;let credential=self.repo.credential(id)?;
  if let Some(current)=self.cli.read_active()?{let current_id=self.repo.views(Some(&current)).into_iter().find(|v|v.is_current).map(|v|v.id);if let Some(current_id)=current_id{self.repo.save_credential(&current_id,&current)?;}}
  let target_credential=if let Some(current)=self.cli.read_active()?{if current.same_account(&credential){current}else{credential}}else{credential};
  let config=merge_config(&self.cli.config()?,&target.config,&target.region,&target.key)?;
  let credential_path=self.cli.home.join("credentials").join(format!("{}.json",crate::cli::safe_key(&target.key)?));
  let changes=vec![FileChange{path:credential_path,after:target_credential.bytes()},FileChange{path:self.cli.home.join("config.toml"),after:config.into_bytes()}];
  self.stopped()?;
  let tx=Transaction::prepare(&self.repo.root,&self.cli.home,changes)?;
  let _guards=guard_files(&[self.cli.credential_path()?,self.cli.home.join("config.toml")])?;
  self.stopped()?;
  tx.apply_with(atomic::write)
 }
 pub fn recover(&self)->AppResult<()>{let _lock=OperationLock::acquire(&self.repo.root)?;if self.repo.root.join("pending.dpapi").exists(){self.stopped()?;Transaction::recover(&self.repo.root,&self.cli.home)?;}Ok(())}
 pub async fn refresh_account(&mut self,id:&str)->AppResult<()> {
  let _lock=OperationLock::acquire(&self.repo.root)?;self.repo.reload()?;
  if self.repo.root.join("pending.dpapi").exists(){return Err(AppError::new("recovery_required","请先重启助手以恢复未完成切换"));}
  let account=self.repo.account(id)?;let mut credential=self.repo.credential(id)?;
  let active=self.cli.read_active()?;let is_active=active.as_ref().is_some_and(|a|a.same_account(&credential));
  if is_active{credential=active.expect("checked");self.repo.save_credential(id,&credential)?;}
  let client=UsageClient::new()?;
  let needs_refresh=credential.expires_at()<=chrono::Utc::now().timestamp() as f64+300.0;
  if is_active&&needs_refresh{self.stopped()?;}
  let _guards=if is_active&&needs_refresh{guard_files(&[self.cli.credential_path()?])?}else{vec![]};
  let result=client.fresh_credential(&credential,&account.region).await;
  let fresh=match result{Ok(c)=>c,Err(e)=>{self.repo.update_quota(id,None,Some(e.message.clone()))?;return Err(e);}};
  if fresh.bytes()!=credential.bytes(){
   self.repo.save_credential(id,&fresh)?;
   if is_active {
    self.stopped()?;
    let path=self.cli.credential_path()?;
    // A prepared transaction is necessary before the exclusive file guard.
    // The old handle denies concurrent readers; replacement file has new token.
    atomic::write(&path,&fresh.bytes())?;
   }
  }
  drop(_guards);
  match client.fetch_quota(&fresh,&account.region).await{Ok(q)=>self.repo.update_quota(id,Some(q),None),Err(e)=>{self.repo.update_quota(id,None,Some(e.message.clone()))?;Err(e)}}
 }
}
fn guard_files(paths:&[std::path::PathBuf])->AppResult<Vec<std::fs::File>>{
 let mut files=vec![];
 for path in paths {if path.is_file(){let mut options=std::fs::OpenOptions::new();options.read(true);#[cfg(windows)]{use std::os::windows::fs::OpenOptionsExt;options.share_mode(4);}files.push(options.open(path).map_err(|_|AppError::new("credential_busy","CLI 文件被其他进程占用，请关闭 CLI 后重试"))?);}}
 Ok(files)
}
pub fn merge_config(current:&str,profile:&str,region:&str,key:&str)->AppResult<String>{
 let (api,auth)=endpoints(region)?;let key=crate::cli::safe_key(key)?;
 let mut doc=parse_config(current)?;let source=parse_config(profile)?;
 if doc.get("providers").is_none(){doc["providers"]=toml_edit::Item::Table(toml_edit::Table::new());}
 if let Some(provider)=source.get("providers").and_then(|p|p.get("managed:kimi-code")){doc["providers"]["managed:kimi-code"]=provider.clone();}
 if doc["providers"].get("managed:kimi-code").is_none(){doc["providers"]["managed:kimi-code"]=toml_edit::Item::Table(toml_edit::Table::new());}
 let provider=&mut doc["providers"]["managed:kimi-code"];
 provider["type"]=toml_edit::value("kimi");provider["base_url"]=toml_edit::value(api);provider["api_key"]=toml_edit::value("");
 if let Some(table)=provider.as_table_like_mut(){table.remove("api_key_env");table.remove("env");}
 provider["oauth"]=provider_oauth(auth,&key);
 if source.get("models").is_some()&&doc.get("models").is_none(){doc["models"]=toml_edit::Item::Table(toml_edit::Table::new());}
 if let Some(models)=source.get("models").and_then(|m|m.as_table_like()){for (name,model) in models.iter(){if model.get("provider").and_then(|p|p.as_str())==Some("managed:kimi-code")&&doc.get("models").and_then(|m|m.get(name)).is_none(){doc["models"][name]=model.clone();}}}
 if doc.get("default_model").is_none(){if let Some(model)=source.get("default_model"){doc["default_model"]=model.clone();}else{return Err(AppError::new("config_missing","缺少 CLI 模型配置，请通过“添加账号”完成官方登录后再切换"));}}
 for service in ["moonshot_search","moonshot_fetch"]{
  if doc.get("services").and_then(|v|v.get(service)).and_then(|v|v.get("oauth")).is_some(){doc["services"][service]["oauth"]=provider_oauth(auth,&key);doc["services"][service]["base_url"]=toml_edit::value(format!("{api}/{}",if service=="moonshot_search"{"search"}else{"fetch"}));}
 }
 Ok(doc.to_string())
}
fn provider_oauth(auth:&str,key:&str)->toml_edit::Item{let mut t=toml_edit::Table::new();t["storage"]=toml_edit::value("file");t["key"]=toml_edit::value(format!("oauth/{key}"));t["oauth_host"]=toml_edit::value(auth);toml_edit::Item::Table(t)}
