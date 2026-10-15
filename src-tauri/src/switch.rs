use crate::{cli::CliAdapter,storage::Repository,model::{AppResult,AppError}};
pub struct SwitchService {pub repo:Repository,pub cli:CliAdapter}
impl SwitchService {pub fn new(repo:Repository,cli:CliAdapter)->Self{Self{repo,cli}}pub fn switch_to(&mut self,_:&str)->AppResult<()>{Err(AppError::new("unimplemented","未实现"))} pub fn recover(&self)->AppResult<()>{Ok(())}}
pub fn merge_config(_: &str,_:&str,_:&str,_:&str)->AppResult<String>{Err(AppError::new("unimplemented","未实现"))}
