# Service Level Objectives

> Write the SLO before the feature. These are the targets the hosted RustAG cloud
> product commits to. Local/CLI stagenets are best-effort (they run on the user's
> own machine) — every SLO below applies to the **hosted control plane and hosted
> stagenets**, not the open-source local CLI.
>
> Status of each target: **Target** = committed goal, instrumentation in place;
> **Aspirational** = goal set, measurement not yet wired (tracked in
> [known-gaps.md](./known-gaps.md)). Targets are intentionally modest — this is an
> early-stage product and under-promising is correct.

## Availability

| Objective | Target | Status |
| --- | --- | --- |
| Control-plane API (`/v1/*`) uptime | 99.5% monthly | Target |
| Cloud stagenet creation succeeds within 30s | 99% of attempts | Target — enforced by the orchestrator's health-gated start (it deletes the row and returns an error if the stagenet does not report healthy within the start timeout, so a "created" stagenet is always a reachable one) |

## Latency

| Objective | Target | Status |
| --- | --- | --- |
| `getAccountInfo` (cache hit) | p99 < 50 ms | Target |
| `getAccountInfo` (cache miss, cold mainnet fetch) | p99 < 2 s | Target |
| Oracle price staleness | p99 < 2 s with the realtime push mirror (down from up-to-30s polling) | Target — push path built ([`realtime.rs`](../crates/rustag-mirror/src/realtime.rs)); the measured before/after number is still to be recorded |
| Stagenet wake-from-sleep | p99 < 15 s | Aspirational — sleep-after-idle / wake-on-request is not yet built (single-node MVP keeps stagenets resident) |

## Isolation

| Objective | Target | Status |
| --- | --- | --- |
| Cross-tenant data-access incidents | **Hard zero** (not a percentage) | Target — enforced today by tenant-scoped queries + ownership checks + per-tenant process isolation; hardware (Kata) isolation is the production hardening (see [threat-model.md](./threat-model.md)) |

The isolation objective is the one number that is a hard zero. Every control-plane
query is scoped to the authenticated `tenant_id`, a stagenet lookup that does not
match the caller's tenant returns `NotFound` (so one tenant cannot even probe
another's slugs), and each stagenet runs in its own process and data directory.

## Data durability

| Objective | Target | Status |
| --- | --- | --- |
| Stagenet state persistence | Persisted to the datastore, daily backups, ≥7-day retention | Aspirational — state is persisted (SQLite today, Postgres on the swap path); automated backups are an operational task for the hosted deployment |

## Error budget & review

- The error budget for the control-plane API is the inverse of the availability
  target (0.5%/month). When the budget is exhausted, ship reliability work before
  features.
- These SLOs are reviewed every grant-reporting cycle. The honest answer to "are
  you meeting your SLO?" is part of the Phase 2 grant report — see
  [known-gaps.md](./known-gaps.md) for what is measured vs. aspirational.

## Failure-mode commitments

For every external dependency, the documented behavior on its failure (the "name
the failure mode for every dependency" gate):

| Dependency | If it fails | Behavior |
| --- | --- | --- |
| Mainnet RPC (cold fetch) | unreachable / errors | Serve stale cached data with a warning; never panic. Cache miss with no upstream = explicit error to the caller, not a hang. |
| Realtime WebSocket stream | disconnects mid-test | Caller logs and reconnects; CLEAN accounts keep their last value (and the poll loop remains as a floor); **DIRTY/PINNED accounts are never touched** regardless. |
| Datastore | write fails | Surfaced as an error to the API caller; the orchestrator frees reserved ports/slug so a failed create leaks nothing. |
| A hosted stagenet process | crashes | Marked stopped on next reconciliation; `kill_on_drop` prevents orphans on control-plane shutdown. |
