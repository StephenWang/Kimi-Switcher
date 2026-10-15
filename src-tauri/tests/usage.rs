use kimi_switcher::{
    cli::Credential,
    oauth::parse_refreshed,
    usage::{endpoints, parse_usage, retry_delay},
};
use serde_json::json;
#[test]
fn parses_week_and_five_hour_used_ratios() {
    let q=parse_usage(&json!({"usages":{"limit_7d":{"used_ratio":0.3,"reset_time":"2026-10-18T01:00:00Z"},"limit_5h":{"used_ratio":"0.5"}}})).unwrap();
    assert_eq!(q.week.unwrap().used_ratio, 0.3);
    assert_eq!(q.five_hour.unwrap().used_ratio, 0.5);
    assert!(q.plan.is_none());
}
#[test]
fn missing_is_not_zero() {
    let q = parse_usage(&json!({"usages":{"limit_5h":{"used_ratio":0}}})).unwrap();
    assert!(q.week.is_none());
    assert_eq!(q.five_hour.unwrap().used_ratio, 0.0);
}
#[test]
fn rejects_nonfinite_ratio() {
    assert!(
        parse_usage(&json!({"usages":{"limit_5h":{"used_ratio":"NaN"}}}))
            .unwrap()
            .five_hour
            .is_none()
    );
    assert!(
        parse_usage(&json!({"usages":{"limit_5h":{"used_ratio":-1}}}))
            .unwrap()
            .five_hour
            .is_none()
    );
}
#[test]
fn unauthorized_is_not_retried() {
    assert_eq!(retry_delay(401, None, 0).unwrap(), None);
    assert_eq!(retry_delay(403, None, 0).unwrap(), None);
}
#[test]
fn rate_limit_honors_retry_after() {
    assert_eq!(retry_delay(429, Some("5"), 0).unwrap(), Some(5));
    assert!(retry_delay(429, Some("90"), 0).is_err());
    assert_eq!(retry_delay(500, None, 0).unwrap(), Some(1));
    assert_eq!(retry_delay(500, None, 1).unwrap(), Some(2));
    assert_eq!(retry_delay(500, None, 2).unwrap(), None);
}
#[test]
fn untrusted_host_receives_no_token() {
    assert!(endpoints("https://evil.invalid").is_err());
    assert_eq!(
        endpoints("global").unwrap().0,
        "https://api.kimi.ai/coding/v1"
    );
}
#[test]
fn refreshed_token_preserves_wire_fields() {
    let c = Credential::parse(
        &serde_json::to_vec(
            &json!({"access_token":"old","refresh_token":"old-r","expires_at":1,"extra":"keep"}),
        )
        .unwrap(),
    )
    .unwrap();
    let next = parse_refreshed(
        &c,
        &json!({"access_token":"new","refresh_token":"new-r","expires_in":3600}),
        1000,
    )
    .unwrap();
    assert_eq!(next.expires_at(), 4600.0);
    assert_eq!(next.refresh(), "new-r");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&next.bytes()).unwrap()["extra"],
        "keep"
    );
    assert!(parse_refreshed(&c, &json!({"access_token":"bad"}), 1000).is_err());
}
#[test]
fn bulk_refresh_cannot_report_success_when_some_requests_failed() {
    use kimi_switcher::{model::AppError, usage::summarize_refresh};
    let error = AppError::new("cli_running", "请关闭 CLI");
    assert!(summarize_refresh(3, &[]).is_ok());
    let result = summarize_refresh(3, &[error]);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("2/3"));
}
