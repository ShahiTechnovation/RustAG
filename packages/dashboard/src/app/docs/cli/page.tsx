import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";
import { ParamTable } from "@/components/docs/ParamTable";
import { PhaseBadge } from "@/components/docs/PhaseBadge";

export const metadata: Metadata = {
  title: "CLI reference",
  description:
    "Every rustag subcommand — rehearse, forensics, record, verify, attest, serve, and the stagenet management surface.",
};

const TOC: TocItem[] = [
  { id: "conventions", title: "Conventions" },
  { id: "phase-1", title: "Phase 1 — assurance core" },
  { id: "rehearse", title: "rehearse", depth: 3 },
  { id: "verify", title: "verify", depth: 3 },
  { id: "serve", title: "serve", depth: 3 },
  { id: "phase-2", title: "Phase 2 — forensics & corpus" },
  { id: "forensics", title: "forensics", depth: 3 },
  { id: "record", title: "record", depth: 3 },
  { id: "phase-3", title: "Phase 3 — stagenet & dev tools" },
  { id: "create", title: "create", depth: 3 },
  { id: "start", title: "start", depth: 3 },
  { id: "airdrop", title: "airdrop", depth: 3 },
  { id: "override", title: "override", depth: 3 },
  { id: "schedule", title: "schedule", depth: 3 },
  { id: "attest", title: "attest", depth: 3 },
  { id: "scan", title: "scan", depth: 3 },
];

