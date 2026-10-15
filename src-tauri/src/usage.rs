use crate::model::{AppResult,AppError,QuotaSnapshot};
pub fn parse_usage(_: &serde_json::Value)->AppResult<QuotaSnapshot>{Err(AppError::new("unimplemented","未实现"))}
pub fn retry_delay(_:u16,_:Option<&str>,_:usize)->AppResult<Option<u64>>{Ok(None)}
pub fn endpoints(_: &str)->AppResult<(&'static str,&'static str)>{Err(AppError::new("unimplemented","未实现"))}
