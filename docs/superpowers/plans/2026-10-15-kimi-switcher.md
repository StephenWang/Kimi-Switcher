# Kimi Code CLI 多账号切换助手 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 交付按截图制作的 Windows 桌面应用，安全管理并切换 Kimi Code CLI 登录账号，显示真实额度。

**Architecture:** TypeScript + Vite 负责视图，Rust + Tauri 2 负责 CLI 适配、DPAPI 账号仓库、登录、额度和切换事务。前端只能获取脱敏账号视图，账号变更及活动凭据写入通过事务协调。

**Tech Stack:** Rust stable、Tauri 2、TypeScript、Vite；serde/serde_json、tokio、reqwest(rustls)、toml_edit、windows、uuid、sha2、chrono、tempfile；前端使用原生 DOM 组件与 Vitest。

**Spec:** `docs/superpowers/specs/2026-10-14-kimi-switcher-design.md`（用户已批准）。

## Global Constraints

- 首版面向本机已安装的 Kimi Code CLI 2.1.1。
- 项目独立放在 `kimi-switcher/`，交付源代码、开发说明和通过构建验证的 Windows 应用。已有 Python 项目不属于修改范围。
- 前端、普通日志和错误消息不包含 access token、refresh token 或完整凭据文件。
- 使用临时 CLI 目录验证添加、导入、切换和删除，不修改用户真实凭据进行自动化测试。
- 检测到正在使用同一全局凭据目录的 Kimi CLI 时阻止切换，并提示先关闭相关会话，不强制结束用户进程。
- 套餐信息仅展示经核实的接口字段，缺失时显示“套餐信息暂不可用”。
- 进度条表示已用百分比，默认以百分比显示。
- 真实浏览器授权需要用户完成登录；没有该验证时明确报告“未验证真实新账号授权”，不能把模拟流程说成真实登录成功。

## Review Focus

1. 外部 CLI 刷新或登录后，当前账号不能错误归属；任务 2、3、4 验证身份变化和最新令牌同步。
2. 非 ASCII 用户目录、空格路径、环境目录覆盖必须正确解析；任务 1、5 验证路径和无 shell 参数启动。
3. 崩溃发生在凭据与配置写入之间时，必须恢复成同一个账号；任务 4 注入每个写入阶段失败。
4. 额度缺失、0%、超范围、无效重置时间和时区跨日不能误导；任务 3、6 验证解析与显示。
5. 登录取消、应用关闭、授权失败不得遗留子进程或临时凭据；任务 5 验证任务生命周期。

## 文件布局与共同类型

所有下列路径相对于 `kimi-switcher/`。

- `src-tauri/src/model.rs`：`AccountId`（UUID 字符串）、`AccountView`、`AppSnapshot`、`QuotaWindow`、`LoginStatus`、`AppError`。`AppResult<T> = Result<T, AppError>`。
- `src-tauri/src/cli/{mod.rs,paths.rs,credentials.rs,process.rs,lock.rs}`：版本适配、路径、秘密凭据、进程检测和 CLI 锁。
- `src-tauri/src/storage/{mod.rs,crypto.rs,atomic.rs}`：加密仓库与原子文件操作。
- `src-tauri/src/{usage.rs,oauth.rs,switch.rs,login.rs,commands.rs,lib.rs,main.rs}`：业务服务与 Tauri 入口。
- `src/{main.ts,api.ts,types.ts,styles.css}`、`src/components/{account-card.ts,quota-bar.ts,dialogs.ts}`：界面及 IPC。
- `src-tauri/tests/{adapter.rs,repository.rs,usage.rs,switch.rs,login.rs}`、`src/tests/{format.test.ts,ui.test.ts}`：隔离测试。
- `src-tauri/tests/fixtures/`：合成凭据、配置及 HTTP 响应；不能复制用户真实凭据。
- `README.md`、`docs/compatibility.md`、`docs/verification.md`：使用、适配依据与验证记录。

`Credential` 是 Rust 私有秘密类型，不实现前端序列化或含秘密的 Debug。`AccountView` 只含 id、name、isCurrent、plan、quota、updatedAt、status、error。`QuotaWindow` 含 `usedRatio: f64` 与 `resetAt: Option<String>`；缺失周期使用 `Option`，不能补 0。`AppSnapshot` 含 accounts、cliVersion、cliHome、activeState、operation。`LoginStatus` 含任务 ID、阶段、授权 URL 和可展示错误。

