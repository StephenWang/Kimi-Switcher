# Kimi 换号器

Rust + Tauri 2 开发的 Windows 桌面应用，用于管理 Kimi Code CLI 登录账号。界面按提供的截图制作，支持深色和浅色主题。

## 功能

- 使用官方 `kimi login` 通过浏览器添加账号，登录过程隔离在临时目录，保留原 CLI 登录。
- 导入当前 CLI 账号，Windows DPAPI 加密保存；重命名及删除助手保存的账号。
- 切换默认 CLI 凭据，保留其他配置；备份、校验、失败回滚及启动恢复。
- 查询官方接口的 7 天 / 5 小时已用比例、重置时间和套餐名称；失败时保留上次数据。
- 中国大陆及全球地区账号；空状态、登录等待、取消、错误提示和窄窗口布局。

## 环境

Windows 10/11 x64、Microsoft Edge WebView2 Runtime、Kimi Code CLI **2.1.1**。目前仅接受此已验证版本，其他版本会显示不兼容提示。CLI 支持用户 npm 全局安装入口（`%APPDATA%/npm/node_modules/@moonshot-ai/kimi-code/dist/main.mjs` + PATH 中的 Node.js），或 PATH 中的 `kimi.exe`。

配置目录默认 `%USERPROFILE%/.kimi-code`；若进程设置了 `KIMI_CODE_HOME`，则管理该目录。自定义认证/API 主机与 keyring 凭据不在首版支持范围内。

## 使用

1. 启动 Windows 发布文件 `src-tauri/target/release/kimi-switcher.exe`，或安装 `src-tauri/target/release/bundle/nsis/` 内的安装包。
2. 已在 CLI 登录时，点击“导入当前账号”；添加其他账号时点击“添加账号”，在官方浏览器授权页自行完成登录。
3. 切换前关闭所有 Kimi CLI、server、web 和相关会话，点击目标账号的“切换”，再重新启动 `kimi`。
4. 点击单个账号或顶部“刷新”查询额度。进度条代表已用比例，接口未提供请求次数时只显示百分比。

添加账号与导入不会自动切换活动账号。删除账号只删除助手中的副本，CLI 当前登录仍保留。Windows CLI 2.1.1 未启用官方跨进程 OAuth 锁，助手会保守阻止检测到的所有 Kimi CLI 写进程；切换期间不要同时启动 CLI。关联目录不明的进程也会阻止切换。

## 开发和构建

需要 Node.js 22.12+、Rust stable，以及 Visual Studio 的“使用 C++ 的桌面开发”组件与 Windows SDK。

```powershell
cd D:/Personal/Projects/Gitee/backup_gemini_v3/kimi-switcher
npm ci
npm run tauri dev
```

```powershell
npm test -- --run
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features desktop -- -D warnings
npm run tauri build
```

`cargo test` 默认只编译可独立测试的 Rust 核心；Tauri 桌面编译使用 `desktop` feature，由 Tauri 构建配置启用。安装包按当前 Windows 用户安装，必要时下载微软 WebView2 安装程序。首版产物未做代码签名。

仅预览界面：`npm run dev` 后访问 `http://127.0.0.1:1420/?preview=reference`，可操作合成账号，不能访问真实凭据。`?preview=empty`、`?preview=error` 检查其他状态。未指定 preview 参数的浏览器页面会提示使用桌面应用。合成数据模块不会进入生产构建。

## 数据与恢复

助手数据位于 Tauri 当前用户应用数据目录，Windows 通常为 `%APPDATA%/com.kimiswitcher.desktop/`。

- `accounts.dpapi`：DPAPI 加密账号仓库，绑定当前 Windows 用户；不支持复制到另一台电脑解密使用。
- `operations.lock`：助手实例之间的操作锁。
- `pending.dpapi`：未完成切换的加密恢复记录，启动时恢复原文件。
- `refresh.dpapi`：已轮换令牌的加密恢复记录。刷新令牌不能撤销，启动时向前恢复最新凭据；检测到外部已登录其他账号时停止恢复。
- `backups/`：加密切换备份，正常完成事务时清理为最多 3 份、24 小时内；未完成事务记录不会按过期删除。
- `login-tmp/`：官方登录临时目录，完成、取消和超时后清理。授权期间官方 CLI 按其原生格式写入临时凭据。

恢复失败时，保留整个助手数据目录和 CLI 目录，先关闭 CLI 再重启助手；不要删除 `pending.dpapi` 或 `refresh.dpapi` 来绕过提示。原始 CLI 凭据保持官方格式，助手保存的副本和备份经过 DPAPI 加密。界面与日志不展示 token。

兼容性依据见 [docs/compatibility.md](docs/compatibility.md)，验证记录见 [docs/verification.md](docs/verification.md)。
