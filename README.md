# RustAG

**Attested pre-execution assurance for Solana privileged operations.**

> Rehearse any privileged Solana transaction — a Squads multisig proposal, a program
> upgrade, a treasury move — against faithful mainnet state in a sealed, deterministic
> sandbox, and get a signed diff every signer can independently re-verify **offline**.
> The execution preview that survives a compromised UI.

---

## The Problem

```
Drift   — $285M (April 2026). Audited, uncompromised contracts. Attackers used
           durable nonces + social engineering to get multisig members to blind-sign
           transactions carrying hidden authority rotations.
Loopscale — $5.8M. A program upgrade skipped a single program-ID validation,
           shipping a pricing regression no pre-deploy gate caught.
```

Today, every Squads multisig signer sees **SOL/token balance diffs at best** — or hand-decodes little-endian hex per SEAL's official guidance. There is **no** deterministic, re-runnable, signed pre-execution proof of what a privileged Solana transaction will do.

On EVM, Tenderly + Safe make this a funded category. On SVM, it's defended by a CLI.

**RustAG fills this gap.**

---

## How It Works

RustAG wraps [LiteSVM](https://github.com/LiteSVM/litesvm) with a **lazy mainnet
account mirror** and a **sealed two-pass rehearsal** algorithm:

```
Input: a proposed payload (Squads proposal pubkey | raw base64 message | upgrade build)
  │
  ▼
┌──────────────────────────────────────────────────────────────────────┐
│ (1) Ingest & Resolve                                                │
│   • TouchSetResolver: static keys + v0 ALT + ProgramData PDAs      │
│   • MultiRpcFetcher: N-of-M cross-fetch with InputProvenance        │
│   • SquadsDecoder: fetch and decode Squads v4 proposal payloads     │
└──────────────────────────────────────────────────────────────────────┘
  │
  ▼
┌──────────────────────────────────────────────────────────────────────┐
│ (2) Fidelity Layer                                                   │
│   • ProgramData dereference: follow BPFLoaderUpgradeable → ELF      │
│   • Clock sync: pin Clock sysvar at target slot/blockTime            │
└──────────────────────────────────────────────────────────────────────┘
  │
  ▼
┌──────────────────────────────────────────────────────────────────────┐
│ (3) Sealed Rehearsal (two-pass, Grade A)                             │
│   Pass 1 (discovery): simulate → fault in the full account closure   │
│   Pass 2 (execute): Checkpoint(pre) → restore offline → execute →   │
│     Journal(post) → replay_matches_journal == Grade A                │
│   • diff_accounts(pre, post) → StateDiff                             │
│   • SemanticDiff: human-readable claims per changed account          │
│   • Invariant policy evaluation → alarms                             │
│   • Exploit scan: CPI tree / reentrancy / compute analysis           │
└──────────────────────────────────────────────────────────────────────┘
  │
  ▼
┌──────────────────────────────────────────────────────────────────────┐
│ (4) Evidence Engine                                                   │
│   • EvidenceBundle v2: payload_hash + pre/post_state_root + findings │
│   • Ed25519 signature + domain-tagged signing digest                 │
│   • Offline-verifiable: any signer re-executes on their own machine  │
└──────────────────────────────────────────────────────────────────────┘
  │
  ├───────────────┬─────────────────┬───────────────────┐
  ▼               ▼                 ▼                   ▼
CLI (rehearse)  REST API       CI gate (GH Action)  Signer UI
```

### The Fidelity Grade

Every bundle carries an explicit **fidelity grade** — we never overclaim:

- **Grade A** — deterministically re-executable: a self-contained offline checkpoint +
  journal that any verifier reproduces exactly on their own machine.
- **Grade B** — a signed *observation* of an external engine that is not
  offline-self-contained.

---

## Quick Start

```bash
# 1. Build the CLI
cargo build --release            # produces target/release/rustag

# 2. Rehearse a Squads multisig proposal
rustag rehearse \
  --proposal <SQUADS_PROPOSAL_PUBKEY> \
  --rpc https://mainnet.helius-rpc.com/?api-key=YOUR_KEY \
  --signer ./groundtruth-key.json

# 3. Verify the evidence bundle (any signer, offline)
rustag verify evidence-bundle.json \
  --rpc https://my-own-rpc.com \
  --proposal <SQUADS_PROPOSAL_PUBKEY>

# Output: VALID ✓ — Grade A (deterministically re-executable)
#         Pre-state root matches: ✓
#         Post-state root matches: ✓
#         Signature valid: ✓
#         Alarms: 0
```

### Rehearse a raw payload

```bash
# Rehearse a base64-encoded transaction
rustag rehearse \
  --payload <BASE64_VERSIONED_TX> \
  --rpc https://mainnet.helius-rpc.com/?api-key=YOUR_KEY \
  --signer ./groundtruth-key.json
```

### Upgrade-rehearsal CI gate (Phase 3)

```yaml
# .github/workflows/upgrade-gate.yml
- uses: rustag/groundtruth-action@v1
  with:
    program-id: ${{ env.PROGRAM_ID }}
    candidate-binary: ./target/deploy/my_program.so
    rpc: ${{ secrets.MAINNET_RPC }}
    # Posts signed diff + alarm report to PR, fails on divergence
```

---

## How RustAG Differs from Surfpool

RustAG is **not** a local development environment competitor. It **interoperates** with
Surfpool (the Foundation-canonized integration-testing tool) and fills the layer Surfpool
explicitly does not occupy.

| | Surfpool | RustAG |
|---|---|---|
| **Tier** | Pre-deploy local dev loop | Pre-**sign** / pre-**deploy** assurance |
| **Design center** | Mutable state (26 cheatcodes) | Pinned, content-addressed, self-contained |
| **Output** | A running localnet | A signed, offline-re-executable **EvidenceBundle** |
| **Ingests multisig proposals?** | No | **Yes — the core unit of work** |
| **Attestation** | None | Ed25519 + SHA-256 Merkle, hash-chained |
| **Relationship** | — | Can use Surfpool as one execution backend |

---

## Workspace Layout

| Crate / package | What it is |
| --- | --- |
| `crates/rustag-core` | Sealed deterministic sandbox: LiteSVM + account state machine + fidelity (ProgramData dereference, Clock sync) + persistence |
| `crates/rustag-mirror` | Ingest pipeline: lazy mainnet fetch, TouchSetResolver, Squads v4 decoder, MultiRpcFetcher (N-of-M provenance), ForwardRecorder |
| `crates/rustag-rehearse` | The sealed two-pass rehearsal: `SealedRehearsal::run()` → `EvidenceBundle` with Grade-A re-executability proof |
| `crates/rustag-attest` | Evidence engine: `EvidenceBundle` v2, Ed25519 signing, Merkle-rooted state proofs, hash-chained audit log |
| `crates/rustag-replay` | Checkpoint, Journal, deterministic replay, diff_accounts, timeline reconstruction |
| `crates/rustag-sim` | Invariant policy engine, semantic diff (authority/nonce/treasury/config), exploit scanning, Executor trait |
| `crates/rustag-rpc` | Solana-compatible JSON-RPC + WebSocket + REST (axum) with `POST /api/rehearse` |
| `crates/rustag-cli` | The `rustag` binary: `rehearse`, `verify`, `forensics`, `record`, and legacy stagenet commands |
| `crates/rustag-cloud` | Multi-tenant hosted control plane + Evidence Registry (Phase 5) |
| `crates/rustag-scheduler` | Internal utility: recorder health, watched-proposal re-fetch |
| `crates/rustag-compression` | Off-chain concurrent Merkle tree for on-chain anchoring (Phase 5) |
| `packages/sdk` | `@rustag/sdk` — TypeScript client for the REST API |
| `packages/dashboard` | Next.js signer-review UI + Evidence Explorer |

---

## CLI Reference

| Command | Description |
| --- | --- |
| `rustag rehearse --proposal <PK>` | Rehearse a Squads multisig proposal → signed EvidenceBundle |
| `rustag rehearse --payload <B64>` | Rehearse a raw base64 transaction |
| `rustag verify <file.json>` | Verify an EvidenceBundle offline (re-execute + signature check) |
| `rustag forensics <signature>` | **Phase 4** — Re-execute a historical tx for incident response |
| `rustag record --program <PK>` | **Phase 3** — Start recording a program's mainnet transactions |
| `rustag create <name>` | Create a new stagenet (legacy local dev) |
| `rustag start [name]` | Start a stagenet's JSON-RPC + WS + REST servers |
| `rustag airdrop -s <name> <pk> <sol>` | Airdrop SOL to a wallet on a stagenet |
| `rustag scan [-s name]` | Scan recorded transactions for exploit signatures |

---

## Roadmap

### Phase 1 — Wedge: sealed pre-sign rehearsal ✅ (current)
- Sealed two-pass `SealedRehearsal` with Grade-A re-executability
- Squads v4 proposal decoder + TouchSetResolver
- N-of-M multi-RPC provenance
- Semantic diff with authority/nonce/treasury/config decoders
- Invariant policy engine with alarm system
- `rustag rehearse` and `rustag verify` CLI

### Phase 2 — Signer verifier + hosted API + Squads embed
- Web signer-review UI + one-click offline verifier
- Hosted `POST /rehearse` with API keys + per-flow metering
- Squads proposal URL ingestion + embedded "verified execution preview"
- secp256r1/secp256k1 precompiles for passkey/EVM-wallet proposals

### Phase 3 — Upgrade-rehearsal CI gate + ForwardRecorder
- Yellowstone gRPC-based per-program transaction corpus
- `Executor` trait + `PinnedSvmExecutor` + `SurfnetExecutor`
- GitHub Action step: signed diff + divergence report, fail on alarm
- Adversarial substitution fuzzing over real mainnet traffic

### Phase 4 — Forensics & counterfactual replay
- `rustag forensics <signature>` — historical tx re-execution
- Multi-tx incident timeline reconstruction
- Counterfactual replay: patch program, replay attack, attested verdict
- Published Drift and Loopscale deep reconstructions

### Phase 5 — Standard-setting, compliance, monetization
- Open `EvidenceBundle` spec (in-toto/SLSA-mapped)
- Hosted Evidence Registry GA with on-chain-anchored heads
- STRIDE listing + SIRN membership + Foundation RFP submissions
- Usage-based pricing: free OSS CLI + metered hosted rehearsals

---

## Development

```bash
just build      # cargo build --workspace
just test       # cargo test --workspace
just lint       # clippy -D warnings + fmt --check
just ci         # lint + test
```

Requires Rust 1.85+ (pinned in `rust-toolchain.toml`), Node 22+, and pnpm 10+.

---

*RustAG — because the best DeFi exploits are the ones you rehearse before anyone signs.*
*Open source. MIT OR Apache-2.0.*
