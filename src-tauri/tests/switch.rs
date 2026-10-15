use kimi_switcher::{cli::{CliAdapter,Credential,lock::OperationLock},storage::Repository,switch::{SwitchService,merge_config}};
use serde_json::json;
use base64::{Engine,engine::general_purpose::URL_SAFE_NO_PAD};
fn cred(user:&str,serial:&str)->Credential{let p=URL_SAFE_NO_PAD.encode(json!({"user_id":user}).to_string());Credential::parse(&serde_json::to_vec(&json!({"access_token":format!("e30.{p}.sig"),"refresh_token":serial,"expires_at":2000000000})).unwrap()).unwrap()}
fn setup()->(tempfile::TempDir,SwitchService,String,String){
 let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();let home=dir.path().join("cli");std::fs::create_dir_all(home.join("credentials")).unwrap();
 std::fs::write(home.join("credentials/kimi-code.json"),cred("a","latest-a").bytes()).unwrap();
 std::fs::write(home.join("config.toml"),"default_model = 'my-model'\n[providers.'managed:kimi-code']\ntype='kimi'\nbase_url='https://api.kimi.com/coding/v1'\n[providers.'managed:kimi-code'.oauth]\nstorage='file'\nkey='oauth/kimi-code'\n[permission]\ncustom='keep'\n").unwrap();
 let mut repo=Repository::open(dir.path().join("vault")).unwrap();let a=repo.import(cred("a","old-a"),"A",None).unwrap();let b=repo.import(cred("b","token-b"),"B",None).unwrap();
 let cli=CliAdapter::from_home(home,"2.1.1").unwrap();(dir,SwitchService::new(repo,cli),a,b)
}
#[test] fn syncs_latest_active_tokens_before_switch(){let (_dir,mut svc,a,b)=setup();svc.switch_to(&b).unwrap();assert_eq!(svc.repo.credential(&a).unwrap().refresh(),"latest-a");assert_eq!(svc.cli.read_active().unwrap().unwrap().refresh(),"token-b");assert!(svc.cli.config().unwrap().contains("custom='keep'"));}
#[test] fn lock_contention_writes_nothing(){let (_dir,mut svc,_a,b)=setup();let _lock=OperationLock::acquire(&svc.repo.root).unwrap();assert!(svc.switch_to(&b).is_err());assert_eq!(svc.cli.read_active().unwrap().unwrap().refresh(),"latest-a");}
#[test] fn preserves_unrelated_toml(){let merged=merge_config("default_model='custom'\n[tools]\nmy_tool=true\n","[providers.'managed:kimi-code']\ntype='kimi'\nbase_url='https://api.kimi.com/coding/v1'\n","mainland-cn","kimi-code").unwrap();assert!(merged.contains("my_tool=true"));assert!(merged.contains("default_model='custom'"));assert!(merged.contains("oauth/kimi-code"));}
