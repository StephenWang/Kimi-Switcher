use crate::model::AppResult;
use std::{path::Path,io::Write};
pub fn write(path:&Path,bytes:&[u8])->AppResult<()> {
 let parent=path.parent().ok_or_else(||crate::model::AppError::new("path","无效的数据目录"))?;
 std::fs::create_dir_all(parent)?;
 let mut tmp=tempfile::NamedTempFile::new_in(parent)?;
 tmp.write_all(bytes).map_err(|e|crate::model::AppError::new("atomic_write",&format!("临时写入失败 {:?}",e.raw_os_error())))?;
 tmp.as_file().sync_all().map_err(|e|crate::model::AppError::new("atomic_sync",&format!("文件同步失败 {:?}",e.raw_os_error())))?;
 let tmp_path=tmp.into_temp_path();
 std::fs::rename(&tmp_path,path)?;
 Ok(())
}
