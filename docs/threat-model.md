# Threat Model — RustAG Cloud (Phase 2)

> One page. Scope: the **hosted, multi-tenant** product. The open-source local CLI
> runs entirely on the user's own machine and is out of scope (the user is their
> own trust boundary).

## Tenants

A **tenant** is an organization with an account on the hosted platform. A tenant
owns API keys and one or more **stagenets** (hosted, mainnet-mirroring Solana
environments). Tenants are mutually distrusting: a tenant may run **arbitrary,
untrusted Solana program bytecode** inside their stagenet. That is the product —
people test unaudited code here precisely so they don't test it on mainnet.

## Assets to protect

1. **Tenant A's stagenet state** (accounts, transactions, overrides) from tenant B.
2. **Tenant A's API keys** from disclosure (incl. via logs).
3. **The host / control plane** from any tenant's executing program.
4. **Upstream RPC credentials** (Helius/Triton keys) from all tenants.

## Trust boundaries

```
 Untrusted          │ Semi-trusted              │ Trusted
 ───────────────────┼───────────────────────────┼──────────────────────────
 Tenant's program   │ Stagenet runtime process  │ Control plane (rustag-cloud)
 bytecode, RPC      │ (one per stagenet, runs   │ Datastore
 input, dashboard   │  the tenant's code)       │ Upstream RPC credentials
 requests           │                           │
```

The hard boundary is **between one stagenet runtime and everything else**. A
stagenet executes adversarial code, so it is treated as hostile.

## Key scenario: tenant A's transaction tries to read tenant B's account

This is the question the gate asks directly. Defenses, in depth:

1. **Separate state stores.** Each stagenet has its own account store / data
   directory. There is no shared account namespace — tenant A's stagenet has no
   row for, route to, or handle on tenant B's accounts. A read simply finds
   nothing (or lazily mirrors *mainnet*, which is public data, not B's state).
2. **Control-plane scoping.** Every `/v1/*` query is filtered by the authenticated
   `tenant_id`. A stagenet lookup that doesn't match the caller's tenant returns
   `NotFound`, so A cannot even enumerate B's slugs. (Postgres Row-Level Security
   is the planned second, independent enforcement layer — see
   [known-gaps.md](./known-gaps.md).)
3. **Process isolation (today).** Each stagenet is a separate OS process with its
   own working directory; one cannot open another's database file by path because
   it doesn't know or share it.
4. **Hardware isolation (production hardening).** The
   [`infra/kubernetes`](../infra/kubernetes) manifests run each stagenet pod under
   `runtimeClassName: kata` (Firecracker microVM). This is the answer to a *kernel*
   exploit from untrusted bytecode: a shared-kernel container escape would expose
   every co-tenant, a microVM escape requires breaking the hypervisor. Pods get
   `automountServiceAccountToken: false` and per-tenant CPU/memory quotas.

## Other threats and mitigations

| Threat | Mitigation |
| --- | --- |
| API-key theft | Keys are SHA-256-digested at rest, shown once, tenant-scoped, revocable. |
| Secret leakage via logs & API | Upstream RPC keys are config, not logged; `--log-format json` keeps fields structured for redaction; **do not** `tracing` the mainnet/realtime URLs (they carry `?api-key=`). API read paths (`/api/stagenet` and the cloud stagenet responses) run the URL through `redact_url`, so no `?api-key=` or path-embedded key is ever returned to a browser (regression-tested). A log-scanning test is a tracked gap. |
| Resource exhaustion (noisy neighbor) | Per-tenant stagenet quota in the control plane; per-pod CPU/mem limits (K8s, roadmap). On a public demo (`RUSTAG_DEMO_MODE`) airdrops are capped (100 SOL, both REST and JSON-RPC paths) and the state-mutating / mainnet-quota-draining routes (`override`, `preload`, schedule writes) are refused; aggregate-balance `u64` overflow is separately guarded by `saturating_add`. Inbound per-IP HTTP rate limiting is a tracked gap. |
| Cross-tenant via the proxy | The reverse proxy resolves `/{slug}` to a stagenet only after an ownership check; slugs are unguessable-by-enumeration because probing returns `NotFound`. |
| Malformed input (base58, RPC params) | Validated at the edge; a bad pubkey errors cleanly, never panics the process. |
| Upstream RPC outage | Stale-with-warning, never panic (see [slo.md](./slo.md) failure-mode table). |

## Out of scope (named, not solved here)

- DoS at the network edge (handled by the ingress / CDN layer, not application code).
- Side-channel attacks across microVMs (accepted residual risk at this stage).
- Supply-chain compromise of dependencies (mitigated by `cargo deny`/`audit` once
  wired into CI — a tracked gap).

## Validation

The gate's bar — *"someone other than you has tried to break tenant isolation and
failed"* — is met by an explicit cross-tenant test: prove tenant A's API key
cannot read, reach, or delete tenant B's stagenet. That test is the canary; it
must stay green.
