import type { Metadata } from "next";
import { Check, Minus } from "lucide-react";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";
import { PhaseBadge } from "@/components/docs/PhaseBadge";
import { StatePill } from "@/components/ui";

export const metadata: Metadata = {
  title: "Core concepts",
  description:
    "The lazy account mirror and the Unknown → Clean → Dirty → Pinned account state machine — the single idea the whole of RustAG is built around.",
};

const TOC: TocItem[] = [
  { id: "lazy-mirror", title: "The lazy account mirror" },
  { id: "how-it-works", title: "How it works", depth: 3 },
  { id: "svm-replay", title: "Why this matters on the SVM", depth: 3 },
  { id: "state-machine", title: "Account state machine" },
  { id: "transitions", title: "Transitions", depth: 3 },
  { id: "oracles", title: "Oracle freshness" },
  { id: "why-staging", title: "Why staging, not testnet" },
];

const STATE_ROWS = [
  { state: "Unknown" as const, meaning: "Never fetched; pulled lazily on first access.", sync: false },
  { state: "Clean" as const, meaning: "A faithful mainnet copy.", sync: true },
  { state: "Dirty" as const, meaning: "Modified by a local transaction.", sync: false },
  { state: "Pinned" as const, meaning: "Set via the override API.", sync: false },
];

const MATRIX = [
  { tool: "solana-test-validator", cells: [false, true, false, false, false] },
  { tool: "LiteSVM / Bankrun (libs)", cells: [false, false, false, true, false] },
  { tool: "Devnet / Testnet", cells: [false, true, false, false, false] },
  { tool: "RustAG", cells: [true, true, true, true, true], highlight: true },
];
const MATRIX_COLS = [
  "Real mainnet state",
  "Persistent + RPC",
  "Real-time oracles",
  "Unlimited airdrop",
  "Cloud / multi-tenant",
];

