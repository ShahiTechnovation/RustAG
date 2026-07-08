# RustAG Phase 3 — Production-Readiness Checklist

This is the self-auditable gate for Phase 3, in the spirit of the Phase 1/2 checklists. Every
box is backed by code **and** a test in this repository, or is explicitly marked as a documented
extension point. Run `cargo test --workspace` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`
to confirm the mechanical claims yourself.

## P3.1 — Verifiable Staging Attestation (`rustag-attest`)

- [x] SHA-256 binary Merkle tree with domain-separated leaves/nodes (`merkle.rs`)
- [x] Inclusion proofs generate and verify for every leaf at many tree sizes (`merkle::tests`)
- [x] Tampered leaves and out-of-range indices are rejected (`merkle::tests`)
- [x] Canonical, deterministic account encoding — fixed-width LE fields, explicit data length,
      accounts sorted by pubkey so the root is order-independent (`state.rs`, `state::tests`)
- [x] State root commits to consensus-visible fields only (not internal dirty/clean bookkeeping)
- [x] Manifest signing digest is built from a fixed field order with length-prefixed fields, not
      JSON (no key-order ambiguity in what is signed) (`manifest.rs`, `manifest::tests`)
- [x] Ed25519 signing via `solana_keypair::Keypair`; **offline** verification via the
      `verify`-gated `solana_signature::Signature::verify` (`attestation.rs`)
- [x] Tampering any manifest field, swapping the attester, or changing the account set all fail
      verification (`attestation::tests`)
- [x] JSON artifact round-trips and still verifies (`attestation::tests`)
- [x] Tamper-evident, hash-chained audit log; edits/inserts/deletes are detected at the exact
      index (`audit.rs`, `audit::tests`) — SOC 2 groundwork
- [x] CLI `rustag attest` (generates/loads a solana-keygen-compatible attester key, writes
      `*.attestation.json`) and `rustag verify` (offline, exits non-zero on INVALID), smoke-tested
      including the tamper path

## P3.2 — Time-Travel, Deterministic Replay & Branching (`rustag-replay`)

- [x] Content-addressed `Checkpoint` capture + restore into an isolated offline stagenet;
      restore preserves the state root (`checkpoint.rs`, lib tests)
- [x] Transaction `Journal` stores transactions in canonical bincode+base64 wire form
- [x] `verify_deterministic` proves two independent replays of a journal from a checkpoint yield
      identical state roots (`timeline.rs`, lib tests)
- [x] `Timeline` account-level `StateDiff` (added/removed/changed) between any two checkpoints,
      computed via canonical leaf hashes (lib tests)
- [x] First-class fork-of-fork via `Lineage` + `branch_stagenet`; ancestry/children/depth tracked,
      and a branch-of-a-branch is fully isolated from its siblings (lib tests)

## P3.3 — MEV Bundles, Fuzzing, Exploit Scanning, Differential Execution (`rustag-sim`)

- [x] Atomic Jito-style bundle simulation on an isolated fork; a failing bundle does not land and
      never mutates the base (`bundle.rs`, `bundle::tests`)
- [x] `land_bundle` commits to a stagenet only when the bundle lands atomically (`bundle::tests`)
- [x] Tip accounting against the eight canonical Jito tip accounts, all validated as real pubkeys
      (`bundle::tests::all_default_tip_accounts_are_valid_pubkeys`)
- [x] Deterministic, seeded invariant fuzzing with reproducible violations; built-in
      `owner_unchanged` / `balance_floor` invariants (`fuzz.rs`, `fuzz::tests`)
- [x] Deterministic exploit-signature scanner: program panic, CPI depth at limit, re-entrant
      self-invocation, compute griefing, dangerous log signatures, high failure rate
      (`exploit.rs`, `exploit::tests`) — no model, no network, fully reproducible
- [x] Differential execution harness reports the first divergence in
      `(success, computeUnits, error, stateRoot)`; agrees for identical backends and detects
      divergence for differing ones (`diff.rs`, `diff::tests`)
- [x] CLI `rustag scan` with a `--fail-on <severity>` CI gate, smoke-tested

## P3.4 — State / ZK Compression Testing (`rustag-compression`)

- [x] Keccak-256 hashing matching `spl-account-compression`; empty-node table by self-hashing
      (`hash.rs`, `hash::tests`)
- [x] Sparse concurrent Merkle tree: append, authoritative update, prove, verify (`tree.rs`)
- [x] Root-history / changelog ring buffer; `replace_leaf` models on-chain acceptance of proofs
      against a **recent** root, fast-forwarded over intervening changes (`tree::tests`)
- [x] Oracle property test: a fast-forwarded stale proof equals the freshly generated proof
      (`tree::tests::fast_forward_matches_fresh_proof_property`)
- [x] Stale-but-untouched proofs are accepted; proofs for a leaf changed underneath are rejected;
      proofs older than the window are rejected (`tree::tests`)
- [x] Canopy layer utility that hashes up to the root (`tree::tests::canopy_layer_hashes_up_to_the_root`)
- [x] CLI `rustag tree` builds a tree and prints root + verified proofs, smoke-tested

## Cross-cutting quality gates

- [x] `cargo build --workspace` is clean
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes with zero
      exceptions
- [x] `cargo test --workspace` passes (Phase 3 adds 60+ tests; whole workspace green)
- [x] No `unsafe` blocks introduced by Phase 3
- [x] Every new workspace dependency (`sha3`, `hex`) carries a one-line justification in `Cargo.toml`
- [x] No secrets leak into artifacts — the attestation `mainnetSource` has its query string
      (`?api-key=...`) redacted before it is written (`commands/attest.rs` via `redact_url`)
- [x] Production code only — no AI-generated scratch/placeholder files; every file compiles, is
      documented, and is exercised by a test

## Adversarial review pass

A five-dimension adversarial review (each finding independently verified by a skeptic) was run
over the Phase 3 code. Seven confirmed findings were fixed and regression-tested:

- [x] `verify` no longer reports VALID when an **explicit** `--stagenet` cannot be resolved — it
      now hard-errors so the state-root check is never silently skipped (`commands/verify.rs`)
- [x] `replay_matches_journal` added: replay is now checked against the journal's recorded final
      root, not just against a second replay (`timeline.rs`, lib test)
- [x] `redact_url` now reduces an RPC endpoint to `scheme://host[:port]`, so a **path-embedded**
      API key (`/v2/<KEY>`) can no longer leak into a signed artifact (`commands/mod.rs` + tests)
