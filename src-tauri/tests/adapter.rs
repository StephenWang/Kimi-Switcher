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
fn allows_cli_versions_without_a_known_incompatibility() {
    for version in [
        "0.43.1", "1.0.0", "2.0.0", "2.0.1", "2.0.2", "2.1.0", "2.1.1", "2.2.0", "9.0.0",
    ] {
        assert!(
            supports_version(version),
            "{version} must not be rejected by a whitelist"
        );
    }
}
#[test]
fn rejects_only_known_incompatible_cli_and_missing_version() {
    use kimi_switcher::cli::CliAdapter;
    assert!(!supports_version("0.1.1"));
    assert!(!supports_version(" 0.1.1\n"));
    assert!(!supports_version(""));
    let error = CliAdapter::from_home(PathBuf::from("unused"), "0.1.1")
        .err()
        .unwrap();
    assert!(error.message.contains("0.1.1"));
    assert!(error.message.contains("login --region"));
}
#[test]
fn fresh_cli_directory_has_no_active_account() {
    use kimi_switcher::cli::CliAdapter;
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let cli = CliAdapter::from_home(dir.path().to_owned(), "2.1.1").unwrap();
    assert_eq!(
        cli.region_and_key().unwrap(),
        ("mainland-cn".into(), "kimi-code".into())
    );
    assert!(cli.read_active().unwrap().is_none());
    for config in ["", "default_model='other'\n", "[tools]\nmy_tool=true\n"] {
        std::fs::write(dir.path().join("config.toml"), config).unwrap();
        assert!(cli.read_active().unwrap().is_none());
    }
}
