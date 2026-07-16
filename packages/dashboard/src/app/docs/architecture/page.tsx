import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";
import { PhaseBadge } from "@/components/docs/PhaseBadge";
import { cn } from "@/lib/cn";

export const metadata: Metadata = {
  title: "Architecture",
  description:
    "RustAG dual-layer design: Ingest layer (TouchSetResolver, SquadsDecoder, MultiRpcFetcher) feeding a Sealed Rehearsal layer (LiteSVM, semantic diff, invariant policy, EvidenceBundle signing).",
};

const TOC: TocItem[] = [
  { id: "overview", title: "Architecture overview" },
  { id: "ingest-layer", title: "Ingest layer", depth: 3 },
  { id: "rehearsal-layer", title: "Sealed rehearsal layer", depth: 3 },
  { id: "data-flow", title: "End-to-end data flow", depth: 3 },
  { id: "crates", title: "Crate map" },
  { id: "phases", title: "Phase 2 & 3" },
  { id: "phase-2", title: "Phase 2 features", depth: 3 },
  { id: "phase-3", title: "Phase 3 features", depth: 3 },
];

const DATA_FLOW = `  Wallet / Squads UI / Multisig signer / CI pipeline
           │  POST /api/rehearse { proposal | payload }
           ▼
┌─────────────────────────────────────────────────────────┐
│ INGEST LAYER                                             │
│   SquadsDecoder  — Borsh-decode VaultTransaction        │
│   TouchSetResolver — walk instruction accounts          │
│   MultiRpcFetcher  — getMultipleAccounts (≤100/call)    │
│   ForwardRecorder  — record traffic corpus              │
└─────────────────────────────────────────────────────────┘
           │ sealed pre-state closure (pubkey → AccountData)
           ▼
┌─────────────────────────────────────────────────────────┐
│ SEALED REHEARSAL (rustag-rehearse)                       │
│   Pass 1 (Pre-state)                                     │
│     • load closure into isolated LiteSVM instance        │
│     • content-hash every account → pre_state_root        │
│   Pass 2 (Execution)                                     │
│     • execute payload → capture post-state               │
│     • SemanticDiff  — 11 change types                    │
│     • InvariantPolicy — 6 alarm rules                    │
│     • FidelityGrade — Grade A / Grade B                  │
│   Signing                                                │
│     • Ed25519 sign over pre+post root + diff + alarms    │
│     → EvidenceBundle.json + closure.json                 │
└─────────────────────────────────────────────────────────┘
           │  signed EvidenceBundle
           ▼
  Signer review / offline verify / CI gate / registry`;

type Tone = "lime" | "blue" | "amber" | "multi";

const CRATES: { crate: string; resp: string; phase: string; tone: Tone }[] = [
  {
    crate: "rustag-rehearse",
    resp: "Sealed two-pass rehearsal engine: PortableBundle, EvidenceBundle, FidelityGrade (A/B). The core GroundTruth primitive.",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "rustag-mirror",
    resp: "Ingest layer: TouchSetResolver, SquadsDecoder, MultiRpcFetcher (≤100 keys/call, no solana-rpc-client), ForwardRecorder corpus builder.",
    phase: "1 · realtime 2",
    tone: "multi",
  },
  {
    crate: "rustag-sim",
    resp: "SemanticDiff (11 change types), InvariantPolicy (6 alarm rules), fuzzing, exploit scanning, differential execution.",
    phase: "1 / 2",
    tone: "multi",
  },
  {
    crate: "rustag-attest",
    resp: "Ed25519 signing, Merkle state_root, offline verify, EvidenceBundle wrapper, hash-chained AuditLog.",
    phase: "1 / 3",
    tone: "multi",
  },
  {
    crate: "rustag-core",
    resp: "Persistent SVM stagenet runtime: LiteSVM + AccountSync state machine (Unknown→Clean→Dirty→Pinned) + SQLite via sqlx.",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "rustag-rpc",
    resp: "axum server: POST /api/rehearse, POST /api/verify, Solana-compatible JSON-RPC, WebSocket, REST API.",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "rustag-cli",
    resp: "The rustag binary: rehearse, verify, forensics, record, serve, and full stagenet management surface.",
    phase: "1 + 2/3",
    tone: "multi",
  },
  {
    crate: "rustag-scheduler",
    resp: "Activity Scheduler: @every / cron actions (airdrop / transfer / raw-tx) for the stagenet dev-tool surface.",
    phase: "2",
    tone: "blue",
  },
  {
    crate: "rustag-cloud",
    resp: "Multi-tenant control plane: isolated child processes behind a reverse proxy with Bearer rk_… API-key auth.",
    phase: "2",
    tone: "blue",
  },
  {
    crate: "rustag-replay",
    resp: "Time-travel: content-addressed Checkpoint, deterministic Journal replay, Timeline diffs, fork-of-fork Lineage.",
    phase: "3",
    tone: "amber",
  },
  {
    crate: "rustag-compression",
    resp: "Off-chain spl-account-compression-compatible ConcurrentMerkleTree (keccak-256, changelog, root-history, canopy).",
    phase: "3",
    tone: "amber",
  },
  {
    crate: "packages/sdk",
    resp: "@rustag/sdk — TypeScript client for POST /api/rehearse, POST /api/verify, and the full REST surface.",
    phase: "1",
    tone: "lime",
  },
];