### Task 1: 可构建的工程与 CLI 只读适配器

**Files:** 创建工程配置 `package.json`、`tsconfig.json`、`vite.config.ts`、`index.html`、`.gitignore`、`src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities/default.json}`、入口及上述 model、cli 模块；测试 `src-tauri/tests/adapter.rs`。

**Interfaces:** `CliAdapter::detect() -> AppResult<CliAdapter>`；`CliAdapter::from_home(home: PathBuf, version: &str) -> AppResult<CliAdapter>` 供测试；`read_active(&self) -> AppResult<Option<Credential>>`；`Credential::parse(bytes: &[u8]) -> AppResult<Credential>`；`identity(&self) -> Option<String>`；`same_account(&self, other: &Credential) -> bool`；`CliAdapter::guard(&self) -> AppResult<CliGuard>`。

- [ ] 只读检查已安装 2.1.1 的凭据字段、身份来源、地区端点、刷新请求、managed provider 配置和锁实现，将精确来源及锁算法记入 `docs/compatibility.md`。仅读取用户凭据键名和类型，禁止输出值。协议不明时保留拒绝写入分支。
- [ ] 建立最小 Tauri/Vite 配置和空入口，编写 `resolves_unicode_home_override`、`rejects_unsupported_credential`、`recognizes_rotated_token_identity`、`unknown_identity_is_not_assumed_equal`、`rejects_unsupported_cli_version`。合成新旧 token，断言稳定身份相同时为同账号，未知身份不能错误合并；旧 `.kimi` 不替代当前目录。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test adapter`，确认尚未实现的行为失败。
- [ ] 实现路径优先级、秘密类型、身份匹配和验证边界。进程检测使用 Windows 进程信息，检测不到关联归属时返回明确阻止原因。CLI 凭据锁必须使用任务第一步确认的原生锁语义，不能用不同锁文件假装互斥。
- [ ] 重跑 adapter 测试；运行 `npm run build` 与 `cargo check --manifest-path src-tauri/Cargo.toml`。通过后仅提交本任务文件，提交信息 `feat: scaffold Tauri app and inspect Kimi CLI safely`。

### Task 2: DPAPI 仓库与账号管理

**Files:** storage 模块、`model.rs`；测试 `src-tauri/tests/repository.rs`。

**Interfaces:** `Repository::open(root: PathBuf) -> AppResult<Repository>`；`import(&mut self, credential: Credential, name: &str, replace_id: Option<&str>) -> AppResult<AccountId>`；`credential(&self, id: &str) -> AppResult<Credential>`；`rename(&mut self, id: &str, name: &str) -> AppResult<()>`；`remove(&mut self, id: &str) -> AppResult<()>`；`views(&self, active: Option<&Credential>) -> Vec<AccountView>`；`save_credential(&mut self, id: &str, credential: &Credential) -> AppResult<()>`。

- [ ] 编写 `dpapi_round_trip_and_no_plaintext`、`duplicate_import_updates_tokens`、`ambiguous_identity_requires_explicit_choice`、`remove_current_keeps_cli_file`、`rename_rejects_empty_long_duplicate`。断言加密文件不包含合成 token；名称 trim 后长度 1–40 字符，重复名称返回可读错误；导入不修改活动凭据。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test repository` 确认失败。
- [ ] 实现 DPAPI 当前用户加密、UUID 账号 ID、版本化持久化格式、原子仓库写入、去重和显式覆盖选择。错误中不包含凭据字节；损坏仓库不能静默重建并丢失账号。
- [ ] 重跑 repository 与 adapter 测试；提交 `feat: persist and manage encrypted account profiles`。

### Task 3: OAuth 刷新与真实额度

**Files:** `oauth.rs`、`usage.rs`、合成 fixtures；测试 `src-tauri/tests/usage.rs`。

**Interfaces:** `UsageClient::refresh(&self, credential: &Credential) -> AppResult<RefreshResult>`，其中 `RefreshResult { credential: Credential, quota: QuotaSnapshot }`；`parse_usage(payload: &serde_json::Value) -> AppResult<QuotaSnapshot>`；`QuotaSnapshot` 含可选 week、fiveHour、plan 和 fetchedAt。网络通过内部 `HttpTransport` trait 注入测试响应，生产仅允许已核实官方 HTTPS 主机，禁止认证重定向到其他主机。

