use crate::model::{AppError, AppResult};
pub fn protect(bytes: &[u8]) -> AppResult<Vec<u8>> {
    transform(bytes, true)
}
pub fn unprotect(bytes: &[u8]) -> AppResult<Vec<u8>> {
    transform(bytes, false)
}
#[cfg(windows)]
fn transform(bytes: &[u8], encrypt: bool) -> AppResult<Vec<u8>> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(AppError::new("crypto", "账号数据超出大小限制"));
    }
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // DPAPI allocation is released with LocalFree; no UI and current-user scope.
    let ok = unsafe {
        if encrypt {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(AppError::new(
            "crypto",
            "无法加密或解密账号数据，请使用原 Windows 用户并检查数据完整性",
        ));
    }
    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData as *mut _);
    }
    Ok(result)
}
#[cfg(not(windows))]
fn transform(_: &[u8], _: bool) -> AppResult<Vec<u8>> {
    Err(AppError::new("platform", "此版本仅支持 Windows DPAPI"))
}
