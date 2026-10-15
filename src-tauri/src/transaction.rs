use crate::{model::{AppResult,AppError},storage::{atomic,crypto},cli::read_optional};
use serde::{Serialize,Deserialize};
use std::path::{Path,PathBuf};
pub struct FileChange {pub path:PathBuf,pub after:Vec<u8>}
#[derive(Serialize,Deserialize)]
struct Entry{path:PathBuf,before:Option<Vec<u8>>,after:Vec<u8>}
#[derive(Serialize,Deserialize)]
struct Journal{home:PathBuf,entries:Vec<Entry>}
pub struct Transaction{root:PathBuf,journal:Journal}
fn valid_path(home:&Path,path:&Path)->bool {
 path==home.join("config.toml") || (path.parent()==Some(home.join("credentials").as_path()) && path.file_stem().and_then(|v|v.to_str()).is_some_and(|v|crate::cli::safe_key(v).is_ok()) && path.extension().is_some_and(|e|e=="json"))
}
impl Transaction {
 pub fn prepare(root: &Path,home:&Path,changes:Vec<FileChange>)->AppResult<Self>{
  if root.join("pending.dpapi").exists(){return Err(AppError::new("recovery_required","存在未完成事务，请重启助手以恢复"));}
  let mut entries=vec![];
  for change in changes {if !valid_path(home,&change.path){return Err(AppError::new("transaction_path","事务路径不属于 CLI 凭据或配置"));} entries.push(Entry{before:read_optional(&change.path)?,path:change.path,after:change.after});}
  let journal=Journal{home:home.to_owned(),entries};
  let bytes=serde_json::to_vec(&journal).map_err(|_|AppError::new("transaction","事务编码失败"))?;
  atomic::write(&root.join("pending.dpapi"),&crypto::protect(&bytes)?)?;
  Ok(Self{root:root.to_owned(),journal})
 }
 pub fn apply_with<F:FnMut(&Path,&[u8])->AppResult<()>>(&self,mut write:F)->AppResult<()>{
  let result=(||{for entry in &self.journal.entries {write(&entry.path,&entry.after)?;if std::fs::read(&entry.path)?!=entry.after{return Err(AppError::new("verify","切换后文件验证失败"));}} Ok(())})();
  if result.is_err(){return match self.rollback(){Ok(())=>Err(AppError::new("switch_rolled_back","切换失败，原账号已恢复")),Err(_)=>Err(AppError::new("recovery_failed","切换失败且恢复未完成。请关闭 CLI 并重启助手，勿删除账号数据"))};}
  self.finish()
 }
 fn rollback(&self)->AppResult<()>{
  for entry in self.journal.entries.iter().rev(){if let Some(bytes)=&entry.before{atomic::write(&entry.path,bytes)?;}else if entry.path.exists(){std::fs::remove_file(&entry.path)?;}}
  self.finish()
 }
 fn finish(&self)->AppResult<()>{
  let backups=self.root.join("backups");std::fs::create_dir_all(&backups)?;
  let source=self.root.join("pending.dpapi");
  atomic::write(&backups.join(format!("{}-{}.dpapi",chrono::Utc::now().timestamp(),uuid::Uuid::new_v4())),&std::fs::read(&source)?)?;
  std::fs::remove_file(source)?;
  let mut files=std::fs::read_dir(&backups)?.filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|x|x=="dpapi")).collect::<Vec<_>>();
  files.sort_by_key(|e|std::cmp::Reverse(e.file_name()));
  for (i,file) in files.into_iter().enumerate(){let expired=file.metadata().and_then(|m|m.modified()).ok().and_then(|t|t.elapsed().ok()).is_some_and(|t|t.as_secs()>86400);if i>=3||expired{std::fs::remove_file(file.path())?;}}
  Ok(())
 }
 pub fn recover(root: &Path,home:&Path)->AppResult<()>{
  let Some(bytes)=read_optional(&root.join("pending.dpapi"))? else{return Ok(());};
  let journal:Journal=serde_json::from_slice(&crypto::unprotect(&bytes)?).map_err(|_|AppError::new("recovery_failed","恢复记录已损坏，请保留数据并检查备份"))?;
  if journal.home!=home||journal.entries.iter().any(|e|!valid_path(home,&e.path)){return Err(AppError::new("recovery_path","恢复记录与当前 CLI 目录不匹配，已停止写入"));}
  Self{root:root.to_owned(),journal}.rollback()
 }
}