- [ ] 编写测试：`parses_week_and_five_hour_used_ratios` 断言输入 0.30/0.50 原样保留；`missing_is_not_zero` 断言缺失为 None、显式 0 为 Some；`rejects_nonfinite_ratio`；`expired_token_refreshes_once`；`unauthorized_is_not_retried`；`rate_limit_honors_retry_after`；`untrusted_host_receives_no_token`。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test usage` 确认失败。
- [ ] 按核实协议实现认证和额度请求：单次超时 20 秒，瞬时失败最多重试 2 次，延迟 1/2 秒；429 使用 Retry-After，超过 30 秒则返回等待提示。套餐字段未确认则返回 None，不能硬编码截图套餐。
- [ ] 增加网络失败时保留上次成功缓存、刷新后令牌轮换和未知地区拒绝发送的测试；重跑 usage 测试并提交 `feat: fetch real quota and refresh OAuth credentials`。

### Task 4: 原子切换与崩溃恢复

**Files:** `switch.rs`、storage atomic、cli lock/process 模块；测试 `src-tauri/tests/switch.rs`。

**Interfaces:** `SwitchService::switch_to(&mut self, id: &str) -> AppResult<AppSnapshot>`；`recover(&mut self) -> AppResult<()>`；`refresh_account(&mut self, id: &str) -> AppResult<AccountView>`。持有 Repository、CliAdapter 及 UsageClient，所有修改共享应用操作互斥锁。原子写入生产使用 Windows 可替换目标的原生文件 API。

- [ ] 编写 `syncs_latest_active_tokens_before_switch`、`lock_contention_writes_nothing`、`running_cli_blocks_switch`、`unknown_process_home_blocks_switch`、`preserves_unrelated_toml`、`failure_at_each_write_restores_both_files`、`startup_recovers_pending_transaction`。对准备备份、配置替换、凭据替换、回读和提交阶段分别注入失败。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test switch` 确认失败。
- [ ] 实现锁内重读、账号同步、加密事务日志、备份、配置最小合并、原子替换和回滚。成功事务保留最多 3 份且不超过 24 小时的加密备份，未完成事务不能按过期清除。恢复失败则阻止后续写操作。
- [ ] 实现当前账号刷新时锁内更新活动文件和仓库，非当前账号仅更新仓库。批量刷新最多 3 个并发请求，凭据提交和切换仍串行，使用操作版本避免过期请求回写旧账号状态。
- [ ] 重跑 switch、repository、usage 测试，提交 `feat: switch CLI accounts with recovery and conflict protection`。

### Task 5: 隔离登录任务与 Tauri IPC

**Files:** `login.rs`、`commands.rs`、Tauri 入口与 capabilities；测试 `src-tauri/tests/login.rs`。

**Interfaces:** Tauri 命令 `get_snapshot()`、`import_current(name, replaceId)`、`rename_account(id, name)`、`delete_account(id)`、`switch_account(id)`、`refresh_account(id)`、`refresh_all()`、`start_login(region, name)`、`get_login_status(taskId)`、`cancel_login(taskId)`；业务失败均返回脱敏 `AppError { code, message }`，成功返回新 snapshot 或 LoginStatus。

- [ ] 用测试子进程编写 `isolated_login_does_not_change_active_home`、`unicode_and_space_paths_are_single_arguments`、`cancel_kills_owned_child_and_cleans_directory`、`timeout_does_not_create_account`、`shutdown_cleans_owned_tasks`、`invalid_auth_url_is_not_opened`。不得调用真实用户登录或推理完成自动测试。
- [ ] 运行 `cargo test --manifest-path src-tauri/Cargo.toml --test login` 确认失败。
- [ ] 实现官方 CLI 的参数数组启动、独立 KIMI_CODE_HOME、stdout/stderr 流式状态解析及官方授权 URL 验证。任务超时 10 分钟，禁止记录原始输出中的秘密；清理临时文件失败要显示问题并阻止错误成功状态。
- [ ] 登录完成校验凭据并导入仓库，重复身份进入更新流程；配置合并交由任务 4。添加不改变当前账号。应用退出回收自己创建的任务，保持原 CLI 会话不受进程终止影响。
- [ ] 接入 IPC 和启动恢复；检查 capabilities/CSP 仅开放所需权限；重跑 login 测试及全部 Rust 测试，提交 `feat: integrate isolated CLI login and desktop commands`。

