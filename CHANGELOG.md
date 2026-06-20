# Changelog

All notable changes to RustAG are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Phase 3 (trust & depth)

- **`rustag-attest`** - verifiable staging attestation: a SHA-256 Merkle commitment over a
  pubkey-sorted account set (`state_root`), a fixed-field-order signing digest, Ed25519 signing
  via `solana_keypair::Keypair`, and **offline** verification. Includes per-account inclusion
  proofs and a tamper-evident, hash-chained `AuditLog` (SOC 2 groundwork).
- **`rustag-replay`** - time-travel debugging and deterministic replay: content-addressed
  `Checkpoint`s, a canonical transaction `Journal`, `verify_deterministic`, account-level
  `Timeline` diffs, and first-class fork-of-fork `Lineage`.
- **`rustag-compression`** - an off-chain concurrent Merkle tree matching
  `spl-account-compression` (keccak-256, sparse storage, changelog + root-history fast-forward,
  canopy) for testing compressed-account/NFT programs.
- **`rustag-sim`** extensions - atomic Jito-style bundle simulation with tip accounting
  (`bundle`), deterministic seeded invariant fuzzing (`fuzz`), a reproducible exploit-signature
  scanner (`exploit`), and a differential-execution harness for client-diversity divergence
  (`diff`).
- **CLI** - `rustag attest`, `rustag verify`, `rustag scan` (with a `--fail-on <severity>` CI
  gate), and `rustag tree`.

### Security

- Attestation artifacts redact the mainnet RPC query string (`?api-key=...`) from
  `mainnetSource` before writing, so a shareable proof never leaks a key.

## Phase 2 - multi-tenant SaaS

- Real-time mirror over the `accountSubscribe` WebSocket (feature `realtime`).
- Activity Scheduler (`@every`/cron on-chain actions), simulation framework (fork / replay /
  stress / compare), analytics time-series, and the `rustag-cloud` multi-tenant control plane.

## Phase 1 - local CLI MVP

- LiteSVM runtime with the lazy mainnet account mirror, dirty/clean/pinned state machine,
  unlimited airdrops, state overrides, SQLite persistence, a Solana-compatible JSON-RPC +
  WebSocket + REST server, the `rustag` CLI, a TypeScript SDK, and a Next.js dashboard.

[Unreleased]: https://github.com/rustag/rustag/compare/v0.1.0...HEAD
