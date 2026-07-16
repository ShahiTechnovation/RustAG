import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";

export const metadata: Metadata = {
  title: "Trust & security",
  description:
    "RustAG's threat model, Ed25519 EvidenceBundle integrity, N-of-M provenance, Grade A verification, service-level objectives, honest early-access limitations, and FAQ.",
};

const TOC: TocItem[] = [
  { id: "threat-model", title: "Security & threat model" },
  { id: "bundle-integrity", title: "EvidenceBundle integrity", depth: 3 },
  { id: "grade-a", title: "Grade A verification", depth: 3 },
  { id: "audit-log", title: "Audit-log tamper-evidence", depth: 3 },
  { id: "service-levels", title: "Service levels" },
  { id: "limitations", title: "Known limitations" },
  { id: "faq", title: "FAQ" },
];

const FAQ = [
  {
    q: "Can a malicious proposer forge an EvidenceBundle?",
    a: "No. A valid Grade A bundle requires a valid Ed25519 signature over the pre_state_root + post_state_root + semantic_diff + alarms. The pre_state_root is derived by independently re-fetching the closure from mainnet — a forged pre-state would not match what any honest verifier fetches. A compromised rehearser UI cannot produce a bundle whose pre-state root is consistent with mainnet AND whose signature is valid under a known attester key.",
  },
  {
    q: "What does Grade A guarantee?",
    a: "Grade A means the EvidenceBundle is deterministically re-executable: every account in the closure was fetched from mainnet at a recorded slot, content-hashed, and the pre_state_root matches. Any verifier who re-fetches those pubkeys at the same slot (or uses the portable closure.json) will produce the same pre_state_root, execute the same payload, and produce the same post_state_root — without trusting the rehearser.",
  },
  {
    q: "What is Grade B?",
    a: "Grade B means one or more accounts in the touch set could not be resolved (rate-limited RPC, account does not exist yet, etc.). The bundle is still signed but the post_state_root cannot be independently reproduced without a complete closure. Treat Grade B bundles as advisory — they show what the rehearser saw, but they are not independently verifiable.",
  },
  {
    q: "Is this safe to run — can anything break on mainnet?",
    a: "No. The rehearsal runs inside a sealed, isolated LiteSVM instance with no write path to mainnet. The ingest layer only calls getMultipleAccounts (public read-only RPC). Nothing is signed and broadcast to any Solana cluster. The actual proposal remains pending in Squads until M-of-N signers manually approve it.",
  },
  {
    q: "How is this different from just simulating a transaction in a wallet?",
    a: "Wallet simulators run simulateTransaction via the cluster RPC — the result is a log dump with no pre-state commitment, no semantic diff, no invariant alarms, and no signature. Anyone can alter the simulation result before showing it to a signer. RustAG's EvidenceBundle commits to the exact pre-state used, what changed semantically (not just logs), which alarm rules fired, and signs all of it — so a signer can verify the bundle independently before approving.",
  },
  {
    q: "What invariant rules fire alarms?",
    a: "Phase 1 ships 6 rules: upgrade_authority_changed (CRITICAL), large_sol_drain (HIGH, >100 SOL delta), nonce_authority_combo (MEDIUM — nonce + authority rotation in same tx), program_freeze_guard (HIGH), token_authority_changed (HIGH), account_closed_drain (MEDIUM). More rules are additive in future phases.",
  },
  {
    q: "Can I gate CI on an EvidenceBundle?",
    a: "Yes. rustag rehearse --proposal <PUBKEY> --fail-on high exits non-zero if any alarm reaches HIGH or above. The upgrade-rehearsal.yml GitHub Action in the repo shows how to wire this into a PR gate: it fetches the proposal, runs the rehearsal, and blocks merge if any HIGH/CRITICAL alarms fire.",
  },
  {
    q: "Is the closed-source hosted service the only way to use this?",
    a: "No. Everything described in Phase 1 runs entirely from source — cargo build --release, then rustag rehearse. The hosted service (Phase 2+) adds the Evidence Registry, Squads UI embed, and N-of-M provenance chain. The CLI + local server is and will remain open source.",
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
      lead="RustAG's security story is about the EvidenceBundle — not the rehearser. The bundle must be independently verifiable even if the rehearsal service is compromised. This page covers the cryptographic integrity model, Grade A verification, service-level objectives, and an honest list of what the Phase 1 MVP does and does not do."
      toc={TOC}
    >
      <H2 id="threat-model">Security & threat model</H2>
      <p>
        The primary adversary in the GroundTruth threat model is a{" "}
        <strong>malicious or compromised transaction proposer</strong> — not the rehearsal
        service itself. A multisig signer should be able to verify an EvidenceBundle without
        trusting:
      </p>
      <ul>
        <li>The proposer&apos;s UI or wallet.</li>
        <li>The rehearsal service that produced the bundle.</li>
        <li>The RPC endpoint the rehearsal service used.</li>
      </ul>
      <p>
        The hosted service threat model (Phase 2+) also covers tenant isolation:{" "}
        each rehearsal runs in its own context with no shared account namespace. The local CLI
        runs entirely on your own machine and is out of scope for multi-tenant isolation.
      </p>

      <H3 id="bundle-integrity">EvidenceBundle integrity</H3>
      <p>
        The <code>rustag-attest</code> crate signs a structured digest — not a JSON blob. The
        signing input is built from a fixed field order with length-prefixed,
        domain-tagged fields:
      </p>
      <ol>
        <li>
          <code>pre_state_root</code> — SHA-256 Merkle root over the pubkey-sorted closure.
          Leaves are domain-separated with <code>0x00</code>; nodes with <code>0x01</code> to
          prevent second-preimage attacks. Account leaves commit to consensus-visible fields
          only (lamports, data, owner, executable, rent_epoch); the internal
          dirty/clean/pinned bookkeeping is deliberately excluded.
        </li>
        <li>
          <code>payload_hash</code> — SHA-256 of the raw bincode-serialized transaction bytes.
        </li>
        <li>
          <code>post_state_root</code> — same Merkle construction over post-execution accounts.
        </li>
        <li>
          <code>semantic_diff_hash</code> — SHA-256 of the canonical JSON-serialized diff.
        </li>
        <li>
          <code>alarms_hash</code> — SHA-256 of the canonical JSON-serialized alarm list.
        </li>
        <li>
          <code>fidelity_grade</code> — 1 byte: 0x41 = A, 0x42 = B.
        </li>
      </ol>
      <CodeBlock
        lang="rust"
        filename="verify a bundle offline — no server, no network"
        code={`use rustag_attest::EvidenceBundle;

let bundle: EvidenceBundle = serde_json::from_str(&bundle_json)?;
let closure = PortableBundle::from_file("groundtruth-closure.json")?;

// Re-derives pre_state_root from closure, re-executes payload,
// checks Ed25519 signature — exits Err if INVALID.
let report = bundle.verify_against(&closure)?;
assert!(report.grade == FidelityGrade::A);
assert!(report.alarms.is_empty() || report.alarms.iter().all(|a| a.severity < Severity::High));`}
      />

      <H3 id="grade-a">Grade A verification</H3>
      <p>
        A Grade A bundle is <strong>deterministically re-executable</strong> by any verifier who
        has the portable <code>closure.json</code>. The verification steps are:
      </p>
      <ol>
        <li>
          Re-derive <code>pre_state_root</code> from closure → confirm it matches the bundle.
        </li>
        <li>Execute the payload in a fresh LiteSVM instance loaded from the closure.</li>
        <li>
          Re-derive <code>post_state_root</code> → confirm it matches the bundle.
        </li>
        <li>Check the Ed25519 signature over the structured digest described above.</li>
      </ol>
      <p>
        Steps 1–4 require zero network access — just the bundle + closure files and a Rust
        binary. The attester&apos;s pubkey is embedded in the bundle and is the only trust
        anchor a verifier needs to establish out-of-band (e.g., from the Squads multisig&apos;s
        governance configuration).
      </p>
      <CodeBlock
        lang="bash"
        code={`# Three-command verification workflow
rustag rehearse --proposal <PUBKEY> --rpc $RPC    # produces bundle + closure
rustag verify bundle.json --closure closure.json   # offline, exit 1 if INVALID
# ✓ Grade A · Signature valid · pre/post roots match`}
      />

      <H3 id="audit-log">Audit-log tamper-evidence</H3>
      <p>
        <code>AuditLog</code> is an append-only, <strong>hash-chained</strong> log — the SOC 2
        groundwork. Each entry carries a monotonic <code>seq</code>, a <code>prev_hash</code>,
        and its own <code>hash</code>; the chain is genesis-anchored at the all-zero hash. Any
        insertion, deletion, or edit anywhere in the log breaks the chain from that point
        forward, and <code>verify()</code> returns <code>Err(index)</code> at the exact first
        inconsistent entry.
      </p>

      <H2 id="service-levels">Service levels</H2>
      <p>
        SLO targets apply to the <strong>hosted</strong> service (Phase 2+). Local CLI
        rehearsals run on your own machine with your own RPC key and are best-effort. Targets
        are deliberately modest; under-promising at this stage is intentional.
      </p>
      <SloTable
        rows={[
          ["Rehearsal API (POST /api/rehearse) uptime", "99.5% / mo", "Target"],
          ["Grade A bundle latency (Helius RPC)", "p99 < 8 s", "Target"],
          ["Grade A bundle latency (cached closure)", "p99 < 2 s", "Target"],
          ["getAccountInfo closure hit", "p99 < 50 ms", "Target"],
          ["Oracle price staleness (realtime, Phase 2)", "p99 < 2 s", "Target"],
          ["Cross-tenant data-access incidents", "hard 0", "Enforced + tested"],
          ["Evidence Registry write durability", "99.99%", "Target"],
        ]}
      />

      <H2 id="limitations">Known limitations</H2>
      <Callout variant="early" title="Phase 1 honest boundary">
        <strong>
          rehearse + verify + forensics work today — build from source and run locally.
        </strong>{" "}
        The Evidence Registry (hosted append-only bundle store), Squads UI embed (signer-review
        panel), Yellowstone gRPC real-time recording, and per-flow pricing are Phase 2 and are
        not yet released.
      </Callout>
      <p>
        Other known limitations in Phase 1:
      </p>
      <ul>
        <li>
          <strong>Foreign program execution</strong> — the closure resolver fetches program
          accounts verbatim (readable and present), but does not yet JIT-load BPF bytecode from
          the program-data account. Rehearsing a Squads proposal that itself invokes a complex
          foreign program (e.g., a full Jupiter swap CPI) may produce a Grade B bundle if the
          bytecode is unavailable. Your own deployed program reading real mainnet state works today.
        </li>
        <li>
          <strong>Address lookup tables</strong> — v0 transactions with ALTs are resolved via
          a mirror fetch; ALT-heavy DeFi bundles are resolved but may produce Grade B if any
          ALT account is unavailable.
        </li>
        <li>
          <strong>Datastore</strong> — SQLite + moka in the stagenet runtime (not Postgres);
          correct for the single-node local MVP.
        </li>
        <li>
          <strong>Observability</strong> — <code>tracing</code> spans + a JSON{" "}
          <code>/api/metrics</code> time-series; Prometheus scrape endpoint deferred.
        </li>
        <li>
          <strong>Supply-chain CI</strong> — <code>cargo audit</code> /{" "}
          <code>cargo deny</code> are not yet wired into CI; crates not yet published to
          crates.io / npm.
        </li>
      </ul>

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
        filename="the GroundTruth trust layer in three commands"
        code={`rustag rehearse --proposal <PUBKEY> --rpc $RPC   # → bundle.json + closure.json (signed, Grade A)
rustag verify bundle.json --closure closure.json  # offline; exits non-zero if INVALID
rustag rehearse --proposal <PUBKEY> --rpc $RPC --fail-on high  # CI gate: exit 1 on HIGH/CRITICAL alarms`}
      />
    </DocArticle>
  );
}