export default function CliPage() {
  return (
    <DocArticle
      eyebrow="Reference"
      title="rustag CLI"
      lead="Every rustag subcommand, with the exact flags. Phase 1 commands (rehearse, verify, serve) are the GroundTruth core — they ship and work today. Phase 2 (forensics, record) and Phase 3 (stagenet dev tools) are in the same binary."
      toc={TOC}
    >
      <H2 id="conventions">Conventions</H2>
      <ul>
        <li>
          A global <code>--log-format text|json</code> (env <code>RUSTAG_LOG_FORMAT</code>,
          default <code>text</code>) applies to every command.
        </li>
        <li>
          <code>--rpc &lt;URL&gt;</code> / env <code>RUSTAG_MAINNET_RPC</code> — the mainnet RPC
          endpoint for closure resolution. Required for live rehearsals; not needed for{" "}
          <code>--demo</code> or <code>--offline</code>.
        </li>
        <li>
          All output paths default to the working directory. Use <code>--out</code> and{" "}
          <code>--closure</code> to customize.
        </li>
      </ul>

      {/* ---------------------------------------------------------- Phase 1 */}
      <H2 id="phase-1">
        Phase 1 — assurance core <PhaseBadge phase={1} className="ml-1" />
      </H2>

      <H3 id="rehearse">rustag rehearse</H3>
      <p>
        The primary command. Fetches the proposal or payload, resolves the full account closure,
        runs the sealed two-pass rehearsal, and writes a signed{" "}
        <code>EvidenceBundle</code> + portable pre-state closure.
      </p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          {
            name: "--proposal <PUBKEY>",
            type: "—",
            desc: "Squads v4 VaultTransaction proposal address. RustAG fetches, Borsh-decodes, and rehearses it.",
          },
          {
            name: "--payload <BASE64>",
            type: "—",
            desc: "Base64 bincode-serialized VersionedTransaction to rehearse directly.",
          },
          {
            name: "--rpc <URL>",
            type: "$RUSTAG_MAINNET_RPC",
            desc: "Mainnet RPC for closure resolution. Required for --proposal and --payload.",
          },
          {
            name: "--offline",
            type: "off",
            desc: "Skip the mirror entirely. The payload must be self-contained (no external accounts).",
          },
          {
            name: "--demo",
            type: "off",
            desc: "Run the built-in ownership-takeover demo. No RPC needed.",
          },
          {
            name: "--signer <PATH>",
            type: "ephemeral",
            desc: "Path to a Solana JSON keypair (64-byte array) to sign the bundle with.",
          },
          {
            name: "--out <PATH>",
            type: "groundtruth-bundle.json",
            desc: "Where to write the signed EvidenceBundle.",
          },
          {
            name: "--closure <PATH>",
            type: "groundtruth-closure.json",
            desc: "Where to write the portable pre-state closure (needed for offline verify).",
          },
          {
            name: "--fail-on <SEVERITY>",
            type: "off",
            desc: "Exit non-zero if any alarm reaches this severity (info | low | medium | high | critical). Use in CI.",
          },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`# Squads v4 proposal
rustag rehearse \\
  --proposal 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU \\
  --rpc $HELIUS_RPC \\
  --fail-on high

# Raw transaction
rustag rehearse --payload <BASE64_TX> --rpc $RPC

# Built-in demo (no network)
rustag rehearse --demo`}
      />

      <H3 id="verify">rustag verify &lt;BUNDLE&gt;</H3>
      <p>
        Verify an EvidenceBundle offline. Checks the Ed25519 signature, re-derives the
        pre-state root from the closure, and confirms fidelity grade. Exits non-zero if INVALID.
      </p>
      <ParamTable
        cols={["Flag", "Type", "Meaning"]}
        rows={[
          {
            name: "--closure <PATH>",
            type: "groundtruth-closure.json",
            desc: "The portable pre-state closure to verify against.",
          },
          {
            name: "--signature-only",
            type: "flag",
            desc: "Only check the Ed25519 signature; skip re-deriving the state root.",
          },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`rustag verify groundtruth-bundle.json --closure groundtruth-closure.json`}
      />

      <H3 id="serve">rustag serve</H3>
      <p>
        Start the REST API server (<code>POST /api/rehearse</code>,{" "}
        <code>POST /api/verify</code>, <code>GET /api/health</code>) for the dashboard and
        external integrations. Long-running foreground process.
      </p>
      <ParamTable
        cols={["Flag / Env", "Default", "Meaning"]}
        rows={[
          {
            name: "RUSTAG_MAINNET_RPC",
            type: "—",
            desc: "Mainnet RPC URL. Required for live rehearsals via the API.",
          },
          {
            name: "RUSTAG_BIND_HOST",
            type: "127.0.0.1",
            desc: "Host to bind. Set to 0.0.0.0 on Render/Docker.",
          },
          {
            name: "RUSTAG_DEMO_MODE",
            type: "0",
            desc: "Cap airdrops and disable write operations (public demo safety).",
          },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
rustag serve           # REST API on $PORT or 9000`}
      />

      {/* ---------------------------------------------------------- Phase 2 */}
      <H2 id="phase-2">
        Phase 2 — forensics & corpus <PhaseBadge phase={2} className="ml-1" />
      </H2>

      <H3 id="forensics">rustag forensics &lt;SIGNATURE&gt;</H3>
      <p>
        Re-execute a historical mainnet transaction deterministically. In counterfactual mode,
        substitute the deployed program with a patched ELF and emit a{" "}
        <strong>BLOCKED</strong> or <strong>REPRODUCED</strong> verdict.
      </p>
      <ParamTable
        cols={["Flag", "Type", "Meaning"]}
        rows={[
          {
            name: "<SIGNATURE>",
            type: "required",
            desc: "Base-58 transaction signature to fetch and re-execute.",
          },
          { name: "--rpc <URL>", type: "$RUSTAG_MAINNET_RPC", desc: "Mainnet RPC." },
          {
            name: "--patch <PATH>",
            type: "—",
            desc: "Path to a patched program ELF to substitute before re-execution.",
          },
          {
            name: "--patch-program <PUBKEY>",
            type: "—",
            desc: "Program ID to patch. Required when --patch is set.",
          },
          {
            name: "--json",
            type: "off",
            desc: "Emit machine-readable JSON verdict instead of human text.",
          },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`# Re-execute a historical transaction
rustag forensics <SIGNATURE> --rpc $RPC

# Counterfactual: would this patch have stopped it?
rustag forensics <SIGNATURE> \\
  --rpc $RPC \\
  --patch ./patched.so \\
  --patch-program <PROGRAM_ID>`}
      />

      <H3 id="record">rustag record</H3>
      <p>
        Build a real mainnet traffic corpus for a watched program — used as input to the
        upgrade-rehearsal CI gate.
      </p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          { name: "--program <PUBKEY>", type: "—", required: true, desc: "Program to watch." },
          { name: "--rpc <URL>", type: "$RUSTAG_MAINNET_RPC", desc: "Mainnet RPC." },
          { name: "--limit <N>", type: "100", desc: "Max transactions to record." },
          { name: "--out <PATH>", type: "corpus.json", desc: "Output corpus file." },
          { name: "--append", type: "off", desc: "Append to an existing corpus file." },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`rustag record --program <PROGRAM_ID> --rpc $RPC --out corpus.json --limit 500`}
      />

      {/* ---------------------------------------------------------- Phase 3 */}
      <H2 id="phase-3">
        Phase 3 — stagenet & dev tools <PhaseBadge phase={3} className="ml-1" />
      </H2>
      <Callout variant="info">
        These commands manage a persistent local SVM stagenet — useful for integration testing
        and the dashboard. For pre-execution assurance, use <code>rustag rehearse</code> instead.
      </Callout>

      <H3 id="create">rustag create &lt;NAME&gt;</H3>
      <p>Register a new stagenet.</p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          { name: "--rpc-port <PORT>", type: "8899", desc: "JSON-RPC port." },
          { name: "--ws-port <PORT>", type: "rpc_port + 1", desc: "WebSocket port." },
          { name: "--api-port <PORT>", type: "9000", desc: "REST API port." },
          {
            name: "--mainnet-rpc <URL>",
            type: "$RUSTAG_MAINNET_RPC",
            desc: "Endpoint the lazy mirror fetches from.",
          },
          { name: "--no-mirror", type: "off", desc: "Fully offline stagenet." },
        ]}
      />

      <H3 id="start">rustag start [NAME]</H3>
      <p>Run the JSON-RPC, WebSocket, and REST servers (long-running, foreground).</p>
      <CodeBlock
        lang="bash"
        code={`rustag create demo
rustag start demo --preload pyth raydium`}
      />

      <H3 id="airdrop">rustag airdrop [-s NAME] &lt;PUBKEY&gt; &lt;SOL&gt;</H3>
      <p>
        Credit SOL to a wallet via the running stagenet. Airdrops are unlimited — capped only
        to prevent <code>u64</code> overflow.
      </p>
      <CodeBlock lang="bash" code={`rustag airdrop -s demo <YOUR_WALLET> 1000`} />

      <H3 id="override">rustag override [-s NAME] --pubkey &lt;PK&gt;</H3>
      <p>
        Set (and pin) account state via the running stagenet. Pass <code>--lamports</code>{" "}
        <em>or</em> <code>--token-balance</code>.
      </p>
      <CodeBlock
        lang="bash"
        code={`rustag override -s demo --pubkey <PUBKEY> --lamports 5000000000`}
      />

      <H3 id="schedule">rustag schedule [-s NAME] &lt;SUBCOMMAND&gt;</H3>
      <p>Manage recurring on-chain activities (add, list, rm, toggle).</p>
      <CodeBlock
        lang="bash"
        code={`rustag schedule -s demo add nightly "@every 30s" --airdrop <PUBKEY> --sol 5
rustag schedule -s demo list`}
      />

      <H3 id="attest">rustag attest [-s NAME]</H3>
      <p>
        Produce a signed, Merkle-rooted attestation of staged state (operates offline against the
        persisted store).
      </p>
      <CodeBlock
        lang="bash"
        code={`rustag attest -s demo --program <PROGRAM_ID>
rustag verify .rustag/demo.attestation.json -s demo`}
      />

      <H3 id="scan">rustag scan [-s NAME]</H3>
      <p>Scan recorded transactions for exploit signatures — a CI gate.</p>
      <CodeBlock
        lang="bash"
        code={`rustag scan -s demo --fail-on high`}
      />

      <Callout variant="info">
        Looking for the programmatic surface? See the{" "}
        <a href="/docs/sdk">SDK & API reference</a> for <code>@rustag/sdk</code>, the REST
        contract, and the Solana JSON-RPC methods the stagenet implements.
      </Callout>
    </DocArticle>
  );
}
