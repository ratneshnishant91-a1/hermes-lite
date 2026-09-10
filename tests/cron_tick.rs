use hermes_lite::{cron,store::Store,tools::Tools,Config};
use std::fs;
use tempfile::TempDir;

#[test] fn cron_tick_executes_and_records() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("hermes.db");
    let ws = dir.path().join("ws"); fs::create_dir_all(&ws)?;
    let cfg = Config { workspace_root: ws.display().to_string(), db_path: db_path.display().to_string(), ..Default::default() };
    let store = Store::open(&cfg.db_path)?;
    let job = cron::add(&store, "tick-test", "every 1s", "echo ok >> tick.log")?;
    assert!(job.enabled);
    let tools = Tools::new(&cfg)?;
    let executed = cron::tick(&store, &tools)?;
    assert_eq!(executed, 1);
    let log = cfg.workspace_root.join("tick.log");
    assert!(log.exists());
    Ok(())
}