export default function ConceptsPage() {
  return (
    <DocArticle
      eyebrow="Core concepts"
      title="The lazy mirror"
      lead="RustAG replays mainnet on a local SVM with no block to fork from. It does that by fetching the exact accounts a transaction touches, on first access, and tracking every write through a four-state machine."
      toc={TOC}
    >
      <H2 id="lazy-mirror">The lazy account mirror</H2>
      <p>
        The lazy account mirror is the core idea of RustAG. Rather than copying all of mainnet up front, it
        fetches the exact accounts a transaction touches — on first access — and caches them locally.
      </p>

      <H3 id="how-it-works">How it works</H3>
      <p>When a transaction reads account X:</p>
      <ol>
        <li>
          <strong>Local hit?</strong> Return the stagenet&apos;s local copy.
        </li>
        <li>
          <strong>Miss?</strong> Fetch it from mainnet → cache it → mark it <code>Clean</code> → return it.
        </li>
        <li>
          <strong>A transaction writes X?</strong> Mark it <code>Dirty</code> — it is now frozen from
          mainnet sync forever, so your local changes are never clobbered.
        </li>
      </ol>
      <CodeBlock
        lang="text"
        filename="the lazy-mirror decision flow"
        code={`Local hit?  → return local copy
Miss?       → fetch from mainnet → cache → mark Clean → return
Write to X? → mark Dirty (frozen from mainnet sync forever)

Background: re-fetch Clean ORACLE accounts every 30s
            Dirty + Pinned accounts: never overwritten`}
      />
      <p>
        A background task re-fetches <code>Clean</code> <strong>oracle</strong> accounts every 30 seconds
        (the default interval), so Pyth/Switchboard prices stay fresh. In the transaction path, a pre-load
        step batch-fetches any static account key that is not already loaded and not <code>Dirty</code>,
        loading it into LiteSVM as <code>Clean</code>; fetch failures are logged and tolerated.
      </p>

      <H3 id="svm-replay">Why this matters on the SVM</H3>
      <p>
        This is how &ldquo;mainnet replay&rdquo; works on Solana. EVM tools (Tenderly, Anvil&apos;s{" "}
        <code>--fork-url</code>) fork at a block hash and pull state from that fixed point; the SVM has no
        equivalent block to fork from. So RustAG instead fetches accounts on demand and tracks every write,
        so it always knows what it may and may not refresh from mainnet.
      </p>
      <p>
        The mirror itself (<code>rustag-mirror</code>) is a deliberately dependency-light read side: given
        pubkeys, it returns current mainnet state via a raw <code>getMultipleAccounts</code> JSON-RPC call
        over <code>reqwest</code> (≤100 keys per call), avoiding <code>solana-rpc-client</code> so it
        doesn&apos;t fork the Solana crate versions LiteSVM 0.12 unifies on.
      </p>
      <Callout variant="early" title="Known limitation (early access)">
        The mirror loads program accounts <em>verbatim</em>, so they are readable and present, but Phase 1
        does not yet extract and JIT-load BPF bytecode from the separate program-data account. Your own
        deployed program can read real mainnet state today; invoking a foreign program like a full Jupiter
        swap end-to-end needs the fuller program-loading planned for Phase 2.
      </Callout>

      <H2 id="state-machine">Account state machine</H2>
      <p>
        Every account in a stagenet carries one of four sync states — the <code>AccountSync</code> enum in{" "}
        <code>crates/rustag-core/src/account_state.rs</code>. The state decides whether the background
        scheduler is allowed to overwrite the account from mainnet.
      </p>

      <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border bg-white/[0.015]">
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
                State
              </th>
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
                Meaning
              </th>
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.14em] text-faint">
                Background sync?
              </th>
            </tr>
          </thead>
          <tbody>
            {STATE_ROWS.map((r) => (
              <tr key={r.state} className="border-b border-border/60 align-middle last:border-0">
                <td className="px-4 py-3">
                  <StatePill state={r.state} />
                </td>
                <td className="px-4 py-3 text-muted">{r.meaning}</td>
                <td className="px-4 py-3">
                  {r.sync ? (
                    <span className="font-mono text-xs text-state-clean">Yes</span>
                  ) : (
                    <span className="font-mono text-xs text-faint">Never</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <p>
        <code>Clean</code> carries a <code>fetched_at</code> timestamp and <code>Dirty</code> carries a{" "}
        <code>modified_at</code> timestamp; <code>Unknown</code> and <code>Pinned</code> are plain variants.
        An account <code>is_syncable()</code> only when it is <code>Clean</code> or <code>Unknown</code> —
        exactly the set the background oracle loop is allowed to refresh.
      </p>
      <CodeBlock
        lang="rust"
        filename="crates/rustag-core/src/account_state.rs"
        code={`pub enum AccountSync {
    /// Never fetched. Will be fetched lazily on first access.
    Unknown,
    /// Fetched from mainnet. May be re-synced by the background scheduler.
    Clean { fetched_at: DateTime<Utc> },
    /// Modified by a local transaction. Never overwritten by mainnet sync.
    Dirty { modified_at: DateTime<Utc> },
    /// Explicitly set by the user via the override API. Immune to everything.
    Pinned,
}`}
      />

      <H3 id="transitions">Transitions</H3>
      <ul>
        <li>
          <strong>Unknown → Clean:</strong> first access misses locally, so RustAG fetches from mainnet,
          caches it, and stamps it <code>Clean</code> (<code>from_remote</code> / <code>mark_clean</code>).
        </li>
        <li>
          <strong>Clean → Clean (refreshed):</strong> the background oracle sync re-fetches <code>Clean</code>{" "}
          oracle accounts every 30s, re-stamping <code>fetched_at</code>.
        </li>
        <li>
          <strong>Clean / Unknown → Dirty:</strong> a local transaction writes the account. Writable
          accounts are derived from the message header&apos;s{" "}
          <code>(num_required_signatures, num_readonly_signed, num_readonly_unsigned)</code> layout and
          marked <code>Dirty</code>; their post-state is persisted. Read-only accounts (programs, oracles,
          sysvars) stay <code>Clean</code> and keep syncing.
        </li>
        <li>
          <strong>any → Pinned:</strong> the override API (<code>rustag override</code>) calls{" "}
          <code>pin()</code>, making the account immune to everything — no background sync, no clobbering.
        </li>
      </ul>
      <p>
        Once an account is <code>Dirty</code> or <code>Pinned</code>, the background mirror never touches it
        again, so user-modified and explicitly-pinned state is preserved deterministically.
      </p>

      <H2 id="oracles">Oracle freshness</H2>
      <p>
        Oracle accounts are the one category RustAG actively keeps fresh. A background loop
        (<code>spawn_oracle_sync</code>) re-fetches <code>Clean</code> oracle accounts on the default 30s
        interval (clamped to a 1s minimum), so Pyth/Switchboard prices don&apos;t go stale under your tests.
      </p>
      <p>
        <PhaseBadge phase={2} /> A <strong>push</strong> path over the standard <code>accountSubscribe</code>{" "}
        WebSocket — the protocol Geyser/Yellowstone providers serve — drops oracle staleness to a p99 target
        of under 2 seconds. It is behind the <code>realtime</code> cargo feature; build with{" "}
        <code>--features realtime</code>.
      </p>
      <Callout variant="tip" title="The invariant that never bends">
        <code>Dirty</code> and <code>Pinned</code> accounts are <strong>never</strong> overwritten by any
        sync — neither the 30s poll nor the realtime push path. Whatever you write or pin stays put, so a
        test stays deterministic.
      </Callout>

      <H2 id="why-staging">Why staging, not testnet</H2>
      <p>
        RustAG is a <em>staging</em> environment, not another testnet — and that distinction is the whole
        value proposition. Devnet pools are empty or fake, faucets cap around ~5 SOL/day while an
        integration suite needs 20–50 SOL/day, and you can&apos;t fork the SVM at a block the way EVM tools
        do. RustAG mirrors the actual mainnet account state on demand instead.
      </p>

      <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border bg-white/[0.015]">
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                &nbsp;
              </th>
              {MATRIX_COLS.map((c) => (
                <th
                  key={c}
                  className="px-3 py-2.5 text-center align-bottom font-mono text-[10px] uppercase leading-tight tracking-[0.1em] text-faint"
                >
                  {c}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {MATRIX.map((row) => (
              <tr
                key={row.tool}
                className={cnRow(row.highlight)}
              >
                <td
                  className={
                    "whitespace-nowrap px-4 py-3 font-mono text-[12.5px] " +
                    (row.highlight ? "font-semibold text-brand" : "text-fg")
                  }
                >
                  {row.tool}
                </td>
                {row.cells.map((ok, i) => (
                  <td key={i} className="px-3 py-3 text-center">
                    {ok ? (
                      <Check size={15} className="mx-auto text-state-clean" />
                    ) : (
                      <Minus size={15} className="mx-auto text-faint" />
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <p>
        Because writes are tracked as <code>Dirty</code> and pins are honored, you can reproduce a mainnet
        incident locally — pin the exact account state and replay the failing transaction against a frozen
        snapshot. State persists across restarts (SQLite via <code>sqlx</code>), so a stagenet behaves like
        a real, always-on environment rather than a throwaway fixture.
      </p>
    </DocArticle>
  );
}

function cnRow(highlight?: boolean): string {
  return (
    "border-b border-border/60 last:border-0 " + (highlight ? "bg-brand/[0.04]" : "")
  );
}
