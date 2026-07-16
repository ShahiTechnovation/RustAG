# RustAG Architecture: The GroundTruth Pre-Execution Assurance Layer

## Overview

RustAG is a **pre-execution assurance infrastructure** for privileged Solana
operations. It produces cryptographically signed, offline-verifiable evidence
that a proposed transaction does exactly what its human author intends —
*before any multisig signer approves it*.

This document describes the technical architecture that enables that guarantee.

---

## The Two-Layer Model

```
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 1: INGEST                                                 │
│  rustag-mirror                                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ TouchSet     │  │ SquadsDecoder│  │  MultiRpcFetcher     │   │
│  │ Resolver     │  │ (Squads v4)  │  │  (N-of-M provenance) │   │
│  │ ALT expand   │  │ proposal →   │  │  N endpoints agree?  │   │
│  │ ProgramData  │  │ VersionedTx  │  │  InputProvenance     │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  ForwardRecorder (Phase 3): real mainnet traffic corpus  │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
         │ Closure (content-addressed account snapshots)
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 2: SEALED REHEARSAL                                       │
│  rustag-core + rustag-rehearse + rustag-sim                      │
│                                                                  │
│  SealedRehearsal::run()                                          │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Pass 1 — DISCOVERY                                      │    │
│  │  Dry-run payload in offline LiteSVM → capture all        │    │
│  │  touched account keys (including CPI-touched accounts)   │    │
│  └──────────────────────────────────────────────────────────┘    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Pass 2 — EXECUTION                                      │    │
│  │  Re-execute in sealed LiteSVM (mirror disabled):         │    │
│  │  • pre_state_root = SHA256(closure)                      │    │
│  │  • run payload                                           │    │
│  │  • post_state_root = SHA256(post-state)                  │    │
│  │  • semantic diff (SemanticChange[])                      │    │
│  │  • invariant scan (Alarm[])                              │    │
│  └──────────────────────────────────────────────────────────┘    │
│         │                                                         │
│         ▼ Signs with Ed25519                                      │
│  EvidenceBundle { manifest, findings, signature }                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### rustag-mirror: Faithful State Ingest

The mirror is a **pure read-side** — it never writes to mainnet. Its
responsibility is producing a pinned, content-addressable snapshot of every
account a proposed payload will read or write.

**TouchSetResolver** computes the full account closure:
- Static keys from the message header
- Lookup-table-resolved keys (v0 transactions)
- ProgramData accounts (the real bytecode + upgrade authority)
- The Clock sysvar (for time-dependent logic)

**SquadsDecoder** fetches and Borsh-decodes a Squads v4 `VaultTransaction`
proposal account, extracting the raw `TransactionMessage` bytes and metadata
(multisig address, vault index, threshold, approval count).

**MultiRpcFetcher** cross-fetches the closure from N independent RPC endpoints
and requires M-of-N agreement before accepting an account's state. Records
`InputProvenance` for every fetched account (which endpoints agreed, at which
slot).

**ForwardRecorder** (Phase 3) polls `getSignaturesForAddress` to build a corpus
of real mainnet traffic for a watched program. Used as input to the
upgrade-rehearsal CI gate.

---

### rustag-core: The SVM Runtime

**Stagenet** wraps LiteSVM with:
- **Lazy mainnet mirroring** — accounts fetched from mainnet on first access
- **Dirty/clean tracking** — locally-modified accounts frozen from re-sync
- **Upgradeable program loading** — `load_upgradeable_program` follows the
  `Program → ProgramData` pointer and calls `LiteSVM::add_program` to register
  the real ELF for execution
- **Clock pinning** — `sync_clock(slot, unix_ts)` pins the sysvar for
  time-dependent rehearsal
- **Unlimited airdrops** — for test setup
- **SQLite persistence** — state survives process restarts

---

### rustag-rehearse: The Sealed Two-Pass Algorithm

`SealedRehearsal::run()` executes the two-pass algorithm that produces a
`Grade A` (deterministically re-executable) evidence bundle:

**Pass 1 — Discovery**: Runs the payload in an offline LiteSVM, ignoring
execution failure. Captures every account the payload touched (including
CPI-touched accounts that the static key set might not include).

**Pass 2 — Execution**: Creates a second offline LiteSVM loaded with exactly
the closure from Pass 1. Re-runs the payload. This run is sealed: no live RPC
calls, no non-determinism. Records:
- `pre_state_root` = SHA256 over all (pubkey, data, lamports, owner) tuples
- `post_state_root` = SHA256 over the post-execution state
- `success` and `compute_units`

The bundle is then signed with Ed25519 over all of the above. A verifier can
re-derive both roots independently and re-run the payload to confirm
`post_state_root`.

---

### rustag-sim: Semantic Diff & Invariant Policies

**SemanticDiff** decodes raw account state changes into human-readable claims:

| Variant | Decodes |
|---|---|
| `AccountCreated` | New account with owner and lamports |
| `NonceAccountCreated` | New durable-nonce account (replay-attack primitive) |
| `AccountClosed` | Account removed |
| `OwnerChanged` | Owning program changed |
| `UpgradeAuthority` | Program upgrade authority rotated |
| `ProgramFrozen` | Upgrade authority set to None (irrevocable) |
| `ProgramUpgraded` | ProgramData deployment slot changed (new bytecode) |
| `TokenAmount` | SPL token account balance changed |
| `TokenAuthorityChanged` | SPL token owner/close-authority changed |
| `SolBalance` | Pure SOL balance move |
| `DataChanged` | Data changed in a way not otherwise decoded |

**Standard Policy** runs 6 invariant rules on every rehearsal:

| Rule | What it catches |
|---|---|
| `upgrade-authority` | Any program upgrade authority rotation |
| `owner-change` | Any account owner change |
| `new-nonce-account` | Any new durable-nonce account creation |
| `program-freeze` | Upgrade authority set to None (irreversible) |
| `nonce-authority-combo` | Nonce creation + authority change in same payload (Drift attack pattern) |
| `large-sol-drain` | >80% SOL drain from any single account |

---

### rustag-rpc: REST API

Two forensics-specific endpoints:

- `POST /api/rehearse` — accepts a base64 payload (or Squads proposal key),
  runs a sealed rehearsal, returns signed `EvidenceBundle` + semantic diff +
  alarms. Supports `policy_rules` for custom invariants.
- `POST /api/verify` — accepts a bundle JSON, verifies the Ed25519 signature,
  returns fidelity grade and state root metadata.

---

## What Makes This Different from Surfpool

| Capability | Surfpool | RustAG |
|---|---|---|
| Primary use case | Local development | Pre-signing assurance |
| Output | Running local network | Signed EvidenceBundle |
| State model | Mutable (26 cheatcodes) | Pinned, content-addressed |
| Multisig proposals | Not supported | Core workflow (`--proposal`) |
| Semantic diff | No | Yes (11 variants) |
| Invariant policies | No | Yes (6 standard rules) |
| Cryptographic attestation | No | Ed25519 signed + SHA-256 |
| Independent verification | No | Yes (re-executable) |
| Forensics / counterfactual | No | Yes (`rustag forensics --patch`) |
| Upgrade CI gate | No | Yes (GitHub Action) |
| Real traffic corpus | No | Yes (`rustag record`) |

---

## Data Flow: Squads Proposal Rehearsal

```
rustag rehearse --proposal <SQUADS_PUBKEY> --rpc <MAINNET_RPC>
                │
                ▼