- [x] `encode_tx_result` length-prefixes the optional error, so `None` and `Some("")` can no
      longer collide in the tx-results root (`state.rs` + test)
- [x] The exploit scanner's CPI parser ignores `Program log:`/`data:`/`return:` lines, so a
      `msg!("invoke [1]")` is no longer mistaken for a CPI frame (`exploit.rs` + test)
- [x] The attestation signing digest binds the full-precision `created_at` (nanoseconds), not
      whole seconds (`manifest.rs`)
- [x] Deterministic-replay scope documented: journals must be captured against an offline
      (mirror-disabled) stagenet to replay bit-for-bit (`journal.rs`)

## Documented extension points (honest boundaries, not faked)

- [ ] **Full Firedancer/Frankendancer execution backend** — the `differential` harness is
      complete and tested with the LiteSVM backend on both sides; a second backend pointed at a
      real Firedancer RPC is a `Stagenet`-shaped integration concern, not a code gap. The
      divergence-detection logic (the part that must be correct) ships and is tested today.
- [ ] **LLM-backed vulnerability detection** — the OSS path is the deterministic
      exploit-signature scanner (no key, no network). A hosted product can back the same
      `Finding` surface with an LLM; this repo intentionally ships no AI-dependent files.
- [ ] **On-chain attestation registry** — Phase 3 produces and verifies attestations offline. An
      optional on-chain registry program that anchors `state_root` + attester on Solana is a
      natural follow-on and is out of scope for this milestone.