const TONE_CLS: Record<Tone, string> = {
  lime: "border-state-clean/35 bg-state-clean/12 text-state-clean",
  blue: "border-state-pinned/35 bg-state-pinned/12 text-state-pinned",
  amber: "border-state-dirty/35 bg-state-dirty/12 text-state-dirty",
  multi: "border-border-strong bg-white/[0.03] text-muted",
};

export default function ArchitecturePage() {
  return (
    <DocArticle
      eyebrow="Advanced"
      title="Architecture"
      lead="RustAG is a dual-layer system. The Ingest layer resolves every account a proposed transaction will touch. The Sealed Rehearsal layer executes it in isolation, diffs the state, fires invariant alarms, and signs the result as a cryptographic EvidenceBundle."
      toc={TOC}
    >
      <H2 id="overview">Architecture overview</H2>
      <p>
        The core design principle: the rehearser must be <strong>independently verifiable</strong>.
        That means the input (pre-state closure) must be content-addressable from public mainnet
        data, and the output (EvidenceBundle) must be byte-for-byte reproducible by anyone who
        runs the same closure through the same payload.
      </p>
      <p>
        RustAG achieves this by splitting responsibility into two layers that share no mutable
        state with each other.
      </p>

      <H3 id="ingest-layer">Ingest layer</H3>
      <p>
        The ingest layer&apos;s job is to resolve the exact set of accounts the proposed payload
        will read or write — called the <strong>touch set</strong>. It never executes anything;
        it only reads mainnet.
      </p>
      <ul>
        <li>
          <strong>SquadsDecoder</strong> — Borsh-decodes a Squads v4{" "}
          <code>VaultTransaction</code> from its on-chain proposal address.
        </li>
        <li>
          <strong>TouchSetResolver</strong> — static analysis walk of instruction account metas
          to produce the minimal pubkey set.
        </li>
        <li>
          <strong>MultiRpcFetcher</strong> — batches <code>getMultipleAccounts</code> calls
          (≤100 keys per call) over raw reqwest without <code>solana-rpc-client</code> — this
          keeps LiteSVM 0.12 dep compatibility.
        </li>
        <li>
          <strong>ForwardRecorder</strong> — optional corpus recorder that serializes the
          resolved closure to disk for CI replay.
        </li>
      </ul>

      <H3 id="rehearsal-layer">Sealed rehearsal layer</H3>
      <p>
        The rehearsal layer receives the sealed closure (pubkey → AccountData snapshot at a
        known slot). It runs a deterministic two-pass process and produces a signed artifact:
      </p>
      <ol>
        <li>
          <strong>Pass 1 — Pre-state root</strong>: load the closure into an isolated LiteSVM
          instance, SHA-256 hash every account in pubkey order →{" "}
          <code>pre_state_root</code>.
        </li>
        <li>
          <strong>Pass 2 — Execute + diff</strong>: execute the payload, capture post-state,
          run <code>SemanticDiff</code> (11 change types) and{" "}
          <code>InvariantPolicy</code> (6 alarm rules), derive <code>post_state_root</code>,
          assign <code>FidelityGrade</code>.
        </li>
        <li>
          <strong>Signing</strong>: Ed25519-sign the concatenation of pre_state_root +
          post_state_root + semantic_diff + alarms + grade → write{" "}
          <code>EvidenceBundle.json</code> and portable <code>closure.json</code>.
        </li>
      </ol>
      <Callout variant="info" title="Grade A vs Grade B">
        Grade A means the closure is complete — every account was resolved and the rehearsal is
        deterministically re-executable offline. Grade B means one or more accounts could not be
        fetched (rate-limited, new account, etc.); the bundle is still signed but the
        post_state_root cannot be reproduced without re-fetching. Treat Grade B as advisory only
        and verify with a fresh RPC key.
      </Callout>

      <H3 id="data-flow">End-to-end data flow</H3>
      <CodeBlock lang="text" filename="GroundTruth two-layer data flow" code={DATA_FLOW} />

      <H2 id="crates">Crate map</H2>
      <p>
        RustAG is a Cargo workspace under <code>crates/</code>. The core dependency direction is{" "}
        <code>
          rustag-cli → rustag-rpc → rustag-rehearse → rustag-sim + rustag-attest →
          rustag-mirror → rustag-core
        </code>
        . Every Phase 2/3 crate is pure Rust with no external service dependency.
      </p>

      <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border bg-white/[0.015]">
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                Crate
              </th>
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                Responsibility
              </th>
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                Phase
              </th>
            </tr>
          </thead>
          <tbody>
            {CRATES.map((c) => (
              <tr key={c.crate} className="border-b border-border/60 align-top last:border-0">
                <td className="whitespace-nowrap px-4 py-3 font-mono text-[12px] text-brand">{c.crate}</td>
                <td className="px-4 py-3 text-[13px] leading-relaxed text-muted">{c.resp}</td>
                <td className="px-4 py-3">
                  <span
                    className={cn(
                      "inline-block whitespace-nowrap rounded-[3px] border px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em]",
                      TONE_CLS[c.tone],
                    )}
                  >
                    {c.phase}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H2 id="phases">Phase 2 & 3</H2>
      <p>
        The invariant across all phases:{" "}
        <strong>
          the pre-state closure is always sealed before execution and never mutated after.
        </strong>{" "}
        Every Phase 2/3 extension is additive — it does not change how Phase 1 bundles are
        produced or verified.
      </p>

      <H3 id="phase-2">
        Phase 2 features <PhaseBadge phase={2} className="ml-1" />
      </H3>
      <ul>
        <li>
          <strong>Yellowstone gRPC recording</strong> — real-time traffic corpus from a Geyser
          stream; replaces <code>ForwardRecorder</code>&apos;s poll-based approach with a push
          source for sub-second latency corpus building.
        </li>
        <li>
          <strong>Evidence Registry</strong> — hosted, append-only store for signed bundles with
          N-of-M signer provenance. A Squads vault can require M-of-N reviewers to submit a valid
          Grade A bundle before a proposal can be approved.
        </li>
        <li>
          <strong>Squads web UI embed</strong> — a signer-review panel that fetches and renders
          the EvidenceBundle inline in the Squads proposal UI, without requiring any CLI.
        </li>
        <li>
          <strong>Activity Scheduler</strong> — recurring on-chain actions for the stagenet dev
          surface (<code>@every / cron</code>).
        </li>
        <li>
          <strong>Real-time mirror push</strong> — <code>accountSubscribe</code> WebSocket /
          Yellowstone gRPC → oracle prices under 2s staleness.
        </li>
        <li>
          <strong>Cloud control plane</strong> (<code>rustag-cloud</code>) — multi-tenant hosted
          rehearsal service with <code>Bearer rk_…</code> API-key auth.
        </li>
      </ul>
      <CodeBlock
        lang="rust"
        filename="semantic diff — 11 change types (rustag-sim)"
        code={`// A sample of the SemanticChange variants produced by SemanticDiff
SemanticChange::LamportsDrained    { from, to, delta }
SemanticChange::UpgradeAuthority   { from, to }       // CRITICAL alarm
SemanticChange::ProgramUpgraded    { pubkey, old_hash, new_hash }
SemanticChange::TokenAuthorityChanged { mint, from, to }
SemanticChange::AccountClosed      { pubkey, recovered_lamports }
SemanticChange::DataWritten        { pubkey, len }
// + 5 more: Created, Frozen, Thawed, NonceDerived, SysvarMutated`}
      />

      <H3 id="phase-3">
        Phase 3 features <PhaseBadge phase={3} className="ml-1" />
      </H3>
      <ul>
        <li>
          <strong>Per-flow pricing & quota</strong> — usage-metered rehearsal API with
          tiered plans (free / pro / enterprise).
        </li>
        <li>
          <strong>Time-travel & replay</strong> (<code>rustag-replay</code>) —
          content-addressed <code>Checkpoint</code>s, deterministic <code>Journal</code> replay,{" "}
          <code>Timeline</code> diffs, and fork-of-fork <code>Lineage</code>.
        </li>
        <li>
          <strong>Adversarial simulation</strong> (<code>rustag-sim</code>) — atomic Jito-style
          bundles with tip accounting, deterministic invariant fuzzing, and a reproducible
          exploit-signature scanner.
        </li>
        <li>
          <strong>State / ZK compression testing</strong> (<code>rustag-compression</code>) — a
          keccak-256 <code>ConcurrentMerkleTree</code> matching{" "}
          <code>spl-account-compression</code> so compressed-state programs verify
          deterministically off-chain.
        </li>
      </ul>

      <Callout variant="early" title="Honest boundary">
        Phase 1 delivers: rehearse, verify, forensics, serve, and the signed EvidenceBundle end-to-end.
        The Evidence Registry, Squads UI embed, Yellowstone gRPC, and hosted multi-tenant service are
        Phase 2 and are not yet released. Everything documented on this page as Phase 1 works today —
        build from source and run locally.
      </Callout>
    </DocArticle>
  );
}
