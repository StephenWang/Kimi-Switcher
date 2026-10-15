# 独立审阅与修复

日期：2026-10-15。独立审阅范围：b8571ce → e2bc11e；涵盖 Rust、前端、设计、计划及实现取舍。审阅只使用合成数据，未访问真实凭据或线上账号接口。以下问题均在最终修复阶段以失败测试复现后修正，并运行完整测试套件；未以二次审阅代替回归测试。

## 审阅发现

1. Important：配置缺少 providers 表时首次启动崩溃。改为可选读取，缺少配置、空配置、只有其他配置时均返回默认地区与空登录状态。回归测试 `fresh_cli_directory_has_no_active_account` 已观察失败后通过。
2. Important：刷新恢复只比较账号身份，可能用旧令牌覆盖同账号的新一代令牌。改为精确比较事务前后凭据；遇到第三种状态时保持文件和恢复记录不变，返回冲突。回归测试 `refresh_recovery_preserves_newer_external_token_for_same_account` 已观察失败后通过。
3. Important：切换恢复未检查外部更新，可能覆盖用户后来的登录或配置。改为写入前检查整个文件集合，只允许事务原状态或目标状态；发现任何外部修改即停止，保留所有文件和记录。回归测试 `switch_recovery_preserves_external_changes_and_keeps_journal`、`switch_recovery_checks_all_files_before_restoring_any` 已观察失败后通过。
4. 原分级 Minor，实施者按实际影响升为 Important：批量额度查询忽略错误，可能在完全失败时报告成功。现在汇总失败并返回成功数/总数及错误，前端进入错误提示路径。回归测试 `bulk_refresh_cannot_report_success_when_some_requests_failed` 已观察失败后通过。

没有未处理的独立审阅发现；无延期的代码审阅 Minor。

## 审阅未判定的范围与实施判断

- 真实 OAuth 和联网额度：需要用户账号及浏览器交互，保留为明确的未验证项，不以合成测试冒充真实授权。
- 安装向导执行及视觉一致性：视觉已由浏览器对照验收；可执行文件已启动验证，安装向导未在用户系统执行。
- Windows 下用户同时新启动 CLI 的绝对隔离：官方 CLI 2.1.1 没有 Windows OAuth 锁，保留进程阻止、文件独占与明确限制；仍不承诺同时新启动时完全隔离。
