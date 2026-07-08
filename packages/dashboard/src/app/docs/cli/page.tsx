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
    "Every rustag subcommand with the exact flags from the clap definitions — create, start, airdrop, override, preload, logs, doctor, schedule, metrics, attest, verify, scan, tree.",
};

const TOC: TocItem[] = [
  { id: "conventions", title: "Conventions" },
  { id: "phase-1", title: "Phase 1 — core" },
  { id: "create", title: "create", depth: 3 },
  { id: "start", title: "start", depth: 3 },
  { id: "stop-status-list", title: "stop · status · list", depth: 3 },
  { id: "airdrop", title: "airdrop", depth: 3 },
  { id: "override", title: "override", depth: 3 },
  { id: "preload", title: "preload", depth: 3 },
  { id: "logs", title: "logs", depth: 3 },
  { id: "doctor", title: "doctor", depth: 3 },
  { id: "phase-2", title: "Phase 2" },
  { id: "schedule", title: "schedule", depth: 3 },
  { id: "metrics", title: "metrics", depth: 3 },
  { id: "phase-3", title: "Phase 3" },
  { id: "attest", title: "attest", depth: 3 },
  { id: "verify", title: "verify", depth: 3 },
  { id: "scan", title: "scan", depth: 3 },
  { id: "tree", title: "tree", depth: 3 },
];

