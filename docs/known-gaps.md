# Known Gaps

> The production-readiness gate (Phase 1 Part A) requires that every deferred
> item be written down with a reason, rather than silently skipped. This is that
> list. It is deliberately honest — a reviewer should be able to read it and know
> exactly where the MVP ends and the production roadmap begins.

RustAG implements the **StageSVM Phase 2 spec** with a set of deliberate MVP
substitutions: every Phase 2 subsystem is present and working, but several use a
simpler, single-node technology than the spec's eventual target. In each case the
chosen implementation satisfies the *same contract* as the target, so the swap is
additive (a new backend behind an existing interface), not a rewrite. The
substitutions and their swap paths are below.

Legend: **Done** · **Substituted** (works, simpler tech, swap path noted) ·
**Deferred** (not built; reason given).

---

## Phase 2 architecture substitutions

### Streaming mirror — `accountSubscribe` WebSocket instead of native Yellowstone gRPC — Substituted
- **Built:** [`crates/rustag-mirror/src/realtime.rs`](../crates/rustag-mirror/src/realtime.rs) opens a WebSocket to a mainnet RPC, `accountSubscribe`s to the watched set, and pushes `RemoteAccount` updates over an `mpsc` channel — sub-second propagation, replacing Phase 1's 30s poll. Feature-gated (`--features realtime`).
- **Why this and not `yellowstone-grpc-client`:** the standard Solana pub/sub protocol is served by every Geyser/Yellowstone-backed provider (Helius, Triton), so one implementation points at any endpoint with zero provider lock-in and no `protoc`/`tonic` build dependency. It delivers the headline Phase 2 win (push vs poll) today.
- **Swap path:** the consumer (`rustag_core::spawn_realtime_apply`) is source-agnostic — it consumes `RemoteAccount`s from an `mpsc::Sender`. A native Yellowstone gRPC producer satisfies the identical contract; adding it is a new producer module, not a change to the apply path or the dirty/clean invariant.
- **Not yet done:** live filter updates on an open subscription (currently the watch set is fixed at connect); automatic reconnect is the caller's responsibility (documented), not built into the client.

