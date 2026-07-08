# RustAG — Phase 3 Master Technical Brief

## From Multi-Tenant SaaS to a Verifiable, Audit-Grade Staging Layer for Solana

> Phase 1 made a CLI that mirrors mainnet. Phase 2 turned it into hosted, multi-tenant
> infrastructure. **Phase 3 is about trust and depth: making the *output* of staging
> something an auditor, a grant committee, or a CI gate can cryptographically rely on —
> and making the staging engine itself deep enough to surface the bugs that only appear
> at the edges (atomic MEV bundles, compressed state, client-diversity divergence,
> adversarial transaction sequences).**

---

## 3.0 — WHO YOU ARE (AI Persona, Phase 3)

You are now operating as a **principal protocol-security engineer** layered on top of the
Phase 1 Solana expertise and the Phase 2 distributed-systems discipline. The Phase 3 mandate
changes the question you ask of every feature:

- Phase 1 asked *"does this work on my laptop?"*
- Phase 2 asked *"does this hold up as multi-tenant infrastructure?"*
- Phase 3 asks **"can someone who does not trust us *verify* what staging actually proved —
  and does staging go deep enough to catch the exploits that reach mainnet?"**

Operating principles that follow from that mandate:

1. **Determinism is a feature, not an accident.** Anything Phase 3 emits as evidence
   (a state root, an attestation, a replay) must be byte-for-byte reproducible by a third
   party from public inputs. No timestamps inside hashed payloads, no map-iteration order
   leaking into a Merkle root, no hidden RNG.
2. **Cryptographic over procedural.** Where Phase 2 said "our application code scopes the
   query," Phase 3 says "here is a signature and a Merkle proof — check it yourself." Trust
   is delegated to math, not to our codebase being bug-free.
3. **Adversarial by default.** Every new simulation primitive is built to *break* the
   program under test: atomic-bundle rollback, oracle-shock sequences, invariant fuzzing,
   exploit-signature scanning. The best DeFi bugs are the ones found in staging.
4. **No external trust required to verify.** A `.attestation.json` file verifies offline
   with nothing but the public key and the account set. No call home, no server, no SaaS
   dependency. This is what makes it a *moat* rather than a *feature*.
5. **Honest about boundaries.** Where a capability genuinely needs an external client
   (real Firedancer execution) or a hosted secret (an LLM key), Phase 3 ships the *harness
   and the extension point*, fully tested against a reference implementation, and says so
   plainly — it does not fake the dependency.

---

## 3.1 — PHASE 3 SCOPE (what this milestone delivers)

Phase 3 adds five new capabilities, all implemented as pure-Rust crates with no external
service dependency, each independently testable and wired into the CLI:

| # | Capability | Crate | The moat it builds |
|---|------------|-------|--------------------|
| **P3.1** | **Verifiable Staging Attestation** | `rustag-attest` | A signed, Merkle-rooted proof that a program was tested against an exact, mainnet-derived state set — verifiable offline by anyone. Plus a tamper-evident, hash-chained audit log (SOC 2 groundwork). |
| **P3.2** | **Time-Travel Debugging & Deterministic Replay** | `rustag-replay` | Checkpoint any stagenet, replay its transaction journal deterministically, step backward/forward, and branch (fork-of-fork) with full lineage — the basis of security-audit replays. |
| **P3.3** | **MEV / Jito Bundle Simulation, Invariant Fuzzing & Exploit Scanning** | `rustag-sim` (extended) | Atomic all-or-nothing bundle execution with tip accounting; property-based invariant fuzzing; a deterministic exploit-signature scanner; and a differential-execution harness for client-diversity divergence. |
| **P3.4** | **State / ZK Compression Testing** | `rustag-compression` | A concurrent Merkle tree matching `spl-account-compression` semantics (keccak-256, canopy, changelog, root-history) so teams can test compressed-account programs and verify proofs off-chain. |
| **P3.5** | **CLI + Docs surface** | `rustag-cli` (extended) | `rustag attest`, `rustag verify`, `rustag bundle`, `rustag fuzz`, `rustag scan`, `rustag tree` — every Phase 3 capability reachable from the command line. |

### Explicitly designed-with-extension-point, not faked

- **Full-client diversity (real Firedancer/Frankendancer execution).** Phase 3 ships the
  `DifferentialHarness` (P3.3) that runs an identical transaction sequence through two
  execution backends and reports the first divergence in `(success, compute_units, error,
  state_root)`. The reference backend is LiteSVM; a second backend pointed at a real
  Firedancer RPC is a documented `trait Backend` implementation. The *divergence-detection
  logic* is real and tested today; the second client binary is an integration concern.
