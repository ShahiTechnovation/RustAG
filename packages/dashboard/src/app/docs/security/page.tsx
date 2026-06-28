import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";

export const metadata: Metadata = {
  title: "Trust & security",
  description:
    "RustAG's threat model, verifiable attestation and tamper-evident audit log, service-level objectives, an honest list of early-access limitations, and an FAQ.",
};

const TOC: TocItem[] = [
  { id: "threat-model", title: "Security & threat model" },
  { id: "isolation", title: "Tenant isolation", depth: 3 },
  { id: "attestation", title: "Attestation integrity", depth: 3 },
  { id: "audit-log", title: "Audit-log tamper-evidence", depth: 3 },
  { id: "service-levels", title: "Service levels" },
  { id: "limitations", title: "Known limitations" },
  { id: "faq", title: "FAQ" },
];

const FAQ = [
  {
    q: "What does it cost to read real mainnet state?",
    a: "Zero SOL. On first access RustAG lazily mirrors the mainnet account into the stagenet (mainnet data is public), then keeps oracles fresh in the background. You can airdrop unlimited SOL — no faucet, no cap — so an integration suite can actually run. Airdrops are capped only to prevent u64 overflow.",
  },
  {
    q: "Is this safe to run — can I break anything on-chain?",
    a: "No. A stagenet is an isolated environment; transactions you send execute locally against mirrored state and spend zero real SOL. Reading mainnet only pulls public account data on demand; it never writes to mainnet. You test unaudited code here precisely so you don't test it on mainnet.",
  },
  {
    q: "How is this different from solana-test-validator?",
    a: "The test validator gives you an empty local cluster — no real Raydium pools, no real Pyth prices, and you can't fork the chain the way you can on Ethereum. RustAG is a mainnet-mirroring stagenet: it lazily pulls real, current mainnet accounts on first access, so your code runs against live DeFi state.",
  },
  {
    q: "How does it compare to Bankrun / LiteSVM?",
    a: "RustAG's runtime is LiteSVM-backed, so it shares that fast in-process execution model — but it adds the lazy mainnet mirror, a dirty/clean/pinned state machine, SQLite persistence, a Solana-compatible JSON-RPC + WebSocket + REST server, a CLI, a dashboard, and (Phase 3) signed attestations and an exploit scanner. Bankrun/LiteSVM are in-process test harnesses; RustAG is a persistent, drop-in cluster.",
  },
  {
    q: "Is execution deterministic / reproducible?",
    a: "Yes, and it's provable. rustag attest writes a signed, Merkle-rooted manifest committing to the exact pubkey-sorted account set and ordered transaction outcomes; rustag verify <file> checks it offline with no server and no network, exiting non-zero if INVALID. The rustag-replay crate adds checkpointing and deterministic journal replay.",
  },
  {
    q: "Can I run a full Jupiter swap end-to-end today?",
    a: "Your own deployed program reading real mainnet state works now. Executing a foreign on-chain program by loading its BPF bytecode (a complete Jupiter swap end-to-end) is the remaining boundary — it needs the Phase 2+ program-loading. See Known limitations.",
  },
  {
    q: "Can I gate CI on this?",
    a: "Yes. rustag scan -s <name> --fail-on <severity> scans recorded transactions for exploit signatures and exits non-zero at or above the given severity, so it's a CI gate, not just a report. The GitHub Action spins up an ephemeral per-PR stagenet, runs your command against real mainnet state, posts a PR summary, and tears down.",
  },
];

