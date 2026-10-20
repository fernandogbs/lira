# Contributing to Lira

Thank you for helping build Lira.

## Developer Certificate of Origin (DCO) 1.1

By contributing, you agree that your contributions are licensed under the project license (MIT OR Apache-2.0) and that you certify the DCO 1.1 statement.

Every commit must include a `Signed-off-by` line matching the commit author:

```bash
git commit -s -m "your message"
```

CI runs `scripts/check-dco.sh` on pull requests. Locally:

```bash
DCO_BASE=main ./scripts/check-dco.sh
```

## Checks

```bash
SKIP_DCO=1 ./scripts/check.sh
```

## Adding an agent adapter

1. Implement `lira_adapter_api::AgentAdapter` in a new crate under `crates/adapters/`.
2. Run the shared contract against your adapter:

```rust
use std::sync::Arc;
use lira_adapter_api::{run_adapter_contract, FakeAdapter};

#[tokio::test]
async fn contract() {
    let adapter: Arc<dyn AgentAdapter> = Arc::new(YourAdapter::new());
    run_adapter_contract(adapter).await.unwrap();
}
```

Use `FakeAdapter` in `lira-adapter-api` as a reference for expected behavior and event shapes.
