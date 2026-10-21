# Idle RSS / CPU baseline

Measured with `./scripts/measure-idle.sh` against a release `lira-app` (no listening, popup closed).

| Date | Host | avg RSS (MiB) | Notes |
|------|------|---------------|-------|
| _(fill on first CI/nightly run)_ | | | Fail CI if RSS regresses >20% once a baseline row exists |

CI job `idle` (workflow_dispatch / weekly schedule) builds release and runs a short sample; it uploads the log as an artifact. Hard fail is enabled only when repo variable `IDLE_RSS_BASELINE_MIB` is set.
