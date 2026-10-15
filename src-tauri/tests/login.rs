use kimi_switcher::login::{authorization_url, login_arguments};
#[test]
fn accepts_only_official_authorization_url() {
    assert_eq!(
        authorization_url(
            "Opening browser for Kimi device login: https://auth.kimi.com/device?user_code=TEST"
        ),
        Some("https://auth.kimi.com/device?user_code=TEST".into())
    );
    assert!(authorization_url("https://auth.kimi.com.evil.invalid/device").is_none());
    assert!(authorization_url("https://user:pass@auth.kimi.com/device").is_none());
    assert!(authorization_url("http://auth.kimi.com/device").is_none());
    assert!(authorization_url("https://evil.invalid/?url=https://auth.kimi.com").is_none());
}
#[test]
fn login_region_is_a_separate_validated_argument() {
    assert_eq!(
        login_arguments("global").unwrap(),
        vec!["login", "--region", "global"]
    );
    assert!(login_arguments("global; echo secret").is_err());
}
#[tokio::test]
async fn cancel_and_timeout_cleanup_isolated_home() {
    use kimi_switcher::{cli::Launch, login::LoginManager};
    use std::time::Duration;
    for timeout in [Duration::from_secs(30), Duration::from_millis(150)] {
        let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
        let manager = LoginManager::default();
        let launch = Launch {
            executable: "node.exe".into(),
            prefix: vec![format!(
                "{}/tests/fixtures/login-wait.mjs",
                env!("CARGO_MANIFEST_DIR")
            )],
        };
        let started = manager
            .start(
                launch,
                "mainland-cn".into(),
                "test".into(),
                dir.path().to_owned(),
                timeout,
            )
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        if timeout.as_secs() > 1 {
            manager.cancel(&started.task_id).unwrap();
        }
        for _ in 0..80 {
            let s = manager.status(&started.task_id).unwrap();
            if matches!(s.stage.as_str(), "cancelled" | "failed") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        assert!(matches!(
            manager.status(&started.task_id).unwrap().stage.as_str(),
            "cancelled" | "failed"
        ));
        assert_eq!(
            std::fs::read_dir(dir.path().join("login-tmp"))
                .unwrap()
                .count(),
            0
        );
        assert!(!dir.path().join("accounts.dpapi").exists());
    }
}
#[tokio::test]
async fn isolated_login_imports_without_touching_active_cli() {
    use kimi_switcher::{cli::Launch, login::LoginManager, storage::Repository};
    use std::time::Duration;
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let sentinel = dir.path().join("active.json");
    std::fs::write(&sentinel, b"original-account").unwrap();
    let root = dir.path().join("vault space 账户");
    let manager = LoginManager::default();
    let launch = Launch {
        executable: "node.exe".into(),
        prefix: vec![format!(
            "{}/tests/fixtures/login-success.mjs",
            env!("CARGO_MANIFEST_DIR")
        )],
    };
    let job = manager
        .start(
            launch,
            "mainland-cn".into(),
            "Test".into(),
            root.clone(),
            Duration::from_secs(10),
        )
        .unwrap();
    for _ in 0..100 {
        if manager.status(&job.task_id).unwrap().stage == "complete" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(manager.status(&job.task_id).unwrap().stage, "complete");
    assert_eq!(std::fs::read(sentinel).unwrap(), b"original-account");
    assert_eq!(Repository::open(root.clone()).unwrap().views(None).len(), 1);
    assert_eq!(
        std::fs::read_dir(root.join("login-tmp")).unwrap().count(),
        0
    );
}
#[tokio::test]
async fn stderr_eof_does_not_disable_cancellation() {
    use kimi_switcher::{cli::Launch, login::LoginManager};
    use std::time::Duration;
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let manager = LoginManager::default();
    let launch = Launch {
        executable: "node.exe".into(),
        prefix: vec![format!(
            "{}/tests/fixtures/login-eof.mjs",
            env!("CARGO_MANIFEST_DIR")
        )],
    };
    let job = manager
        .start(
            launch,
            "mainland-cn".into(),
            "Test".into(),
            dir.path().to_owned(),
            Duration::from_secs(10),
        )
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    manager.cancel(&job.task_id).unwrap();
    for _ in 0..20 {
        if manager.status(&job.task_id).unwrap().stage == "cancelled" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(manager.status(&job.task_id).unwrap().stage, "cancelled");
}