### Datastore — SQLite + moka instead of Postgres + Redis — Substituted
- **Built:** SQLite data plane ([`001_initial.sql`](../migrations/001_initial.sql), [`002_phase2.sql`](../migrations/002_phase2.sql)) and an in-process cache. The DDL is written Postgres-portable (ISO-8601 `TEXT` timestamps, `INTEGER` booleans, no SQLite-only types) and the `metrics` table is shaped to become a TimescaleDB hypertable on `recorded_at`.
- **Why deferred:** Postgres + Redis are mandatory only once there are concurrent writers across multiple API replicas. The current control plane is single-node, so SQLite (single-writer) and moka (in-process) are correct for the MVP and remove an operational dependency.
- **Swap path:** `sqlx` is already the query layer; the workspace pin can add the `postgres` feature alongside `sqlite`. The schema migrates as-is. Redis replaces moka behind the existing cache interface.
- **Not yet done:** the actual Postgres migration, Postgres Row-Level-Security policies (the spec's defense-in-depth second layer — see [threat-model.md](./threat-model.md)), the one-time SQLite→Postgres migration script, and a `usage_events` table (billing is deferred, below).

### Multi-tenant isolation — child-process isolation instead of Kata Containers + Kubernetes — Substituted
- **Built:** [`crates/rustag-cloud`](../crates/rustag-cloud) is a working multi-tenant control plane. Each hosted stagenet runs as an **isolated child `rustag` process** with its own working directory (private SQLite DB + PID file), its own allocated `(rpc, ws, api)` port triple, and `kill_on_drop` supervision. Tenants authenticate with API keys; every query is tenant-scoped; per-tenant stagenet quotas are enforced; one tenant cannot probe or reach another's slugs.
- **Why this and not Kata/K8s now:** process isolation is the right strength for a single-node MVP and for the open-source local product. Hardware-level (Firecracker/Kata) isolation matters specifically when hosting *other people's untrusted program bytecode on shared infrastructure* — that is a hosted-cluster concern, addressed by the `infra/` manifests, not the control-plane code.
- **Swap path:** the `Orchestrator` interface (`create_and_start` / `stop`) is the seam. A `kube-rs`-backed implementation that creates pods with `runtimeClassName: kata` (see [`infra/kubernetes/stagenet-pod-template.yaml`](../infra/kubernetes/stagenet-pod-template.yaml)) drops in behind the same interface. The K8s/Kata manifests are scaffolded under [`infra/`](../infra); the live cluster is not stood up here.
- **Not yet done:** the `kube-rs` orchestrator implementation, a running K8s cluster with the Kata RuntimeClass, sleep-after-idle / wake-on-request pod lifecycle.

### Auth & billing — API keys instead of Clerk + Stripe — Substituted (auth) / Deferred (billing)
- **Built:** API-key auth with SHA-256-digested keys, shown once, tenant-scoped (`/v1/signup`, `/v1/api-keys`).
- **Why deferred:** Clerk/Stripe are SaaS-launch concerns. The spec's own guidance is "buy auth, don't build it" *at launch* — the API-key layer is the correct primitive for the CLI/CI product and for the first manually-onboarded customers.
- **Not yet done:** Clerk JWT middleware, Stripe usage metering + webhooks, the `usage_events` table, free-tier sleep-after-24h enforcement.

### Observability — `tracing` spans without an OTLP exporter — Substituted
- **Built:** every RPC method and major function is `#[tracing::instrument]`-ed; a JSON time-series analytics endpoint (`/api/metrics`) and a background sampler exist; `--log-format json` structured logging is available (see below).
- **Why deferred:** the OpenTelemetry exporter + Grafana stack is a deployment concern, not a code concern. Phase 1's tracing discipline means wiring OTLP later is additive — point existing spans at a backend, don't retrofit instrumentation.
- **Not yet done:** the `opentelemetry-otlp` exporter, the self-hosted Tempo/Loki/Prometheus stack (Grafana dashboards/alerts are scaffolded under [`infra/grafana`](../infra/grafana)), and a Prometheus-format `/metrics` scrape endpoint (the current `/api/metrics` returns JSON time-series, not Prometheus exposition format).

### Anchor integration — TypeScript provider instead of an `anchor test` CLI plugin — Substituted
- **Built:** [`@rustag/anchor-plugin`](../packages/anchor-plugin) gives a custom Anchor provider + `EphemeralStagenet` lifecycle, so Anchor tests run against a mainnet-mirroring stagenet.
- **Not yet done:** a native `anchor test --provider.cluster rustag` subcommand (Anchor plugin hook), versus the current "set the provider in your test file" approach.

---

## Phase 1 Part A items still open

| Item | Status | Note |
| --- | --- | --- |
| `proptest` property tests for `pre_load_accounts_for_tx` | Deferred | Covered by example-based unit + integration tests; fuzzing not yet added. |
| `loom` concurrency model check on the dirty/clean critical section | Deferred | Critical section uses `DashMap`/`RwLock`; correctness is unit-tested, not model-checked. |
| `getVersion`, `getHealth` RPC methods | **Done** | Both implemented in [`jsonrpc.rs`](../crates/rustag-rpc/src/jsonrpc.rs). |
| `rustag doctor` preflight command | **Done** | Checks data-dir writability, DB openability, mainnet-RPC reachability, port availability. |
| `--log-format json` structured logging | **Done** | Global CLI flag + `RUSTAG_LOG_FORMAT` env. |
| Prometheus `/metrics` endpoint | Deferred | `/api/metrics` returns JSON time-series; Prometheus exposition format not yet emitted. |
| `cargo clippy -- -D warnings` + `cargo fmt --check` gate | **Done** | Both pass with zero warnings and are enforced in CI (the `rust` job). |
| `cargo audit` / `cargo deny` | Deferred | Not yet wired into CI; no advisory triage doc. |
| Examples run in CI | Deferred | Examples exist under [`examples/`](../examples) but the CI workflow does not execute them. |
| Publish to crates.io / npm | Deferred | Not yet published. |
| `@solana/kit`, `anchor test` e2e, `solana-client` (Rust) compatibility | Partially done | Validated against `@solana/web3.js`; the other client matrices are not yet exercised. |
| A.7 performance baseline numbers | Deferred | The `/api/metrics` plumbing exists to capture them; the table in the Phase 2 prompt is not yet filled in. |

---

## What is fully done (no gap)

- **Activity Scheduler** ([`crates/rustag-scheduler`](../crates/rustag-scheduler)) — cron + `@every`/duration expressions, presets via airdrop / transfer / raw-tx replay, persisted to the `schedules` table.
- **Simulation framework** ([`crates/rustag-sim`](../crates/rustag-sim)) — fork-isolated replay, stress, and side-by-side `compare`, never mutating the base stagenet.
- **Analytics** — background sampler + `/api/metrics` time-series + dashboard sparklines.
- **GitHub Action** ([`.github/actions/rustag`](../.github/actions/rustag)) — ephemeral per-PR stagenet, runs your command, posts a PR summary, tears down.
- **SDK + dashboard** — TypeScript SDK and Next.js dashboard, both type-checking in CI.
- The Phase 1 invariant — **a `Dirty` or `Pinned` account is never overwritten by any sync** — holds on the new push path, not just the poll path.