- **LLM-assisted vulnerability detection.** Phase 3 ships a *deterministic* exploit-signature
  scanner (P3.3, `exploit.rs`) — a real rules engine over transaction logs/outcomes, no API
  key, no network. The hosted product can back the same `Finding` surface with an LLM; the
  local/OSS tool stays offline and reproducible. (This repo ships **no AI-dependent files**.)

---

## 3.2 — P3.1 VERIFIABLE STAGING ATTESTATION (the flagship)

**Problem.** A team runs their program against staged mainnet state and says "it's safe to
deploy." Today that claim is unverifiable: a reviewer has to trust the team's word about
*what state* it was tested against. Phase 3 makes the claim a cryptographic artifact.

**Design.**

```
accounts (sorted by pubkey)
   └── canonical leaf encoding  (domain || pubkey || owner || lamports || exec || rent || len || data)
        └── SHA-256 binary Merkle tree  ──►  state_root  (32 bytes)

transaction outcomes
   └── canonical result encoding
        └── SHA-256 Merkle tree  ──►  tx_results_root

AttestationManifest { schema, stagenet, tool_version, source, network, slot,
                      account_count, state_root, programs[], tx_count, tx_results_root, created_at }
   └── domain-separated, fixed-field-order SHA-256  ──►  signing_digest (32 bytes)
        └── Ed25519 sign (solana Keypair)  ──►  Attestation { manifest, attester, signature }
```

**Verification (offline, by anyone):**
1. Recompute `state_root` from the account set → must equal `manifest.state_root`.
2. Recompute `signing_digest` from the manifest fields → verify the Ed25519 signature
   against `attester`.
3. Optionally, a single account's membership is provable with a Merkle inclusion proof
   without revealing the rest of the state.

**Audit log (SOC 2 groundwork).** `AuditLog` is a hash-chained append-only log: each entry
commits to the previous entry's hash, so any insertion/deletion/edit anywhere in the history
is detectable by re-walking the chain. This is the primitive enterprise compliance asks for.

---

## 3.3 — P3.2 TIME-TRAVEL, DETERMINISTIC REPLAY & BRANCHING

- **Checkpoint**: a content-addressed snapshot `{ slot, accounts, state_root }` of a stagenet.
- **Journal**: the ordered list of transactions applied since genesis (or since a checkpoint).
- **Timeline**: checkpoints + journal. `restore_to(checkpoint)` rebuilds an isolated stagenet
  at that point; `replay(checkpoint, txs)` re-applies and returns the resulting `state_root`.
- **Determinism check**: replaying the same journal from the same checkpoint twice must yield
  identical `state_root`s — `verify_deterministic` asserts it and is the basis of audit replay.
- **Lineage (fork-of-fork)**: every fork records its parent id and the slot it branched at,
  forming a `Lineage` tree. Branches are first-class: you can branch a branch and the tree
  records the full ancestry, so "what staged state produced this bug?" is always answerable.

## 3.4 — P3.3 MEV BUNDLES, FUZZING, EXPLOIT SCANNING, DIFFERENTIAL EXECUTION

- **Atomic bundles** (`bundle.rs`): execute a list of transactions all-or-nothing on an
  isolated fork. If any transaction fails, the bundle is reported reverted and the base state
  is never mutated — matching Jito bundle semantics. Tip accounting detects transfers to the
  known Jito tip accounts.
- **Invariant fuzzing** (`fuzz.rs`): a deterministic (seeded) fuzz loop that applies generated
  transactions and, after each, checks a set of user-supplied `Invariant`s (e.g. "total
  lamports are conserved", "this account's owner never changes"). Any violation is captured
  with the seed needed to reproduce it.
- **Exploit-signature scanner** (`exploit.rs`): a deterministic rules engine over transaction
  outcomes/logs that flags known DeFi attack shapes (oracle deviation, lamport drain, failed
  CPI depth, suspicious error codes) as `Finding`s with severities.
- **Differential harness** (`diff.rs`): runs one transaction sequence through two backends and
  reports the first observable divergence — the client-diversity safety net.

## 3.5 — P3.4 STATE / ZK COMPRESSION TESTING

`rustag-compression` provides a `ConcurrentMerkleTree` matching the semantics Solana's
`spl-account-compression` program uses on-chain:

- **keccak-256** leaf/node hashing (so off-chain roots match on-chain roots),
- a **changelog** ring buffer enabling proofs against recent historical roots,
- a **canopy** (cached top layers) so proofs can be shorter,
- `append`, `replace_leaf` (with proof), `prove`, and `verify`.

This lets teams that build compressed-NFT / compressed-account programs validate tree
behavior and proof generation deterministically, off-chain, in their test suite.

---

## 3.6 — PRODUCTION-READINESS CHECKLIST

See [`docs/phase3-checklist.md`](./phase3-checklist.md) for the full, self-auditable
checklist. Every box is backed by code and a test, or explicitly marked as a documented
extension point.
