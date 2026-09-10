use hermes_lite::{events::EventLog, harness::HarnessStore, learning};
use tempfile::TempDir;

#[test]
fn event_log_is_durable_and_learning_clusters_failures() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let log = EventLog::new(dir.path())?;
    let run = log.start("read a file")?;
    log.record(&run, "tool", "workspace path rejected: ../secret", Some(false))?;
    log.record(&run, "tool", "workspace path rejected: ../other", Some(false))?;
    log.finish(&run, "failed", false, Some("workspace path rejected"))?;
    let events = log.recent(20)?;
    assert!(log.path().exists());
    let report = learning::analyze(&events);
    assert!(!report.failures.is_empty());
    assert!(report.failures.iter().any(|f| f.suggested_action.contains("workspace")));
    Ok(())
}

#[test]
fn harness_versions_and_rolls_back() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let store = HarnessStore::new(dir.path())?;
    let first = store.initialize()?;
    let mut second = first.clone();
    second.notes = "Revised after a tested failure pattern.".into();
    let second = store.revise(second)?;
    assert_eq!(second.version, 2);
    let restored = store.rollback(1)?;
    assert_eq!(restored.version, 1);
    Ok(())
}