function SloTable({ rows }: { rows: [string, string, string][] }) {
  return (
    <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-border bg-white/[0.015]">
            {["Objective", "Target", "Status"].map((c) => (
              <th
                key={c}
                className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint"
              >
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r[0]} className="border-b border-border/60 align-top last:border-0">
              <td className="px-4 py-3 text-[13px] text-fg">{r[0]}</td>
              <td className="whitespace-nowrap px-4 py-3 font-mono text-[12px] text-brand">{r[1]}</td>
              <td className="px-4 py-3 text-[13px] text-muted">{r[2]}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function SecurityPage() {
  return (
    <DocArticle
      eyebrow="Trust"
      title="Trust & security"
      lead="RustAG is honest about where the MVP ends. This page covers the hosted threat model, the cryptographic trust layer, service-level objectives, and a plain list of what does and does not work yet."
      toc={TOC}
    >
      <H2 id="threat-model">Security &amp; threat model</H2>
      <p>
        The threat model is scoped to the <strong>hosted, multi-tenant</strong> product. The open-source
        local CLI runs entirely on your own machine and is out of scope — when you run a stagenet locally,
        you are your own trust boundary.
      </p>

      <H3 id="isolation">Tenant isolation</H3>
      <p>
        Tenants are mutually distrusting and may run <strong>arbitrary, untrusted Solana program
        bytecode</strong> inside their stagenet — that is the point of the product. The hard boundary is
        therefore between one stagenet runtime and everything else; a stagenet executes adversarial code and
        is treated as hostile. Cross-tenant isolation is defended in depth:
      </p>
      <ul>
        <li>Each stagenet has its own account store and data directory — no shared account namespace.</li>
        <li>
          Every <code>/v1/*</code> query is filtered by the authenticated <code>tenant_id</code>; a lookup
          that doesn&apos;t match returns <code>NotFound</code>, so one tenant cannot even enumerate
          another&apos;s slugs.
        </li>
        <li>
          Each stagenet runs as a separate OS process today; production hardening runs each pod under{" "}
          <code>runtimeClassName: kata</code> (Firecracker microVM) with per-tenant CPU/memory quotas.
        </li>
        <li>
          API keys are SHA-256-digested at rest, shown once, tenant-scoped, and revocable; upstream RPC keys
          (which carry <code>?api-key=</code>) are deliberately never logged.
        </li>
      </ul>

      <H3 id="attestation">Attestation integrity</H3>
      <p>
        The <code>rustag-attest</code> crate produces a signed, Merkle-rooted proof of the exact
        mainnet-derived state a program was tested against. The signing digest is built from a fixed field
        order with length-prefixed, domain-tagged fields rather than from JSON — JSON key/whitespace ordering
        is not canonical and must never affect what a signature commits to. The <code>state_root</code> is a
        binary SHA-256 Merkle root over the <strong>pubkey-sorted</strong> account set, with leaves
        (<code>0x00</code>) and nodes (<code>0x01</code>) domain-separated to prevent second-preimage
        attacks. Account leaves commit to consensus-visible fields only; the internal dirty/clean/pinned
        bookkeeping is deliberately excluded.
      </p>
      <CodeBlock
        lang="rust"
        filename="sign a manifest, then verify it offline against a concrete account set"
        code={`let attestation = Attestation::create(manifest, &keypair);

// Recompute the state root from \`accounts\`, confirm it matches the
// manifest, and check the Ed25519 signature — no server, no network.
let report = attestation.verify_against(&accounts)?;
assert!(report.is_valid());

// Forging any signed field (e.g. att.manifest.slot = 999) breaks the signature.`}
      />

      <H3 id="audit-log">Audit-log tamper-evidence</H3>
      <p>
        <code>AuditLog</code> is an append-only, <strong>hash-chained</strong> log — the SOC 2 groundwork.
        Each entry carries a monotonic <code>seq</code>, a <code>prev_hash</code>, and its own{" "}
        <code>hash</code>; the chain is genesis-anchored at the all-zero hash. Any insertion, deletion, or
        edit anywhere in the log breaks the chain from that point forward, and <code>verify()</code> returns{" "}
        <code>Err(index)</code> at the exact first inconsistent entry.
      </p>

      <H2 id="service-levels">Service levels</H2>
      <p>
        SLO targets apply to the <strong>hosted</strong> control plane and stagenets — local/CLI stagenets
        run on your own machine and are best-effort. Targets are deliberately modest; under-promising at
        this stage is intentional.
      </p>
      <SloTable
        rows={[
          ["Control-plane API (/v1/*) uptime", "99.5% / mo", "Target"],
          ["Cloud stagenet creation within 30s", "99% of attempts", "Target — health-gated start"],
          ["getAccountInfo (cache hit)", "p99 < 50 ms", "Target"],
          ["getAccountInfo (cold mainnet fetch)", "p99 < 2 s", "Target"],
          ["Oracle price staleness (realtime)", "p99 < 2 s", "Target"],
          ["Cross-tenant data-access incidents", "hard 0", "Enforced + tested"],
          ["Stagenet wake-from-sleep", "p99 < 15 s", "Aspirational"],
        ]}
      />
      <p>
        The error budget is the inverse of the availability target (0.5%/month); when exhausted, reliability
        work ships before features. Failure modes are explicit: a cold-fetch mainnet RPC failure serves
        stale cached data with a warning and never panics; on a realtime WebSocket disconnect the caller
        reconnects while <code>Clean</code> accounts keep their last value and <code>Dirty</code>/
        <code>Pinned</code> accounts are never touched.
      </p>

      <H2 id="limitations">Known limitations</H2>
      <Callout variant="early" title="The headline limitation">
        <strong>Your own deployed program reading real mainnet state works today.</strong> What does{" "}
        <em>not</em> work end-to-end yet is executing an arbitrary foreign on-chain program by loading its
        BPF bytecode — a full Jupiter swap, start to finish. You can preload real Pyth/Raydium/Jupiter{" "}
        <em>accounts</em> and read them, airdrop unlimited SOL, and send/confirm transactions from your own
        code; you cannot yet invoke an unmodified third-party program by its on-chain bytecode and have it
        execute. That needs the Phase 2+ program-loading.
      </Callout>
      <p>
        RustAG implements the Phase 2 spec with deliberate single-node substitutions — each satisfies the
        same contract as the eventual target, so the swap is additive, not a rewrite:
      </p>
      <ul>
        <li>
          <strong>Streaming mirror</strong> — an <code>accountSubscribe</code> WebSocket instead of native
          Yellowstone gRPC. Sub-second push with zero lock-in. Live filter updates on an open subscription
          and built-in auto-reconnect are not yet done.
        </li>
        <li>
          <strong>Datastore</strong> — SQLite + moka instead of Postgres + Redis; correct for the single-node
          MVP. The Postgres migration and Row-Level-Security policies are not yet done.
        </li>
        <li>
          <strong>Multi-tenant isolation</strong> — child-process isolation instead of Kata + Kubernetes;
          the <code>kube-rs</code> orchestrator and a running Kata cluster are not yet done.
        </li>
        <li>
          <strong>Auth &amp; billing</strong> — SHA-256-digested API keys instead of Clerk + Stripe; billing
          is deferred.
        </li>
        <li>
          <strong>Observability</strong> — <code>tracing</code> spans + a JSON <code>/api/metrics</code>{" "}
          time-series; a Prometheus-format <code>/metrics</code> scrape endpoint is deferred.
        </li>
      </ul>
      <p>
        Other open items: <code>cargo audit</code> / <code>cargo deny</code> are not yet wired into CI;
        examples exist under <code>examples/</code> but CI does not execute them; crates are not yet
        published to crates.io / npm; client-compatibility is validated against <code>@solana/web3.js</code>{" "}
        but not yet the <code>@solana/kit</code> or Rust <code>solana-client</code> matrices.
      </p>

      <H2 id="faq">FAQ</H2>
      <div className="mt-2 divide-y divide-border border-y border-border">
        {FAQ.map((item) => (
          <div key={item.q} className="py-5">
            <p className="font-display text-base font-semibold tracking-tight text-fg">{item.q}</p>
            <p className="mt-2 text-sm leading-relaxed text-muted">{item.a}</p>
          </div>
        ))}
      </div>

      <CodeBlock
        lang="bash"
        filename="the Phase 3 trust layer in three commands"
        code={`rustag attest -s demo                       # -> .rustag/demo.attestation.json (signed, Merkle-rooted)
rustag verify demo.attestation.json -s demo # offline; exits non-zero if INVALID
rustag scan -s demo --fail-on high          # CI gate: exits non-zero at/above 'high'`}
      />
    </DocArticle>
  );
}
