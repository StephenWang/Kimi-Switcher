use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use kimi_switcher::{
    cli::{lock::OperationLock, CliAdapter, Credential},
    storage::Repository,
    switch::{merge_config, SwitchService},
};
use serde_json::json;
fn cred(user: &str, serial: &str) -> Credential {
    let p = URL_SAFE_NO_PAD.encode(json!({"user_id":user}).to_string());
    Credential::parse(&serde_json::to_vec(&json!({"access_token":format!("e30.{p}.sig"),"refresh_token":serial,"expires_at":2000000000})).unwrap()).unwrap()
}
fn setup() -> (tempfile::TempDir, SwitchService, String, String) {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let home = dir.path().join("cli");
    std::fs::create_dir_all(home.join("credentials")).unwrap();
    std::fs::write(
        home.join("credentials/kimi-code.json"),
        cred("a", "latest-a").bytes(),
    )
    .unwrap();
    std::fs::write(home.join("config.toml"),"default_model = 'my-model'\n[providers.'managed:kimi-code']\ntype='kimi'\nbase_url='https://api.kimi.com/coding/v1'\n[providers.'managed:kimi-code'.oauth]\nstorage='file'\nkey='oauth/kimi-code'\n[permission]\ncustom='keep'\n").unwrap();
    let mut repo = Repository::open(dir.path().join("vault")).unwrap();
    let a = repo.import(cred("a", "old-a"), "A", None).unwrap();
    let b = repo.import(cred("b", "token-b"), "B", None).unwrap();
    let cli = CliAdapter::from_home(home, "2.1.1").unwrap();
    (dir, SwitchService::new(repo, cli), a, b)
}
#[test]
fn syncs_latest_active_tokens_before_switch() {
    let (_dir, mut svc, a, b) = setup();
    svc.switch_to(&b).unwrap();
    assert_eq!(svc.repo.credential(&a).unwrap().refresh(), "latest-a");
    assert_eq!(svc.cli.read_active().unwrap().unwrap().refresh(), "token-b");
    assert!(svc.cli.config().unwrap().contains("custom='keep'"));
}
#[test]
fn lock_contention_writes_nothing() {
    let (_dir, mut svc, _a, b) = setup();
    let _lock = OperationLock::acquire(&svc.repo.root).unwrap();
    assert!(svc.switch_to(&b).is_err());
    assert_eq!(
        svc.cli.read_active().unwrap().unwrap().refresh(),
        "latest-a"
    );
}
#[test]
fn preserves_unrelated_toml() {
    let merged = merge_config(
        "default_model='custom'\n[tools]\nmy_tool=true\n",
        "[providers.'managed:kimi-code']\ntype='kimi'\nbase_url='https://api.kimi.com/coding/v1'\n",
        "mainland-cn",
        "kimi-code",
    )
    .unwrap();
    assert!(merged.contains("my_tool=true"));
    assert!(merged.contains("default_model='custom'"));
    assert!(merged.contains("oauth/kimi-code"), "{merged}");
}
#[test]
fn failure_at_each_write_restores_both_files() {
    use kimi_switcher::{
        model::AppError,
        storage::atomic,
        transaction::{FileChange, Transaction},
    };
    for fail_at in 0..2 {
        let (_dir, svc, _, _) = setup();
        let config = svc.cli.home.join("config.toml");
        let cred = svc.cli.credential_path().unwrap();
        let before_config = std::fs::read(&config).unwrap();
        let before_cred = std::fs::read(&cred).unwrap();
        let tx = Transaction::prepare(
            &svc.repo.root,
            &svc.cli.home,
            vec![
                FileChange {
                    path: config.clone(),
                    after: b"new-config".to_vec(),
                },
                FileChange {
                    path: cred.clone(),
                    after: b"new-credential".to_vec(),
                },
            ],
        )
        .unwrap();
        let mut index = 0;
        assert!(tx
            .apply_with(|p, b| {
                let n = index;
                index += 1;
                if n == fail_at {
                    Err(AppError::new("injected", "write failed"))
                } else {
                    atomic::write(p, b)
                }
            })
            .is_err());
        assert_eq!(std::fs::read(&config).unwrap(), before_config);
        assert_eq!(std::fs::read(&cred).unwrap(), before_cred);
    }
}
#[test]
fn startup_recovers_pending_transaction() {
    use kimi_switcher::{
        storage::atomic,
        transaction::{FileChange, Transaction},
    };
    let (_dir, svc, _, _) = setup();
    let config = svc.cli.home.join("config.toml");
    let before = std::fs::read(&config).unwrap();
    let _tx = Transaction::prepare(
        &svc.repo.root,
        &svc.cli.home,
        vec![FileChange {
            path: config.clone(),
            after: b"interrupted".to_vec(),
        }],
    )
    .unwrap();
    atomic::write(&config, b"interrupted").unwrap();
    Transaction::recover(&svc.repo.root, &svc.cli.home).unwrap();
    assert_eq!(std::fs::read(&config).unwrap(), before);
    assert!(!svc.repo.root.join("pending.dpapi").exists());
}
#[test]
fn running_or_unreadable_cli_blocks_write() {
    use kimi_switcher::cli::process::validate_process_output;
    assert!(validate_process_output(b"[123]").is_err());
    assert!(validate_process_output(b"null").is_err());
    assert!(validate_process_output(b"unknown").is_err());
    assert!(validate_process_output(b"[]").is_ok());
}
#[test]
fn interrupted_refresh_keeps_rotated_token_and_rolls_forward() {
    use kimi_switcher::refresh_recovery;
    let (_dir, mut svc, a, _) = setup();
    let old = svc.cli.read_active().unwrap().unwrap();
    let fresh = cred("a", "rotated-r");
    refresh_recovery::stage(&svc.repo, &svc.cli, &a, &old, &fresh).unwrap();
    refresh_recovery::complete(&mut svc.repo, &svc.cli).unwrap();
    assert_eq!(
        svc.cli.read_active().unwrap().unwrap().refresh(),
        "rotated-r"
    );
    assert_eq!(svc.repo.credential(&a).unwrap().refresh(), "rotated-r");
    assert!(!svc.repo.root.join("refresh.dpapi").exists());
}
#[test]
fn refresh_recovery_does_not_overwrite_external_login() {
    use kimi_switcher::refresh_recovery;
    let (_dir, mut svc, a, _) = setup();
    let old = svc.cli.read_active().unwrap().unwrap();
    let fresh = cred("a", "rotated-r");
    refresh_recovery::stage(&svc.repo, &svc.cli, &a, &old, &fresh).unwrap();
    std::fs::write(
        svc.cli.credential_path().unwrap(),
        cred("external", "external-token").bytes(),
    )
    .unwrap();
    assert!(refresh_recovery::complete(&mut svc.repo, &svc.cli).is_err());
    assert_eq!(
        svc.cli.read_active().unwrap().unwrap().refresh(),
        "external-token"
    );
}
