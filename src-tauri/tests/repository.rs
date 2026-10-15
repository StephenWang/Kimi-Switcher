use kimi_switcher::{cli::Credential,storage::{Repository,crypto}};
use serde_json::json;
fn cred(access:&str,refresh:&str)->Credential { Credential::parse(&serde_json::to_vec(&json!({"access_token":access,"refresh_token":refresh,"expires_at":2000000000})).unwrap()).unwrap() }
#[test] fn dpapi_round_trip_and_no_plaintext(){
 let encrypted=crypto::protect(b"very-secret-refresh-token").unwrap();
 assert!(!encrypted.windows(25).any(|w|w==b"very-secret-refresh-token"));
 assert_eq!(crypto::unprotect(&encrypted).unwrap(),b"very-secret-refresh-token");
 assert!(crypto::unprotect(b"corrupt").is_err());
}
#[test] fn duplicate_import_updates_tokens(){
 let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();let mut repo=Repository::open(dir.path().to_owned()).unwrap();
 let id=repo.import(cred("first","same-refresh"),"账号A",None).unwrap();
 assert_eq!(repo.import(cred("updated","same-refresh"),"账号A",None).unwrap(),id);
 assert_eq!(repo.views(None).len(),1);assert_eq!(repo.credential(&id).unwrap().access(),"updated");
 let reopened=Repository::open(dir.path().to_owned()).unwrap(); assert_eq!(reopened.views(None).len(),1);
 assert!(!String::from_utf8_lossy(&std::fs::read(dir.path().join("accounts.dpapi")).unwrap()).contains("same-refresh"));
}
#[test] fn ambiguous_identity_requires_explicit_choice(){
 let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();let mut repo=Repository::open(dir.path().to_owned()).unwrap();
 let id=repo.import(cred("one","r-one"),"账号A",None).unwrap();
 assert_eq!(repo.import(cred("two","r-two"),"账号B",None).unwrap_err().code,"identity_choice");
 repo.import(cred("two","r-two"),"账号A",Some(&id)).unwrap();assert_eq!(repo.views(None).len(),1);
}
#[test] fn rename_rejects_empty_long_duplicate(){
 let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();let mut repo=Repository::open(dir.path().to_owned()).unwrap();
 let a=repo.import(cred("one","r-one"),"账号A",None).unwrap();
 let b=repo.import(cred("two","r-two"),"账号B",Some("new")).unwrap();
 assert!(repo.rename(&a," ").is_err());assert!(repo.rename(&a,&"名".repeat(41)).is_err());assert!(repo.rename(&b,"账号A").is_err());
 repo.rename(&b," 新名称 ").unwrap(); assert_eq!(repo.views(None)[1].name,"新名称");
}
#[test] fn remove_current_keeps_cli_file(){
 let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();let cli=dir.path().join("active.json");std::fs::write(&cli,b"keep-me").unwrap();
 let mut repo=Repository::open(dir.path().join("vault")).unwrap();let id=repo.import(cred("one","r-one"),"账号A",None).unwrap();
 repo.remove(&id).unwrap();assert!(repo.views(None).is_empty());assert_eq!(std::fs::read(&cli).unwrap(),b"keep-me");
}
#[test] fn corrupt_vault_does_not_silently_reset(){let dir=tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();std::fs::write(dir.path().join("accounts.dpapi"),b"broken").unwrap();assert!(Repository::open(dir.path().to_owned()).is_err());}
