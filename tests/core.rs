use hermes_lite::{cron,math,workspace::Workspace};
#[test] fn math_words_work(){assert_eq!(math::evaluate("add 2 and 3").as_deref(),Some("5"));}
#[test] fn interval_parses(){assert_eq!(cron::parse_interval("every 5m").unwrap(),300);}
#[test] fn workspace_blocks_escape(){let t=tempfile::tempdir().unwrap();let w=Workspace::new(t.path()).unwrap();assert!(w.resolve("../x").is_err());}