export default function CliPage() {
  return (
    <DocArticle
      eyebrow="Reference"
      title="rustag CLI"
      lead="Every rustag subcommand, with the exact flags from the clap definitions. Phase 1 is the working local MVP; Phase 2 and Phase 3 subcommands ship in the same binary and are newer."
      toc={TOC}
    >
      <H2 id="conventions">Conventions</H2>
      <ul>
        <li>
          <code>-s, --stagenet &lt;NAME&gt;</code> selects which stagenet a command targets. It is optional
          when only one stagenet exists.
        </li>
        <li>
          A global <code>--log-format text|json</code> (env <code>RUSTAG_LOG_FORMAT</code>, default{" "}
          <code>text</code>) applies to every command.
        </li>
        <li>
          Client commands (<code>airdrop</code>, <code>override</code>, <code>preload</code>,{" "}
          <code>logs</code>, <code>status</code>) reach a <em>running</em> stagenet over its REST API; start
          one first with <code>rustag start</code>.
        </li>
      </ul>

      {/* ---------------------------------------------------------- Phase 1 */}
      <H2 id="phase-1">
        Phase 1 — core <PhaseBadge phase={1} className="ml-1" />
      </H2>

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
            desc: "Endpoint the lazy mirror fetches from. Falls back to a built-in default if unset.",
          },
          { name: "--no-mirror", type: "off", desc: "Fully offline stagenet (no mainnet fetches)." },
        ]}
      />

      <H3 id="start">rustag start [NAME]</H3>
      <p>
        Run the JSON-RPC, WebSocket, and REST servers (long-running, foreground). <code>NAME</code> is
        optional if only one stagenet exists.
      </p>
      <ParamTable
        cols={["Flag", "Arity", "Meaning"]}
        rows={[
          {
            name: "--preload <NAMES…>",
            type: "variadic",
            desc: "Extra programs/oracles to preload on startup (e.g. --preload jupiter pyth). Merged with RustAG.toml preload targets.",
          },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`rustag create alt --rpc-port 18899 --api-port 19000
rustag start alt --preload jupiter pyth raydium`}
      />

      <H3 id="stop-status-list">rustag stop · status · list</H3>
      <ul>
        <li>
          <code>rustag stop [-s NAME]</code> — stop a running stagenet (best-effort via its{" "}
          <code>./.rustag/&lt;name&gt;.pid</code> file; on Windows filtered to the <code>rustag.exe</code>{" "}
          image so a recycled PID is never killed).
        </li>
        <li>
          <code>rustag status [-s NAME]</code> — show id, running state, RPC/REST URLs, mainnet RPC, mirror
          enabled/disabled, and account/transaction counts.
        </li>
        <li>
          <code>rustag list</code> — list all stagenets (NAME, ID, RPC, STATUS). No flags.
        </li>
      </ul>

      <H3 id="airdrop">rustag airdrop [-s NAME] &lt;PUBKEY&gt; &lt;SOL&gt;</H3>
      <p>
        Credit SOL to a wallet via the running stagenet. Positional <code>&lt;PUBKEY&gt;</code> (validated)
        and <code>&lt;SOL&gt;</code> (f64). Airdrops are unlimited — capped only to prevent{" "}
        <code>u64</code> overflow.
      </p>
      <CodeBlock lang="bash" code={`rustag airdrop -s demo <YOUR_WALLET> 1000`} />

      <H3 id="override">rustag override [-s NAME] --pubkey &lt;PK&gt;</H3>
      <p>
        Set (and pin) account state via the running stagenet. Pass <code>--lamports</code> <em>or</em>{" "}
        <code>--token-balance</code> — at least one is required.
      </p>
      <ParamTable
        cols={["Flag", "Type", "Meaning"]}
        rows={[
          { name: "--pubkey <PUBKEY>", type: "Pubkey", required: true, desc: "Account to override." },
          { name: "--lamports <N>", type: "u64", desc: "Set the lamport balance." },
          { name: "--token-balance <N>", type: "u64", desc: "Set an SPL token account's raw amount." },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`rustag override -s demo --pubkey <PUBKEY> --lamports 5000000000
rustag override -s demo --pubkey <TOKEN_ACCOUNT> --token-balance 1000000`}
      />

      <H3 id="preload">rustag preload [-s NAME] [PROGRAMS…]</H3>
      <p>
        Load known mainnet programs/oracles via the running stagenet. With no positional args it prints the
        available targets:
      </p>
      <CodeBlock
        lang="text"
        code={`jupiter  pyth  raydium  orca  marinade  spl-token  token-2022  metaplex`}
      />

      <H3 id="logs">rustag logs [-s NAME] [-f]</H3>
      <p>
        Tail the transaction feed. <code>-f, --follow</code> keeps streaming new transactions until{" "}
        <code>Ctrl-C</code>; without it, prints recent history and exits.
      </p>

      <H3 id="doctor">rustag doctor [-s NAME]</H3>
      <p>
        Preflight diagnostics: data dir writable, database openable, mainnet RPC reachable, and the
        stagenet&apos;s ports free (or held by the running stagenet). Exits non-zero on any hard failure.
      </p>

      {/* ---------------------------------------------------------- Phase 2 */}
      <H2 id="phase-2">
        Phase 2 <PhaseBadge phase={2} className="ml-1" />
      </H2>

      <H3 id="schedule">rustag schedule [-s NAME] &lt;SUBCOMMAND&gt;</H3>
      <p>Manage recurring on-chain activities to simulate realistic, ongoing usage.</p>
      <ul>
        <li>
          <code>add &lt;NAME&gt; &lt;SCHEDULE&gt;</code> — <code>&lt;SCHEDULE&gt;</code> is{" "}
          <code>@every 30s</code> / <code>@hourly</code> / a 5-field cron (<code>*/5 * * * *</code>). Action
          flags (pick one): <code>--airdrop &lt;PUBKEY&gt;</code>;{" "}
          <code>--transfer-from &lt;SECRET&gt; --to &lt;PUBKEY&gt;</code>; or{" "}
          <code>--raw-tx &lt;BASE64&gt;</code>. <code>--sol &lt;N&gt;</code> sets the amount (default{" "}
          <code>1.0</code>).
        </li>
        <li>
          <code>list</code> — list all activities with last-run status.
        </li>
        <li>
          <code>rm &lt;ID&gt;</code> — remove an activity by id.
        </li>
        <li>
          <code>toggle &lt;ID&gt; [--off]</code> — enable an activity, or <code>--off</code> to disable it.
        </li>
      </ul>
      <CodeBlock
        lang="bash"
        code={`rustag schedule -s demo add nightly "@every 30s" --airdrop <PUBKEY> --sol 5
rustag schedule -s demo add swap "*/5 * * * *" --raw-tx <BASE64_SIGNED_TX>
rustag schedule -s demo list
rustag schedule -s demo toggle <ID> --off
rustag schedule -s demo rm <ID>`}
      />

      <H3 id="metrics">rustag metrics [-s NAME]</H3>
      <p>Show analytics time-series for a stagenet.</p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          {
            name: "--series <S>",
            type: "all",
            desc: "Restrict to one series (e.g. tvl_lamports, transactions, accounts).",
          },
          { name: "--limit <N>", type: "20", desc: "Most-recent points to fetch per series." },
        ]}
      />

      {/* ---------------------------------------------------------- Phase 3 */}
      <H2 id="phase-3">
        Phase 3 <PhaseBadge phase={3} className="ml-1" />
      </H2>

      <H3 id="attest">rustag attest [-s NAME]</H3>
      <p>
        Produce a signed, Merkle-rooted attestation of staged state (operates offline against the persisted
        store).
      </p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          { name: "-o, --out <PATH>", type: ".rustag/<name>.attestation.json", desc: "Output file." },
          {
            name: "-k, --key <PATH>",
            type: ".rustag/attest-key.json",
            desc: "Attester keypair (Solana JSON); created if missing.",
          },
          { name: "-p, --program <ID>", type: "from txs", desc: "Program id(s) exercised. Repeatable." },
          { name: "--slot <N>", type: "tx count", desc: "Slot to record in the manifest." },
          { name: "--tx-limit <N>", type: "100000", desc: "Cap on transactions folded into the tx-results root." },
        ]}
      />

      <H3 id="verify">rustag verify &lt;FILE&gt; [-s NAME]</H3>
      <p>
        Verify an attestation offline; exits non-zero if INVALID. Positional <code>&lt;FILE&gt;</code> is
        the attestation JSON.
      </p>
      <ParamTable
        cols={["Flag", "Type", "Meaning"]}
        rows={[
          {
            name: "-s, --stagenet <NAME>",
            type: "from file",
            desc: "Recompute the state root against this stagenet's accounts.",
          },
          {
            name: "--signature-only",
            type: "flag",
            desc: "Only check the signature; skip recomputing the state root.",
          },
        ]}
      />

      <H3 id="scan">rustag scan [-s NAME]</H3>
      <p>Scan recorded transactions for exploit signatures — a CI gate.</p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          { name: "--limit <N>", type: "1000", desc: "Most-recent transactions to scan." },
          {
            name: "--fail-on <SEVERITY>",
            type: "off",
            desc: "Exit non-zero if any finding is ≥ this severity (info | low | medium | high | critical).",
          },
        ]}
      />

      <H3 id="tree">rustag tree --leaf &lt;X&gt;</H3>
      <p>
        Build an off-chain <code>spl-account-compression</code>-compatible concurrent Merkle tree (no
        stagenet/network needed) and print its root + proofs.
      </p>
      <ParamTable
        cols={["Flag", "Default", "Meaning"]}
        rows={[
          { name: "--depth <D>", type: "14", desc: "Tree depth (capacity is 2^depth)." },
          { name: "--buffer <N>", type: "64", desc: "Root-history / changelog window size." },
          {
            name: "--leaf <X>",
            type: "—",
            desc: "Leaf to append, in order. A 64-char hex string is a raw 32-byte node; anything else is keccak-256 hashed. Repeatable.",
          },
          { name: "--prove <I>", type: "—", desc: "Print an inclusion proof for leaf index I." },
        ]}
      />
      <CodeBlock
        lang="bash"
        code={`rustag attest -s demo --program <PROGRAM_ID>
rustag verify .rustag/demo.attestation.json -s demo
rustag scan -s demo --fail-on high
rustag tree --depth 14 --leaf hello --leaf world --prove 0`}
      />

      <Callout variant="info">
        Looking for the programmatic surface instead? See the{" "}
        <a href="/docs/sdk">SDK &amp; API reference</a> for <code>@rustag/sdk</code>, the REST contract, and
        the Solana JSON-RPC methods a stagenet implements.
      </Callout>
    </DocArticle>
  );
}
