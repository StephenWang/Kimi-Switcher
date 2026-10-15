use crate::model::{AppError, AppResult};
use super::Launch;
use serde_json::Value;
pub fn ensure_cli_stopped() -> AppResult<()> {
    let script = "$ErrorActionPreference='Stop'; ConvertTo-Json -InputObject @(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(kimi|kimi-code|kimi-cli)(\\.exe)?$' -or (($_.Name -match '^(node|bun|python|pythonw)(\\.exe)?$') -and (($_.CommandLine -match '(@moonshot-ai[\\\\/]kimi-code|kimi_cli|kimi-code[\\\\/].*main|kimi[\\\\/].*cli)') -or [string]::IsNullOrEmpty($_.CommandLine))) } | Select-Object -ExpandProperty ProcessId) -Compress";
    let output=Launch{executable:"powershell.exe".into(),prefix:vec!["-NoProfile".into(),"-NonInteractive".into(),"-Command".into(),script.into()]}.command().output()?;
    if !output.status.success() { return Err(AppError::new("process_scan", "无法核实 CLI 运行状态，已停止写入")); }
    validate_process_output(&output.stdout)
}
pub fn validate_process_output(bytes:&[u8])->AppResult<()> {
    let result:Value=serde_json::from_slice(bytes).map_err(|_|AppError::new("process_scan", "无法解析 CLI 运行状态，已停止写入"))?;
    if !result.is_array()&&!result.is_number(){return Err(AppError::new("process_scan","无法核实进程列表，已停止写入"));}
    if result.as_array().is_some_and(|a|!a.is_empty()) || result.is_number() { return Err(AppError::new("cli_running", "检测到运行中的 Kimi CLI。请先关闭所有 Kimi CLI / server 会话后重试")); }
    Ok(())
}
