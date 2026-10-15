use crate::{cli::Credential,model::{AppResult,AppError,QuotaSnapshot,QuotaWindow}};
use serde_json::Value;
use std::time::Duration;
pub fn parse_usage(payload: &Value)->AppResult<QuotaSnapshot>{
 if !payload["usages"].is_object(){return Err(AppError::new("usage_format","额度接口返回了不支持的数据结构"));}
 let parse=|key:&str|->Option<QuotaWindow>{let entry=&payload["usages"][key];let ratio=entry["used_ratio"].as_f64().or_else(||entry["used_ratio"].as_str()?.parse().ok())?;if !ratio.is_finite()||ratio<0.0{return None;}Some(QuotaWindow{used_ratio:ratio,reset_at:entry["reset_time"].as_str().map(str::to_owned)})};
 Ok(QuotaSnapshot{week:parse("limit_7d"),five_hour:parse("limit_5h"),plan:None,fetched_at:Some(chrono::Utc::now().to_rfc3339())})
}
pub fn retry_delay(status:u16,header:Option<&str>,attempt:usize)->AppResult<Option<u64>>{
 if attempt>=2||!matches!(status,0|429|500|502|503|504){return Ok(None);}
 let seconds=if status==429{header.and_then(|h|h.parse::<u64>().ok().or_else(||chrono::DateTime::parse_from_rfc2822(h).ok().map(|t|(t.timestamp()-chrono::Utc::now().timestamp()).max(0) as u64))).unwrap_or(1<<attempt)}else{1<<attempt};
 if seconds>30{return Err(AppError::new("rate_limit","请求受限，请稍后手动刷新"));}Ok(Some(seconds))
}
pub fn endpoints(region: &str)->AppResult<(&'static str,&'static str)>{match region {"mainland-cn"=>Ok(("https://api.kimi.com/coding/v1","https://auth.kimi.com")),"global"=>Ok(("https://api.kimi.ai/coding/v1","https://auth.kimi.ai")),_=>Err(AppError::new("endpoint","未知地区，已停止发送凭据"))}}
#[derive(Clone)]
pub struct UsageClient { pub(crate) client:reqwest::Client }
impl UsageClient {
 pub fn new()->AppResult<Self>{Ok(Self{client:reqwest::Client::builder().timeout(Duration::from_secs(20)).redirect(reqwest::redirect::Policy::none()).user_agent("kimi-switcher/0.1.0").build().map_err(|_|AppError::new("network","无法初始化 HTTPS 客户端"))?})}
 pub(crate) async fn json(&self,request:reqwest::RequestBuilder)->AppResult<Value>{
  for attempt in 0..3 {
   let response=request.try_clone().ok_or_else(||AppError::new("network","无法重试请求"))?.send().await;
   match response {
    Ok(response)=>{
     let status=response.status().as_u16();
     if response.status().is_success(){
      if response.content_length().is_some_and(|n|n>2*1024*1024){return Err(AppError::new("response_size","接口数据超出大小限制"));}
      let bytes=response.bytes().await.map_err(|_|AppError::new("network","响应读取失败，请重试"))?;
      if bytes.len()>2*1024*1024{return Err(AppError::new("response_size","接口数据超出大小限制"));}
      return serde_json::from_slice(&bytes).map_err(|_|AppError::new("response_format","接口返回内容无法解析"));
     }
     if let Some(delay)=retry_delay(status,response.headers().get("retry-after").and_then(|h|h.to_str().ok()),attempt)? {tokio::time::sleep(Duration::from_secs(delay)).await;continue;}
     return Err(AppError::new(if matches!(status,401|403){"unauthorized"}else{"http"},&format!("接口请求失败（HTTP {status}）{}",if matches!(status,401|403){"，请重新登录"}else{"，请稍后重试"})));
    },
    Err(_)=>{if let Some(delay)=retry_delay(0,None,attempt)?{tokio::time::sleep(Duration::from_secs(delay)).await;continue;}return Err(AppError::new("network","网络连接失败或请求超时，请检查网络后重试"));}
   }
  }
  Err(AppError::new("network","请求重试已用尽"))
 }
 pub async fn fetch_quota(&self,credential:&Credential,region:&str)->AppResult<QuotaSnapshot>{
  let (api,_)=endpoints(region)?;
  let payload=self.json(self.client.get(format!("{api}/usages")).bearer_auth(credential.access()).header("Accept","application/json")).await?;
  let mut quota=parse_usage(&payload)?;
  if let Ok(me)=self.json(self.client.get(format!("{api}/me")).bearer_auth(credential.access())).await {
   quota.plan=me["user_level_name"].as_str().filter(|s|!s.trim().is_empty()).map(str::to_owned);
  }
  Ok(quota)
 }
}
