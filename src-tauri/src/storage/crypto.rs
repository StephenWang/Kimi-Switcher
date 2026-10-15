use crate::model::{AppError,AppResult};
pub fn protect(_: &[u8])->AppResult<Vec<u8>>{Err(AppError::new("unimplemented","未实现"))}
pub fn unprotect(_: &[u8])->AppResult<Vec<u8>>{Err(AppError::new("unimplemented","未实现"))}
