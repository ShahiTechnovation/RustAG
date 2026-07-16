# RustAG → GroundTruth: the pivot plan

**Attested pre-execution assurance for Solana privileged operations.**

> Rehearse any privileged Solana transaction — a Squads multisig proposal, a program
> upgrade, a treasury move — against faithful mainnet state in a sealed, deterministic
> sandbox, and get a signed diff every signer can independently re-verify **offline**.
> The execution preview that survives a compromised UI.

This document is the definitive restructuring plan for RustAG. It is the output of a
26-brief / 5-proposal / 15-verdict research-and-judging process (see
[the appendix](#appendix-how-this-plan-was-chosen)). It supersedes the "persistent
mainnet-mirroring staging environment" positioning in the current README.

---

## 1. Why we are pivoting

RustAG was built as *"the Solana equivalent of Tenderly Virtual TestNets"* — a
LiteSVM-based, lazy-mainnet-mirroring local network. That exact category is now **won and
Foundation-canonized by Surfpool** (txtx): the repo moved into the `solana-foundation`
GitHub org, it is named the official integration-testing tier of the Foundation's own
testing pyramid, and it ships a major release roughly monthly (51 releases through
v1.4.0). Continuing to build "a better local fork" means racing a funded, distribution-
backed incumbent on its home turf. Any thin adjacent feature we ship is absorbed within a
quarter.

**The strategic conclusion:** stop competing on the pre-deploy local loop. Move to the
tier Surfpool explicitly does not occupy and cannot easily reach — the **pre-execution
assurance** layer that sits between a privileged action being *proposed* and it being
*signed/deployed*, and whose entire value is the inverse of Surfpool's: not mutable state
and 26 cheatcodes, but **pinned, content-addressed state and deterministic, offline
re-execution with cryptographic proof.**

### The problem, in incidents

- **Drift — ~$285M, April 2026.** Audited, uncompromised contracts. Attackers used durable
  nonces + social engineering to get multisig members to **blind-sign** transactions that
  "appeared routine but carried hidden authorizations." CoinDesk named the failure *"the
  gap between onchain correctness and offchain human trust — a gap no smart-contract audit
  or monitoring tool is built to cover."* The Foundation's own STRIDE/SIRN monitoring
  admits it *"would not have caught"* it, because the transactions were valid by design.
- **Loopscale — ~$5.8M.** A program **upgrade** that skipped a single program-ID
  validation shipped a pricing regression. No pre-deploy gate replayed real mainnet
  traffic against the candidate build to catch it.
- **The daily reality.** SEAL's official Squads guidance still tells signers to
  hand-decode little-endian hex. Squads' own tooling yields only SOL/token balance diffs.
  There is **no** deterministic, re-runnable, signed pre-execution proof of what a
  privileged Solana transaction will do. On EVM this is a category (Tenderly + Safe); on
  SVM it is defended by a CLI.

### Why this is not a childish Surfpool copy

| | Surfpool | GroundTruth |
|---|---|---|
| Tier | Pre-deploy local dev loop | Pre-**sign** / pre-**deploy** assurance |
| Design center | **Mutable** state (`setAccount`, `timeTravel`, `setSupply`) | **Pinned**, content-addressed, self-contained state |
| Output | A running localnet | A signed, offline-re-executable **EvidenceBundle** |
| Ingests a proposed multisig payload and proves what it does? | No | **Yes — the core unit of work** |
| Attestation / tamper-evidence | None across 51 releases | Ed25519 + SHA-256 Merkle, hash-chained |
| Time travel | Forward-only | Backward, deterministic, branch-of-branch |
| Relationship | — | **Interoperates**: can drive a surfnet as one execution backend |

We do not fork Surfpool or compete on localnet UX. We **interoperate** — GroundTruth can
drive a surfnet (or Agave, or a pinned in-process SVM) behind an `Executor` trait, turning
the Foundation-backed incumbent into a distribution channel — and we fill Surfpool's own
open seam (secp256r1/secp256k1 precompiles, needed for passkey/EVM-wallet proposals).

---

## 2. What we keep (the reuse map)

RustAG is a genuinely well-engineered ~12.7k-LOC workspace. The pivot's leverage is that
its **least-marketed crates are its most valuable** for this direction, and they are
already built and tested.

| Crate | Verdict | Becomes |
|---|---|---|
| `rustag-core` | **KEEP + EXTEND** | Sealed deterministic sandbox (`fork`, `import/export_accounts`, `enable_impersonation`, checkpoint substrate). **ADD `src/fidelity.rs`**: ProgramData dereference + Clock sync — the two execution gaps the audit found. |
| `rustag-attest` | **KEEP (core)** — best-tested (26 tests) | Evidence engine. **ADD `src/evidence.rs`**: `EvidenceBundle` v2. `AuditLog` = per-program chain of custody. |
| `rustag-replay` | **PROMOTE** — was built+tested but *zero dependents* | Load-bearing: `Checkpoint` = pre-state freeze, `Journal`/`execute_and_record` = deterministic rehearsal record, `diff_accounts` = raw diff, `replay_matches_journal` = the **Grade-A re-executability proof**, `Timeline` = incident reconstruction. |
| `rustag-sim` | **REPURPOSE** | `diff.rs` → `Executor` trait + backends. `fuzz.rs` `Invariant`/`FuzzRng` → **invariant-alarm engine** over `(pre,post)`. `exploit.rs` `scan_outcomes` → CPI tree / reentrancy / compute over rehearsal logs. `bundle.rs` atomic semantics → multi-instruction proposal execution. |
| `rustag-mirror` | **REPURPOSE → ingest** | Keep `fetch_multiple`. **ADD** `squads.rs` decoder, `TouchSetResolver`, `MultiRpcFetcher` (N-of-M), `ForwardRecorder` (Yellowstone). |
| `rustag-cli` | **KEEP + EXTEND** | Add `rehearse`, extend `verify`; later `forensics`, `record`. |
| `rustag-rpc` | **KEEP** | Add `POST /rehearse`. |
| `rustag-cloud` | **REPURPOSE (Phase 5)** | Hosted Evidence Registry + tenancy + per-flow metering. |
| `rustag-scheduler` | **DEMOTE** | Internal utility (recorder health, watched-proposal re-fetch). |
| `rustag-compression` | **PARK** | On-chain anchoring of the audit-log/registry head only. |
| `packages/dashboard` | **REPURPOSE** | Signer-review UI + Evidence Explorer. |

**RETIRE:** the "staging clone of mainnet" positioning; the README "real Jupiter/Raydium
execution" claim (the audit's most falsifiable oversell); scheduler-as-product;
cloud-staging-as-SaaS. The workspace description changes from *"mainnet-mirroring staging
environment"* to *"attested pre-execution assurance for Solana privileged operations."*

---

## 3. Architecture

Six components on the existing axum / SQLite / LiteSVM / Ed25519 stack. Every non-UI piece
is a Rust workspace crate.

```
   proposed payload (Squads proposal pubkey | raw base64 message | upgrade build)
                                  │
                                  ▼
 ┌───────────────────────────────────────────────────────────────────────────┐
 │ (1) groundtruth-ingest  (evolved rustag-mirror)                            │
 │     TouchSetResolver: static keys + v0 ALT keys + ProgramData PDAs + Clock │
 │     MultiRpcFetcher:   N-of-M cross-fetch → InputProvenance                │
 │     ForwardRecorder:   (Phase 3) Yellowstone journal of a program's txs    │
 └───────────────────────────────────────────────────────────────────────────┘
                                  │ full account closure
                                  ▼
 ┌───────────────────────────────────────────────────────────────────────────┐
 │ (2) groundtruth-fidelity  (rustag-core::fidelity)                          │
 │     ProgramDataLoader: BPFLoaderUpgradeable → deref ProgramData → load ELF │
 │     ClockSync:         pin Clock sysvar at target slot/blockTime           │
 └───────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
 ┌───────────────────────────────────────────────────────────────────────────┐
 │ (3) rustag-rehearse  — SealedRehearsal two-pass (Grade A)                  │
 │   Pass 1 (discovery): impersonated simulate → fault-in the whole closure  │
 │   Pass 2 (execute):   Checkpoint::capture(pre) → restore offline →        │
 │                       execute_and_record → Journal(post)                  │
 │   replay_matches_journal == true  ⇒  Grade A (offline re-executable)      │
 │   ── diff_accounts(pre,post) → StateDiff                                  │
 │   ── SemanticDiff: decode each changed account into a human claim          │
 │   ── invariant policy over (pre,post) → alarms                            │
 │   ── scan_outcomes(logs) → CPI tree / reentrancy / compute                │
 └───────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
 ┌───────────────────────────────────────────────────────────────────────────┐
 │ (4) groundtruth-evidence  (rustag-attest::evidence)                        │
 │     EvidenceBundle v2: subject / environment / result / prev_bundle_hash   │
 │     signing_digest (domain-tagged, length-prefixed) + Ed25519 signature    │
 │     verify(): offline + re-executable; per-signer, own-RPC drift check     │
 └───────────────────────────────────────────────────────────────────────────┘
                                  │
        ┌─────────────────────────┼──────────────────────────┐
        ▼                         ▼                          ▼
   CLI (rehearse/verify)    PR comment / CI gate       Squads embed / registry
```

**(5) `groundtruth-diff`** (evolved `rustag-sim/diff.rs`) — a trait
`Executor { async fn execute(&mut self, pre_state, tx) -> ExecReport; async fn state_root() }`
with backends `LiteSvmExecutor` (exists), `PinnedSvmExecutor` (subprocess pinning a
`solana-svm` version over IPC, sigverify/blockhash off), `SurfnetExecutor` (Grade B),
`FixtureExecutor` (solfuzz export → Firedancer, conformance). The **upgrade gate** diffs
candidate-vs-deployed build on the *same* executor so substrate infidelity cancels.

**(6) Surfaces** — CLI (`rehearse`, `verify`, `forensics`, `record`); GitHub Action
(`rehearse --upgrade` posts a signed diff+alarm report, fails on divergence); REST/JSON-RPC
(`POST /rehearse`); Next.js signer-review UI + Evidence Explorer; hosted registry +
per-flow metering.

### The two named execution gaps (fidelity)

The audit found the Surfpool-overlapping part was also RustAG's weakest: **real mainnet
programs don't actually execute** because two things are missing. The pivot fixes exactly
these:

1. **ProgramData dereference.** An upgradeable program's executable ELF does not live in
   the program account — it lives in a separate `ProgramData` account referenced by it.
   `fidelity.rs` detects `owner == BPFLoaderUpgradeab1e11111111111111111111111`, parses
   `UpgradeableLoaderState::Program { programdata_address }` (36-byte layout), fetches the
   ProgramData account, and loads the ELF (bytes after the 45-byte header) so LiteSVM can
   execute it.
2. **Clock sync.** Vesting, funding windows, staleness checks, and auction logic all read
   the `Clock` sysvar. `fidelity.rs` pins `Clock` at the target slot/`blockTime` (a fresh
   slot for pre-sign; the transaction's own slot for forensics).

### The fidelity ledger (honesty as a feature)

Every bundle carries an explicit **fidelity grade**, so we never overclaim:

- **Grade A** — deterministically re-executable: a self-contained offline checkpoint +
  journal for which `replay_matches_journal == true`. The verifier reproduces the exact
  post-state root on their own machine.
- **Grade B** — a signed *observation* of an external engine (a surfnet/agave run) that is
  not offline-self-contained.

And the mainnet-state claim is bounded, not overstated: **N-of-M multi-RPC cross-fetch**
records `InputProvenance {endpoints, per-source slot, agreement}`. A bundle proves
*"GroundTruth observed X against this closure, cross-checked across M RPCs,"* never
*"mainnet was X at slot N"* (Solana has no consensus-anchored per-account state proof).
This pre-empts the security-researcher backlash that kills trust products.

---

## 4. Phased roadmap

### Phase 1 — Wedge: `rustag rehearse` for privileged payloads (Weeks 1–6)

**The Drift-shaped product.** Turn a proposed payload into a signed, offline-verifiable
diff with invariant alarms — where today's competition is a CLI.

- **Goals:** ship the sealed pre-sign rehearsal; make `rustag-replay` load-bearing; make
  the README true (ProgramData + Clock). Co-develop with **one committed design-partner
  council** — do not ship a badge, do not build in a vacuum.
- **Deliverables:**
  - `rustag-core::fidelity` — ProgramData dereference + Clock sync.
  - `rustag-rehearse` crate — the two-pass `SealedRehearsal`.
  - `EvidenceBundle` v2 in `rustag-attest` with offline, re-executing `verify`.
  - `rustag-sim::invariants` + `semantic` — authority/upgrade-authority rotation, new
    durable-nonce account, oracle-owner swap, config-byte mutation, treasury floor.
  - `rustag rehearse` and extended `rustag verify` CLI.
  - `TouchSetResolver` + Squads v4 decoder.
  - 3 public signed **reference proofs** decoding the Drift and Loopscale payloads.
  - Journal-based E2E fixtures in CI (closes the audit's "no E2E" gap).
- **Technical detail:** `BPFLoaderUpgradeable` detection →
  `UpgradeableLoaderState::Program{programdata_address}` parse → fetch ProgramData →
  `set_account_no_checks` + `add_program`; `Clock` pinned via `set_sysvar`. Two-pass:
  impersonated discovery `simulate` faults the closure in; `Checkpoint::capture`
  (self-contained `pre_state_root`); `Checkpoint::restore` offline;
  `execute_and_record` → `Journal` (`post_state_root`); impersonation-aware replay proves
  Grade A. Diff via `diff_accounts` + `account_leaf_hash`; alarms via `Invariant`
  combinators; `EvidenceBundle::signing_digest` reuses the manifest domain-tag /
  length-prefix pattern; Ed25519 via `Attestation::create`.

### Phase 2 — Signer verifier, hosted API, Squads embed (Weeks 6–12)

Turn a CLI into the workflow every council runs *before signing*.

- **Goals:** independent per-signer verification that survives a compromised proposer
  device; embedded where signers already are (Squads); metered infrastructure.
- **Deliverables:** web signer-review UI + one-click offline verifier; hosted
  `POST /rehearse` on `rustag-cloud` with API keys + per-flow metering; Squads proposal-URL
  ingestion + embedded "verified execution preview"; N-of-M provenance surfaced in every
  bundle; secp256r1/secp256k1 precompiles (fills Surfpool seam) for passkey/EVM-wallet
  proposals.
- **Technical detail:** the verifier re-derives `payload_hash` from the **on-chain** Squads
  proposal account and rehearses *that*, not what the UI shows; re-fetches the closure from
  the signer's own RPC, flags per-account drift against `closure_merkle_root`, then does an
  offline restore + journal replay and compares `post_state_root`. Hosted service reuses
  the REST bulk-simulation snapshot-under-read-lock isolation.

### Phase 3 — Upgrade-rehearsal CI gate + forward-recorder (Weeks 12–20)

**The Loopscale-killer.** Prove an upgrade changes nothing unintended, using the
protocol's own real mainnet traffic + adversarial substitution.

- **Deliverables:** `ForwardRecorder` (Yellowstone gRPC + `getSignaturesForAddress` /
  `getTransaction` fallback) building a self-contained per-program `Journal` corpus;
  `Executor` trait extracted from `diff.rs` + `PinnedSvmExecutor`; `rustag rehearse
  --upgrade` diffing candidate-vs-deployed on the same executor over the corpus **plus**
  adversarial account/program substitution fuzzing; GitHub Action step posting a signed
  diff+divergence+alarm report and failing on divergence; per-flow pricing.
- **Technical detail:** `PinnedSvmExecutor` is a subprocess pinning `solana-svm` at a
  version, fed explicit pre-state + payload over IPC with sigverify/blockhash off —
  eliminating re-signing, stale-blockhash, and state-root-noise problems. Corpus
  determinism holds because each recorded entry carries the just-in-time pre-state of its
  touched accounts (a self-contained checkpoint). Substitution fuzz reuses `fuzz.rs`'s
  seeded xorshift + `Invariant` machinery.

### Phase 4 — Forensics & counterfactual replay (Months 5–7)

The "after" tense: same engine, up-market to incident response.

- **Deliverables:** `rustag forensics <signature>` — impersonation-based re-execution with
  `Clock` synthesized from the tx's own slot/blockTime; multi-tx incident timelines via
  `Timeline`; counterfactual replay (patch the program on the offline fork, rerun the
  journal, attested verdict); hash-chained evidence bundles for post-mortems / insurance /
  disputes; published deep reconstructions of Drift and Loopscale as sales collateral; 2
  design-partner SIRN-firm engagements; optional cross-runtime conformance via
  `FixtureExecutor` → Firedancer (fundable public good).
- **Technical detail:** historical single-tx reconstruction uses current-state fetch for
  the touched closure with honest per-account drift flags (the failed/admin-tx fast path
  needs no archive); the `ForwardRecorder` corpus supplies full-fidelity pre-state after
  instrumentation. Counterfactual = swap the ProgramData ELF in the offline checkpoint,
  replay the recorded attack `Journal`, diff outcomes, sign a bundle recording both
  binaries in `subject.build_hash`.

### Phase 5 — Standard-setting, compliance, monetization (Months 7–12)

- **Deliverables:** open `EvidenceBundle` spec (in-toto/SLSA-mapped) with external
  governance so competitors implement *our* schema; hosted Evidence Registry GA with
  on-chain-anchored heads; compliance-evidence product for stablecoin/treasury/SPE
  operators (Range precedent); STRIDE listing + SIRN membership + Foundation RFP
  submissions (Pre-Deployment Program Analysis, Program Verification Tooling,
  Post-Deployment Monitoring); usage-based pricing GA (free OSS CLI + metered hosted
  rehearsals + forensics retainers + compliance contracts); SOC2-track hardening.
- **Technical detail:** append-only tenant-scoped registry over the existing SQLite/axum
  stack, hash-chain head anchored on-chain via a `rustag-compression` concurrent Merkle
  tree. Pricing per-flow at the chokepoint (Forta Firewall / Jito pattern), never seats;
  OSS core + paid managed layer (the OpenZeppelin Defender lesson).

---

## 5. Business shape (why this is fundable, not just a grant)

- **Chokepoint pricing.** Charge per screened flow at the multisig/upgrade chokepoint, not
  per seat — crypto teams accept small fees on high-value flows (Forta Firewall / Jito).
- **Post-traumatic budget.** The wedge buyer is a post-Drift Squads council with real
  security budget and a changed signing process — not the zero-WTP local-dev-tool user.
- **Foundation money as GTM.** STRIDE tool-listing + SIRN membership convert grant
  orientation into Foundation-funded revenue + warm enterprise leads; three standing RFPs
  map onto this verbatim.
- **Expansion ladder:** pre-sign rehearsal → upgrade CI gate → SIRN forensics retainers →
  institutional compliance evidence (Range's $8.3M Series A proves that budget is real).
- **Comparable:** Tenderly proved this is a category on EVM (usage-based revenue,
  hundreds of thousands of debugged transactions) and has **no SVM presence**.

---

## 6. Risks & mitigations

1. **Execution fidelity** (LiteSVM ≠ Agave/Firedancer). → The upgrade gate diffs
   candidate-vs-deployed on the *same* executor so infidelity cancels; every bundle carries
   a fidelity grade and cross-checks on-chain meta; precompiles (P2), `PinnedSvmExecutor`
   (P3), solfuzz/Firedancer backend (P4). We never claim "safe," only "here is the
   provable diff."
2. **Overclaim risk** (no consensus-anchored state proofs). → N-of-M multi-RPC provenance
   bounds input trust; the signer re-fetches and re-executes on their own RPC; marketed as
   "deterministically re-verifiable," never as a ZK/consensus proof.
3. **The real Drift vector is a compromised proposer device.** → The verifier re-derives
   the payload hash from the **on-chain** proposal account and rehearses that, not what any
   UI shows; per-signer independent verification is the design center.
4. **Squads could build it in-flow.** → Partner and embed as their "verified preview,"
   move fast, out-depth on full account-state diffs + invariant policy + attestation + the
   record-forward upgrade gate; the open format makes us the neutral verifier they adopt.
5. **Willingness-to-pay** (devs expect free tools; OZ Defender died as SaaS). → Lead with
   the pre-sign product that has budget, gate Phase 1 on a committed design partner, price
   per flow, keep an OSS core + paid managed registry + forensics retainers.
6. **Forward-recorder data dependency.** → The pre-sign wedge needs *no* historical data
   (current-state fetch of a bounded closure); upgrade gate + forensics sell prospectively
   ("instrument now, get perfect attested replay forever") behind a multi-provider
   abstraction — turning the one hard data problem into the data moat.

---

## Appendix: how this plan was chosen

Five pivot proposals were generated from distinct angles and each scored by a 3-lens
adversarial panel (skeptical VC / protocol lead engineer / staff SVM engineer), out of 50:

| Proposal | Angle | Score |
|---|---|---|
| **Blackbox** — flight recorder for deployed programs | post-deploy forensics | 32.7 |
| **Testament** — evidence layer for SVM programs | verify/attest | 31.3 |
| **Vigil** — production safety net | security-net | 31.0 |
| **Refract** — local data plane (DAS/compression) | contrarian | 30.0 |
| **Parallax** — SVM conformance & divergence layer | svm-networks | 28.3 |

**GroundTruth** is the synthesis: it takes the highest-scoring *pain* (the corpus's
loudest, freshest, incident-priced demand — pre-sign rehearsal, per five independent
briefs), fixes the flaws every proposal shared (wedge ≠ moat; WTP; Squads-absorption;
fidelity overclaim) with the judges' steelman fixes, and grafts the best compatible ideas:
Blackbox's forensics as a top-of-funnel credibility demo powered by the *same* engine (not
impossible archive reconstruction); Vigil's slot-anchored forward-recording spine;
Testament's explicit fidelity grading; Parallax's `Executor` trait + `PinnedSvmExecutor`
and optional cross-runtime conformance as a fundable public good; and the open,
in-toto/SLSA-mapped `EvidenceBundle` format as a standard-setting moat.
