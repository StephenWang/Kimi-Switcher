use crate::model::{AppError, AppResult};
use fs2::FileExt;
use std::{
    fs::{File, OpenOptions},
    path::Path,
};
pub struct OperationLock(File);
impl OperationLock {
    pub fn acquire(root: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(root)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("operations.lock"))?;
        file.try_lock_exclusive()
            .map_err(|_| AppError::new("busy", "另一个助手实例正在操作账号，请稍后重试"))?;
        Ok(Self(file))
    }
}
impl Drop for OperationLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}