### Task 6: 截图界面与交互

**Files:** 前端文件、组件及 `src/tests/{format.test.ts,ui.test.ts}`。

**Interfaces:** `src/api.ts` 封装任务 5 同名命令并返回 `Promise<AppSnapshot>` 或 `Promise<LoginStatus>`。`renderAccountCard(account: AccountView): HTMLElement`；`formatReset(value: string | null, locale?: string): string`；`formatQuota(ratio: number | null): string`。秘密字段不出现在 TypeScript 数据类型。

- [ ] 编写前端测试：`formatQuota(null)` 为“暂无数据”，`formatQuota(0)` 为“0%”，0.3 为“30%”；无效日期返回“重置时间未知”；Asia/Shanghai 的跨日时间显示正确；显示百分比允许超过 100%，进度条宽度限制 0–100%。
- [ ] 使用 DOM 测试验证零账号入口、当前标记来自 snapshot、提交中禁用重复操作、失效登录错误、缺失套餐、删除确认、对话框 Escape/焦点恢复及刷新失败仍显示旧数据。运行 `npm test -- --run` 确认未实现行为失败。
- [ ] 实现深色截图布局：约 960×740 窗口、深色背景、圆角卡片、蓝色主按钮和套餐描边，7 天绿色、5 小时黄色，使用率达到 90% 为红色；小窗口换行布局。主题使用 CSS variables 并保存偏好。
- [ ] 实现命名/删除/账号身份冲突对话框、授权等待及取消、每账号错误与全局提示；使用 textContent 或安全 DOM API 渲染名称和网络文本。初始化时加载真实 snapshot，演示数据仅允许显式开发预览模式。
- [ ] 运行 `npm test -- --run`、`npm run build`；查看 960×740 深色、浅色、窄窗口、空状态和登录状态，修正与截图不符及可访问性问题；提交 `feat: build screenshot-based account switcher interface`。

### Task 7: 集成验证与 Windows 交付

**Files:** `README.md`、`docs/verification.md`、构建配置和必要图标资源。

**Interfaces:** `npm run dev` 启动前端；`npm run tauri dev` 启动桌面应用；`npm run tauri build` 生成 Windows 发布产物。README 给出绝对项目入口及相对构建路径，不暴露用户账号和凭据。

- [ ] 在临时数据目录使用合成账号完整运行导入→刷新模拟响应→切换→重命名→删除→重启恢复流程。记录每个操作后活动凭据和配置的一致性；同时验证取消登录和锁冲突。
- [ ] 运行 `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`、`cargo test --manifest-path src-tauri/Cargo.toml`、`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`、`npm test -- --run`、`npm run build`、`npm run tauri build`，记录实际结果并修复失败项。
- [ ] 检查 Windows 应用实际启动、主题切换、卡片布局、键盘操作和无 CLI 时错误提示；检查发布包不包含 fixtures、临时凭据、账号库或调试日志。
- [ ] 完成 README：安装依赖、运行/构建命令、支持 CLI 版本、KIMI_CODE_HOME、生效时机、DPAPI 范围、数据位置、备份及恢复方式。verification 分别记录自动验证、视觉验证和未执行的真实浏览器授权。
- [ ] 按 requesting-code-review 技能完成独立审阅并处理发现；最终重新运行受修改影响的验证。只提交本项目文件，提交 `docs: document Windows build and verified account workflows`，向用户提供可运行产物路径和明确限制。

## 执行方式建议与自检

建议 Native：由当前助手在本会话依次实现，各任务共享较多凭据与事务接口，顺序执行更容易保持一致；完成后进行一次独立审阅。也可选择 Subagent-driven，由独立实施者和审阅者逐任务执行。

计划自检已覆盖设计中的全部功能、故障恢复、保密边界、CLI 兼容性、并发、可访问性和交付验证。所有自动化集成操作限定在临时 CLI 目录。实现时如发现 2.1.1 协议与适配假设不一致，先更新兼容性证据，再实现或收紧功能；不能以模拟成功替代真实集成。
