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
    "How a Solana client request flows through rustag-rpc → rustag-core → rustag-mirror, the Cargo workspace crate map, and the Phase 2 / Phase 3 feature set.",
};

const TOC: TocItem[] = [
  { id: "overview", title: "Architecture overview" },
  { id: "data-flow", title: "Request & data flow", depth: 3 },
  { id: "on-a-transaction", title: "What happens on a tx", depth: 3 },
  { id: "crates", title: "Crate map" },
  { id: "phases", title: "Phase 2 & 3" },
  { id: "phase-2", title: "Phase 2 features", depth: 3 },
  { id: "phase-3", title: "Phase 3 features", depth: 3 },
];

const DATA_FLOW = `  Solana client / wallet / Anchor / dashboard
          │  JSON-RPC      WebSocket      REST
          ▼                               ▼
┌───────────────────────────────────────────────────────┐
│ rustag-rpc  (axum)                                     │
│   • JSON-RPC server   POST  /            (jsonrpc.rs)  │
│   • WebSocket server  GET   /  accountSubscribe (ws.rs)│
│   • REST API          /api/*             (rest.rs)     │
│     mounts one rustag_core::Stagenet behind RwLock     │
└───────────────────────────────────────────────────────┘
          │ send_transaction / get_account_info / airdrop
          ▼
┌───────────────────────────────────────────────────────┐
│ rustag-core  (the engine)                              │
│   • LiteSVM instance (execution)                       │
│   • account-state machine: Unknown→Clean→Dirty/Pinned  │
│   • lazy-mirror logic (pre_load_accounts_for_tx)       │
│   • SQLite persistence via sqlx → survives restarts    │
│   • background workers: oracle sync, metrics, realtime │
└───────────────────────────────────────────────────────┘
          │ getMultipleAccounts (cache miss / oracle refresh)
          ▼
┌───────────────────────────────────────────────────────┐
│ rustag-mirror  (read-side, dependency-light)           │
│   • raw JSON-RPC over reqwest (no solana-rpc-client)   │
│   • MainnetMirror::fetch_multiple (≤100 keys/call)     │
│   • known-program / oracle registry + RpcRateLimiter   │
│   • RealtimeMirror push: accountSubscribe WS → mpsc    │
└───────────────────────────────────────────────────────┘
          │ HTTPS / WSS
          ▼
             Mainnet RPC (Helius / Triton / …)`;

type Tone = "lime" | "blue" | "amber" | "multi";