SquadsDecoder.decode_proposal()
  └─ RPC: getAccountInfo(<SQUADS_PUBKEY>)
  └─ Borsh-decode VaultTransaction
  └─ ProposedPayload { multisig, vault_index, threshold, message_bytes }
                │
                ▼
TouchSetResolver.resolve(<VersionedMessage>)
  └─ Static keys
  └─ v0 ALT expansion
  └─ ProgramData dereference
  └─ Clock sysvar
                │
                ▼
MultiRpcFetcher.fetch_with_provenance(<closure_keys>)
  └─ fetch from N endpoints
  └─ require M-of-N agreement
  └─ InputProvenance per account
                │
                ▼
SealedRehearsal::run()
  └─ Pass 1: Discovery (dry-run)
  └─ Pass 2: Execution (sealed)
  └─ SemanticDiff::decode()
  └─ Policy::standard().evaluate()
  └─ Sign with Ed25519
                │
                ▼
EvidenceBundle → groundtruth-bundle.json
Closure       → groundtruth-closure.json

# Signer verifies independently:
rustag verify groundtruth-bundle.json --closure groundtruth-closure.json
```

---

## The ForwardRecorder → Upgrade Gate Pipeline (Phase 3)

```
rustag record --program <PROGRAM_ID> --rpc <MAINNET_RPC> --out corpus.json
                │
                ▼  (getSignaturesForAddress + getTransaction)
RecordedCorpus { transactions: Vec<RecordedTransaction> }
                │
                ▼
GitHub Action: upgrade-rehearsal.yml
  ├─ Replay corpus against current bytecode → baseline alarms
  └─ rustag forensics --patch <candidate.so> → counterfactual result
      └─ BLOCKED if tx fails / REPRODUCED if tx still succeeds
      └─ Post diff report to PR
      └─ Fail CI on new alarms
```

---

## Fidelity Grades

| Grade | Meaning | When |
|---|---|---|
| **A** | Deterministically re-executable | Closure is complete, two-pass succeeded |
| **B** | Observed (not fully re-executable) | Clock-dependent or incomplete closure |

A Grade A bundle can be re-verified by anyone with the payload and closure,
independent of the original rehearser. A tampered proposer UI cannot produce a
valid Grade A bundle for a different payload.
