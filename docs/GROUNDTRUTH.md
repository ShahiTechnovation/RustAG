# GroundTruth: The Product Story

> **RustAG is now GroundTruth** — the pre-execution assurance layer that the
> Solana ecosystem is missing.

---

## The $285 Million Question

On April 2026, Drift Protocol — one of Solana's most audited DeFi protocols —
lost $285 million. Not to a smart contract exploit. Not to an unpatched
vulnerability. To **blind signing**.

Attackers used durable nonces and social engineering to get multisig council
members to approve transactions that "appeared routine but carried hidden
authority rotations." By the time anyone noticed, the program's upgrade authority
had been silently transferred.

CoinDesk called this failure *"the gap between onchain correctness and offchain
human trust — a gap no smart-contract audit or monitoring tool is built to
cover."*

The Solana Foundation's own STRIDE/SIRN monitoring framework admitted it *"would
not have caught"* the attack — because every transaction was valid by design.

---

## The Gap

Today, when a Squads multisig signer receives a proposal to approve:

- They see **SOL and token balance diffs at best**
- Or they follow SEAL's official guidance to **hand-decode little-endian hex**
- They cannot see authority rotations, config mutations, or nonce creations
- They cannot re-verify what they approved after the fact
- They cannot prove to others what the transaction was supposed to do

On EVM, **Tenderly + Safe** make pre-execution simulation a funded category with
real revenue. On Solana, this capability is defended by a CLI.

---

## What RustAG Does

RustAG fills this gap with **attested pre-execution assurance**:

### 1. Sealed Rehearsal
Given any proposed privileged payload — a Squads multisig proposal, a program
upgrade, a treasury move — RustAG executes it in a deterministic, sealed sandbox
against **real mainnet state** and produces:

- **StateDiff**: every account that changed, and how
- **SemanticDiff**: human-readable claims ("upgrade authority rotated from X to Y",
  "1M USDC moved from treasury", "durable nonce account created")
- **Invariant alarms**: security-critical changes flagged automatically
- **Fidelity grade**: Grade A = deterministically re-executable by anyone

### 2. Signed Evidence Bundle
The result is cryptographically signed with Ed25519 and hash-chained:

- `payload_hash` — proves which transaction was rehearsed
- `pre_state_root` — content-addressed snapshot of input state
- `post_state_root` — content-addressed snapshot of output state
- `findings` — the semantic diff, alarms, and exploit scan
- `signature` — the rehearser's Ed25519 signature over all of the above

### 3. Independent Verification
Any signer can re-verify the bundle **offline**:

- Re-fetch the closure from their **own** RPC (not the proposer's)
- Compare `pre_state_root` with the bundle's claim
- Re-execute the payload and compare `post_state_root`
- If everything matches → the bundle is Grade A trustworthy

A compromised proposer UI cannot make a bundle pass for a different payload.

---

## How We're Different from Surfpool

Surfpool is the Solana Foundation's canonized **local development tool**. We do
not compete with it — we occupy the adjacent layer it explicitly does not:

| | Surfpool | RustAG |
|---|---|---|
| When | During development | Before signing / deploying |
| What | A running local network | A signed, verifiable evidence bundle |
| State | Mutable (26 cheatcodes) | Pinned, content-addressed |
| Multisig proposals | No | **Core unit of work** |
| Attestation | None | Ed25519 + SHA-256 Merkle |
| Relationship | — | Can use Surfpool as execution backend |

RustAG **interoperates** with Surfpool — we can drive a surfnet as one execution
backend behind our `Executor` trait, turning the Foundation-backed incumbent into
a distribution channel rather than a competitor.

---

## The Five-Phase Roadmap

### Phase 1 — The Wedge (Current)
Sealed pre-sign rehearsal: `rustag rehearse --proposal <SQUADS_PUBKEY>` →
signed EvidenceBundle. Squads v4 decoder, TouchSetResolver, N-of-M provenance,
semantic diff with authority/nonce/treasury/config decoders, 6-rule standard
policy.

### Phase 2 — Signer Verifier + Squads Embed
Web signer-review UI, hosted `POST /rehearse` API with metering, Squads deep
integration as "verified execution preview", secp256r1/secp256k1 precompiles.

### Phase 3 — Upgrade-Rehearsal CI Gate
Record a program's mainnet traffic → replay candidate upgrades against it →
GitHub Action that posts a signed diff report and fails on divergence.

### Phase 4 — Forensics & Counterfactual Replay
`rustag forensics <signature>` — reconstruct incidents, replay with patched
programs, signed verdict.

### Phase 5 — Standard-Setting & Monetization
Open `EvidenceBundle` spec (in-toto/SLSA-mapped), hosted Evidence Registry,
STRIDE listing, per-flow pricing.

---

## Why This Is Fundable

1. **Chokepoint pricing.** Per screened flow at the multisig/upgrade chokepoint,
   not per seat. Crypto teams accept small fees on high-value flows.

2. **Post-traumatic budget.** The buyer is a post-Drift council with real security
   budget and a changed signing process — not the zero-WTP dev-tool user.

3. **Foundation alignment.** Three standing Solana Foundation RFPs map directly
   onto this: Pre-Deployment Program Analysis, Program Verification Tooling,
   Post-Deployment Monitoring.

4. **Expansion ladder.** Pre-sign rehearsal → upgrade CI gate → SIRN forensics
   retainers → institutional compliance evidence.

5. **EVM precedent.** Tenderly proved this is a category with real revenue and
   has **no SVM presence**.
