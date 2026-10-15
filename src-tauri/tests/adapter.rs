use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use kimi_switcher::cli::{resolve_home, supports_version, Credential};
use serde_json::json;
use std::path::PathBuf;
fn token(uid: &str, serial: &str) -> Vec<u8> {
    let payload = URL_SAFE_NO_PAD.encode(json!({"user_id":uid,"jti":serial}).to_string());
    serde_json::to_vec(&json!({"access_token":format!("e30.{payload}.sig"),"refresh_token":serial,"expires_at":2000000000,"expires_in":3600,"token_type":"Bearer","scope":""})).unwrap()
}
#[test]
fn resolves_unicode_home_override() {
    assert_eq!(
        resolve_home(
            PathBuf::from("C:/用户"),
            Some(PathBuf::from("D:/账户 文件"))
        ),
        PathBuf::from("D:/账户 文件")
    );
    assert_eq!(
        resolve_home(PathBuf::from("C:/用户"), None),
        PathBuf::from("C:/用户/.kimi-code")
    );
}
#[test]
fn recognizes_rotated_token_identity() {
    let a = Credential::parse(&token("user-a", "old")).unwrap();
    let b = Credential::parse(&token("user-a", "new")).unwrap();
    let c = Credential::parse(&token("user-b", "other")).unwrap();
    assert!(a.same_account(&b));
    assert!(!a.same_account(&c));
}
#[test]
fn rejects_unsupported_credential() {
    assert!(Credential::parse(br#"{"access_token":"secret"}"#).is_err());
}
#[test]
fn unknown_identity_is_not_assumed_equal() {
    let a = Credential::parse(
        &serde_json::to_vec(
            &json!({"access_token":"opaque-a","refresh_token":"one","expires_at":2000000000}),
        )
        .unwrap(),
    )
    .unwrap();
    let b = Credential::parse(
        &serde_json::to_vec(
            &json!({"access_token":"opaque-b","refresh_token":"two","expires_at":2000000000}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!a.same_account(&b));
}
#[test]
fn rejects_unsupported_cli_version() {
    assert!(supports_version("2.1.1"));
    assert!(!supports_version("1.0.0"));
    assert!(!supports_version("9.0.0"));
}
