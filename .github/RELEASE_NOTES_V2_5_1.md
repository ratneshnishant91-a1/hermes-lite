# v2.5.1 — Durable learning foundation

## New components

- **EventLog:** Append-only JSONL run events with explicit sync for durability.
- **Learning analysis:** Deterministic failure clustering with actionable suggestions.
- **Harness manifests:** Versioned policy state with explicit revision and rollback.

## Why this matters

This release implements the safe infrastructure for evidence-first improvement: record runs, inspect failure clusters, add regression tests, revise harness deliberately, and roll back on regression. No autonomous code or policy changes.

## Verification

- `cargo test --all-targets` includes durable_learning tests.
- See `docs/V2_5_1.md` for the operating model.
