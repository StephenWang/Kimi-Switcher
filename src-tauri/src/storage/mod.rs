pub mod crypto;
pub mod atomic;
use crate::{cli::Credential,model::{AppResult,AppError,AccountView}};
use std::path::PathBuf;
pub struct Repository { pub root:PathBuf }
impl Repository {
 pub fn open(root:PathBuf)->AppResult<Self>{std::fs::create_dir_all(&root)?;Ok(Self{root})}
 pub fn import(&mut self,_:Credential,_:&str,_:Option<&str>)->AppResult<String>{Err(AppError::new("unimplemented","未实现"))}
 pub fn credential(&self,_:&str)->AppResult<Credential>{Err(AppError::new("unimplemented","未实现"))}
 pub fn rename(&mut self,_:&str,_:&str)->AppResult<()>{Err(AppError::new("unimplemented","未实现"))}
 pub fn remove(&mut self,_:&str)->AppResult<()>{Err(AppError::new("unimplemented","未实现"))}
 pub fn views(&self,_:Option<&Credential>)->Vec<AccountView>{vec![]}
}