const CRATES: { crate: string; resp: string; phase: string; tone: Tone }[] = [
  {
    crate: "rustag-core",
    resp: "The runtime: LiteSVM + AccountSync state machine + SQLite persistence + lazy-mirror engine. Central type Stagenet.",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "rustag-mirror",
    resp: "Mainnet fetcher: raw JSON-RPC over reqwest, known-program/oracle registry, RpcRateLimiter, and the realtime push source.",
    phase: "1 · realtime 2",
    tone: "multi",
  },
  {
    crate: "rustag-rpc",
    resp: "Solana-compatible JSON-RPC + WebSocket + REST API, all on axum (serve, ServerAddrs, AppState).",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "rustag-cli",
    resp: "The rustag binary (clap subcommands for every Phase 1/2/3 command).",
    phase: "1 + 2/3",
    tone: "multi",
  },
  {
    crate: "rustag-scheduler",
    resp: "Activity Scheduler: pairs a Schedule (@every / cron) with an Action (airdrop / transfer / raw-tx).",
    phase: "2",
    tone: "blue",
  },
  {
    crate: "rustag-sim",
    resp: "Simulation: fork/replay/stress/compare, MEV/Jito bundles, invariant fuzzing, exploit scanning, differential execution.",
    phase: "2 / 3",
    tone: "multi",
  },
  {
    crate: "rustag-cloud",
    resp: "Multi-tenant control plane: each stagenet an isolated child process behind a reverse proxy with API-key auth.",
    phase: "2",
    tone: "blue",
  },
  {
    crate: "rustag-attest",
    resp: "Verifiable attestation: SHA-256 Merkle state_root, Ed25519-signed manifest, offline verify, hash-chained AuditLog.",
    phase: "3",
    tone: "amber",
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
    resp: "@rustag/sdk — TypeScript client for the REST API.",
    phase: "1",
    tone: "lime",
  },
  {
    crate: "packages/anchor-plugin",
    resp: "@rustag/anchor-plugin — ephemeral stagenet provider for Anchor tests against real mainnet state.",
    phase: "2",
    tone: "blue",
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
      lead="RustAG turns LiteSVM into a persistent, mainnet-mirroring staging environment. A request flows down through four layers — protocol, engine, mirror, and mainnet — with all policy living in the core."
      toc={TOC}
    >
      <H2 id="overview">Architecture overview</H2>
      <p>
        Instead of forking at a block hash (which Solana has no equivalent of), RustAG fetches mainnet
        accounts lazily on first access and tracks every local write so it knows what it may and may not
        refresh. A Solana client talks to a stagenet as if it were a cluster.
      </p>

      <H3 id="data-flow">Request &amp; data flow</H3>
      <CodeBlock lang="text" filename="four-layer request flow" code={DATA_FLOW} />
      <p>
        <code>rustag-mirror</code> is a pure read-side that knows nothing about dirty/clean tracking — it
        just answers &ldquo;give me the current mainnet state of these pubkeys.&rdquo; All the policy (state
        machine, persistence, sync invariants) lives in <code>rustag-core</code>, and all the protocol
        surface lives in <code>rustag-rpc</code>.
      </p>

      <H3 id="on-a-transaction">What happens on a transaction</H3>
      <p>
        <code>Stagenet::send_transaction</code> runs four stages:
      </p>
      <ol>
        <li>
          <strong>Pre-load</strong> — extract the transaction&apos;s static account keys and batch-fetch any
          that are not already loaded and not <code>Dirty</code> from mainnet via the mirror, loading them
          into LiteSVM as <code>Clean</code> (fetch failures are logged and tolerated).
        </li>
        <li>
          <strong>Execute</strong> through LiteSVM with signature and blockhash checks on.
        </li>
        <li>
          <strong>Track writes</strong> — derive writable accounts from the message header layout, mark them{" "}
          <code>Dirty</code>, and persist their post-state.
        </li>
        <li>
          <strong>Index</strong> the transaction (signature, success, fee, compute units, programs, logs)
          for the dashboard and <code>rustag logs</code>.
        </li>
      </ol>
      <p>
        For <code>VersionedMessage::V0</code> transactions, <code>prepare_accounts</code> resolves{" "}
        <code>address_table_lookups</code> through the mirror before execution, so v0 DeFi transactions read
        real mainnet state instead of failing with <code>LookupTableAccountNotFound</code>.
      </p>

      <H2 id="crates">Crate map</H2>
      <p>
        RustAG is a Cargo workspace under <code>crates/</code>. The dependency direction is{" "}
        <code>rustag-cli → rustag-rpc → rustag-core → rustag-mirror</code>; <code>rustag-core</code>{" "}
        re-exports the mirror surface so downstream crates have a single dependency. Every Phase 2/3 crate is
        pure Rust with no external service dependency.
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

      <H2 id="phases">Phase 2 &amp; 3</H2>
      <p>
        Everything beyond Phase 1 is built on a single invariant: <strong>a <code>Dirty</code> or{" "}
        <code>Pinned</code> account is never overwritten by any sync</strong> — re-enforced on every new
        path, including the realtime push path.
      </p>

      <H3 id="phase-2">
        Phase 2 features <PhaseBadge phase={2} className="ml-1" />
      </H3>
      <ul>
        <li>
          <strong>Real-time mirror (push)</strong> — a server-side <code>accountSubscribe</code> WebSocket
          (the Geyser/Yellowstone protocol) updates oracle prices sub-second, behind the{" "}
          <code>realtime</code> feature. A native Yellowstone gRPC source is a drop-in producer for the same{" "}
          <code>mpsc</code> channel.
        </li>
        <li>
          <strong>Activity Scheduler</strong> — recurring on-chain actions on <code>@every</code> /
          aliases / 5-field cron (Vixie semantics, no external cron crate); actions are airdrop, signed
          transfer, or raw-tx replay.
        </li>
        <li>
          <strong>Simulation framework</strong> — <code>fork</code> / <code>replay</code> /{" "}
          <code>stress</code> / <code>compare</code> against an isolated in-memory copy; the base is never
          mutated and mainnet is never touched. Reachable via <code>POST /api/simulate</code> and{" "}
          <code>client.simulate([...])</code>.
        </li>
        <li>
          <strong>Analytics</strong> — a background sampler captures TVL, transaction volume, accounts
          mirrored, dirty count, and slot as a queryable time-series.
        </li>
        <li>
          <strong>Cloud control plane</strong> (<code>rustag-cloud</code>) — multi-tenant orchestration;
          each stagenet is an isolated child process behind a reverse proxy with{" "}
          <code>Bearer rk_…</code> API-key auth and enforced cross-tenant isolation.
        </li>
        <li>
          <strong>GitHub Action &amp; Anchor plugin</strong> — an ephemeral per-PR stagenet, and{" "}
          <code>@rustag/anchor-plugin</code>&apos;s <code>rustagAnchorProvider({"{ preload }"})</code> /{" "}
          <code>EphemeralStagenet</code>.
        </li>
      </ul>
      <CodeBlock
        lang="rust"
        filename="stress a fork (rustag-sim) — the base is never mutated"
        code={`let mut fork = base.fork("herd").await?;
let report = rustag_sim::stress(&mut fork, "liquidations", 1_000, |i| build_tx(i)).await?;
println!("success rate: {:.1}%", report.success_rate() * 100.0);`}
      />

      <H3 id="phase-3">
        Phase 3 features <PhaseBadge phase={3} className="ml-1" />
      </H3>
      <p>
        Phase 3 is about trust and depth — making the <em>output</em> of staging something an auditor, grant
        committee, or CI gate can cryptographically rely on. Every artifact is byte-for-byte reproducible
        from public inputs.
      </p>
      <ul>
        <li>
          <strong>Verifiable attestation</strong> (<code>rustag-attest</code>) — a SHA-256 Merkle{" "}
          <code>state_root</code> over the pubkey-sorted account set, an Ed25519-signed manifest, and an
          offline <code>rustag verify</code>; plus a tamper-evident, hash-chained <code>AuditLog</code>.
        </li>
        <li>
          <strong>Time-travel &amp; replay</strong> (<code>rustag-replay</code>) — content-addressed{" "}
          <code>Checkpoint</code>s, deterministic <code>Journal</code> replay (<code>verify_deterministic</code>),{" "}
          <code>Timeline</code> diffs, and fork-of-fork <code>Lineage</code> with full ancestry.
        </li>
        <li>
          <strong>Adversarial simulation</strong> (<code>rustag-sim</code>) — atomic Jito-style bundles with
          tip accounting, deterministic invariant fuzzing (capturing the reproducing seed), a reproducible
          exploit-signature scanner, and a differential-execution harness.
        </li>
        <li>
          <strong>State / ZK compression testing</strong> (<code>rustag-compression</code>) — a keccak-256{" "}
          <code>ConcurrentMerkleTree</code> matching <code>spl-account-compression</code> so compressed-state
          programs and their proofs verify deterministically off-chain.
        </li>
      </ul>
      <CodeBlock
        lang="rust"
        filename="build a concurrent Merkle tree and verify a proof (rustag-compression)"
        code={`use rustag_compression::{ConcurrentMerkleTree, keccak256, verify_path};

let mut tree = ConcurrentMerkleTree::new(14, 64).unwrap();
let root = tree.append(keccak256(b"first compressed leaf")).unwrap();

let proof = tree.prove(0).unwrap();
assert!(verify_path(&root, &proof.leaf, proof.leaf_index, &proof.siblings));`}
      />

      <Callout variant="early" title="Honest boundary">
        Executing <em>arbitrary mainnet programs</em> end-to-end — e.g. a full Jupiter swap — needs the
        fuller program-loading planned for Phase 2+. Phase 1 loads program accounts verbatim (readable and
        present) but does not yet JIT-load their BPF bytecode from the program-data account. Your own
        deployed program reading real mainnet state works today. Real Firedancer execution in the
        differential harness is a documented <code>trait Backend</code> extension point.
      </Callout>
    </DocArticle>
  );
}
