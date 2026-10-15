# CLI 兼容性依据

验证版本：本机 npm `@moonshot-ai/kimi-code` 2.1.1。实现依据来自安装包 dist/main.mjs 的 oauth/storage、oauth-manager、managed-userinfo、managed-usage、region 和 cli/login 区域（2026-10-15）。

- 凭据为 snake_case JSON：access_token、refresh_token、expires_at（Unix 秒）、expires_in、scope、token_type。
- 当前令牌 JWT payload 包含 user_id/sub、device_id；只用身份字段进行本地关联，不把未验证 JWT 当作授权认证。网络请求仍由官方服务校验 token。
- `/coding/v1/me` 的 user_id、user_level_name 为已核实账号与套餐字段；额度为 `/coding/v1/usages` 的 usages.limit_7d/limit_5h.used_ratio/reset_time。
- 大陆地区为 api.kimi.com / auth.kimi.com，全球为 api.kimi.ai / auth.kimi.ai；region 标记及 managed provider 配置决定地区。全球 OAuth 可能使用主机后缀凭据 key，必须解析配置引用，不假设文件名固定。
- OAuth client_id 为 17e5f671-d194-4dfb-9706-5516cb48c098，POST form 到 /api/oauth/token，grant_type=refresh_token。
- **Windows OAuthManager.resolveLockTarget 明确返回 undefined，无官方跨进程锁可复用。** 应用阻止运行中的 CLI，使用自身进程锁、活动文件独占保护及提交前复核。应用不能保证用户同时新启动 CLI 的行为；切换时应关闭 CLI，完成后重新启动。
- FileTokenStorage.save 用临时文件、fsync、rename 保存。保留额外字段，不输出秘密。
- `kimi login --region mainland-cn|global` 在 stderr 输出授权 URL，并自行打开浏览器；可通过子进程 KIMI_CODE_HOME 隔离。

首版只接受已核实的 2.1.1，其他版本只读检测并提示不兼容。自动测试使用合成凭据和临时目录。
