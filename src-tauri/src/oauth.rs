use crate::{cli::Credential,model::{AppError,AppResult},usage::{UsageClient,endpoints}};
use serde_json::{Value,json};
pub fn parse_refreshed(old:&Credential,payload:&Value,now:i64)->AppResult<Credential>{
 let expiry=payload["expires_in"].as_f64().filter(|n|n.is_finite()&&*n>0.0&&*n<315360000.0).ok_or_else(||AppError::new("oauth_format","刷新接口返回了无效的令牌有效期"))?;
 if !payload["access_token"].as_str().is_some_and(|s|!s.is_empty())||!payload["refresh_token"].as_str().is_some_and(|s|!s.is_empty()){return Err(AppError::new("oauth_format","刷新接口未返回完整凭据"));}
 let mut wire=old.wire.clone();
 for key in ["access_token","refresh_token","scope","token_type"] {if !payload[key].is_null(){wire[key]=payload[key].clone();}}
 wire["expires_at"]=json!(now as f64+expiry);wire["expires_in"]=json!(expiry);
 let next=Credential::parse(&serde_json::to_vec(&wire).expect("JSON value"))?;
 if old.identity().is_some()&&next.identity().is_some()&&!old.same_account(&next){return Err(AppError::new("identity_changed","刷新凭据的账号身份不一致，已停止写入"));}Ok(next)
}
impl UsageClient {
 pub async fn fresh_credential(&self,credential:&Credential,region:&str)->AppResult<Credential>{
  let (_,auth)=endpoints(region)?;let now=chrono::Utc::now().timestamp();
  if credential.expires_at()>now as f64+300.0{return Ok(credential.clone());}
  let device=credential.device_id().unwrap_or_else(||"kimi-switcher".into());
  let request=self.client.post(format!("{auth}/api/oauth/token")).header("X-Msh-Device-Id",device).header("X-Msh-Platform","kimi_code_cli").header("X-Msh-Version","2.1.1").form(&[("client_id","17e5f671-d194-4dfb-9706-5516cb48c098"),("grant_type","refresh_token"),("refresh_token",credential.refresh())]);
  parse_refreshed(credential,&self.json(request).await?,now)
 }
}
