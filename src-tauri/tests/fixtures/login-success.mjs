import fs from 'node:fs';
import path from 'node:path';
const home = process.env.KIMI_CODE_HOME;
fs.mkdirSync(path.join(home, 'credentials'), { recursive: true });
fs.writeFileSync(path.join(home, 'credentials/kimi-code.json'), JSON.stringify({access_token:'synthetic-token',refresh_token:'synthetic-refresh',expires_at:2000000000}));
fs.writeFileSync(path.join(home, 'config.toml'), "default_model='test'\n[providers.'managed:kimi-code']\nbase_url='https://api.kimi.com/coding/v1'\n");
