use crate::{cli::Credential,model::{AppError,AppResult}};
pub fn parse_refreshed(_: &Credential,_:&serde_json::Value,_:i64)->AppResult<Credential>{Err(AppError::new("unimplemented","未实现"))}
