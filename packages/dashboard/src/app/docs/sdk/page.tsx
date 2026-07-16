import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2, H3 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";
import { PhaseBadge } from "@/components/docs/PhaseBadge";

export const metadata: Metadata = {
  title: "SDK & API",
  description:
    "The @rustag/sdk TypeScript client, POST /api/rehearse and POST /api/verify REST endpoints, and the Solana JSON-RPC surface used during closure resolution.",
};

const TOC: TocItem[] = [
  { id: "typescript-sdk", title: "TypeScript SDK" },
  { id: "construction", title: "Construction", depth: 3 },
  { id: "rehearse-verify", title: "Rehearse & verify", depth: 3 },
  { id: "other-methods", title: "Other methods", depth: 3 },
  { id: "rest-api", title: "REST API" },
  { id: "rest-core", title: "Core endpoints", depth: 3 },
  { id: "rest-stagenet", title: "Stagenet dev endpoints", depth: 3 },
  { id: "rpc", title: "JSON-RPC (closure resolution)" },
  { id: "websocket", title: "WebSocket subscriptions", depth: 3 },
];

// Core GroundTruth REST endpoints
const REST_CORE: [string, string, string][] = [
  ["GET /api/health", "—", "{ status: \"ok\", version }"],
  [
    "POST /api/rehearse",
    "{ proposal?, payload?, rpc?, demo?, failOn? }",
    "EvidenceBundle JSON — signed, with pre/post state roots, semantic diff, alarms, grade",
  ],
  [
    "POST /api/verify",
    "{ bundle, closure }",
    "{ valid: bool, grade, alarms, signer } — offline verification, no RPC needed",
  ],
  [
    "POST /api/forensics",
    "{ signature, rpc, patch?, patchProgram? }",
    "{ verdict: 'BLOCKED' | 'REPRODUCED', diff, logs }",
  ],
];

// Stagenet dev / dashboard endpoints
const REST_STAGENET: [string, string, string][] = [
  [
    "GET /api/stagenet",
    "—",
    "id, name, network, slot, rpcUrl, wsUrl, mirrorEnabled, accounts, transactions, dirtyAccounts",
  ],
  ["GET /api/accounts", "?limit (1–1000, def 100) &offset", "{ accounts: [...] }, newest-touched first"],
  [
    "GET /api/accounts/{pubkey}",
    "—",
    "one account (lazily mirrored from mainnet); 404 if missing",
  ],
  ["GET /api/transactions", "?limit (1–500, def 50)", "{ transactions: [...] }, newest first"],
  ["POST /api/airdrop", "{ pubkey, sol }", "{ signature, lamports }"],
  ["POST /api/override", "{ pubkey, lamports?, tokenBalance? }", "{ ok: true }"],
  ["POST /api/preload", "{ programs: [...] }", "{ loaded, unknown }"],
  ["GET /api/schedules", "—", "{ schedules: [...] }"],
  ["POST /api/schedules", "{ name, schedule, action }", "the created Schedule"],
  ["DELETE /api/schedules/{id}", "—", "{ ok: <removed> }"],
  ["POST /api/schedules/{id}/toggle", "{ enabled }", "{ ok: true, enabled }"],
  ["GET /api/metrics", "?series &limit (1–10000, def 500)", "{ metrics: { <series>: [{ t, v }] } }"],
  ["POST /api/simulate", "{ transactions, label?, encoding? }", "a ScenarioReport (≤5000 txs)"],
];

const RPC_ROWS: [string, string][] = [
  ["getHealth", "Returns \"ok\"."],
  ["getVersion", "{ \"solana-core\": \"2.1.0\", \"feature-set\": 0 }."],
  ["getGenesisHash / getIdentity", "Fixed stagenet values."],
  ["getSlot / getBlockHeight", "Monotonic slot (advances per transaction)."],
  ["getEpochInfo", "Slot-derived (slotsInEpoch = 432000)."],
  ["getLatestBlockhash", "{ blockhash, lastValidBlockHeight } (slot + 150)."],
  ["isBlockhashValid", "Always true — the stagenet blockhash never expires."],
  ["getMinimumBalanceForRentExemption", "[dataLen] → lamports."],
  ["getBalance", "[pubkey] → { context, value }; lazily mirrors from mainnet."],
  ["getAccountInfo", "[pubkey, {encoding}] → base64 account or null; lazily mirrors."],
  ["getMultipleAccounts", "[[pubkey,…], {encoding}] — used by the ingest layer during closure resolution."],
  ["getProgramAccounts", "[programId, {filters}] → owned accounts; honors dataSize & memcmp (≤10000)."],
  ["getTokenAccountBalance", "[tokenAccount] → SPL amount."],
  ["requestAirdrop", "[pubkey, lamports] → signature."],
  ["sendTransaction", "[txBlob, {encoding}] → signature; base58 default, base64 fallback."],
  ["simulateTransaction", "[txBlob, {encoding}] → { err, logs, unitsConsumed, returnData }."],
  ["getSignatureStatuses", "[[sig,…]] → statuses (confirmationStatus \"finalized\")."],
  ["getTransaction", "[signature] → indexed meta (fee, computeUnitsConsumed, logMessages)."],
  ["getFeeForMessage", "Fixed 5000-lamport fee."],
];

function RestTable({ rows }: { rows: [string, string, string][] }) {
  return (
    <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-border bg-white/[0.015]">
            {["Method & path", "Body / query", "Returns"].map((c) => (
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
              <td className="whitespace-nowrap px-4 py-3 font-mono text-[12px] text-brand">{r[0]}</td>
              <td className="px-4 py-3 font-mono text-[11.5px] text-accent-2">{r[1]}</td>
              <td className="px-4 py-3 text-[13px] leading-relaxed text-muted">{r[2]}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function SdkPage() {
  return (
    <DocArticle
      eyebrow="Reference"
      title="SDK & API"
      lead="Three ways to drive RustAG: the @rustag/sdk TypeScript client over REST, the raw REST contract the dashboard sits on, and the Solana-compatible JSON-RPC used during closure resolution."
      toc={TOC}
    >
      <H2 id="typescript-sdk">TypeScript SDK</H2>
      <p>
        The <code>@rustag/sdk</code> package exposes a single client class,{" "}
        <code>RustagClient</code>, that wraps the running REST API (default base{" "}
        <code>http://localhost:9000</code>). The primary surface is{" "}
        <code>rehearse()</code> and <code>verify()</code>.
      </p>

      <H3 id="construction">Construction</H3>
      <CodeBlock
        lang="ts"
        code={`import { RustagClient } from "@rustag/sdk";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });
// or against the hosted service:
const client = new RustagClient({
  baseUrl: "https://api.rustag.dev",
  apiKey: process.env.RUSTAG_API_KEY,
});`}
      />
      <p>
        <code>RustagClientOptions</code>: <code>baseUrl</code> (default{" "}
        <code>http://localhost:9000</code>; trailing slashes stripped), <code>apiKey</code>{" "}
        (optional <code>Bearer rk_…</code> for the hosted service), and <code>fetch</code> (a
        custom fetch implementation for Node runtimes without global <code>fetch</code>).
      </p>

      <H3 id="rehearse-verify">Rehearse & verify</H3>
      <p>
        These are the two primary methods — everything else is dashboard/stagenet tooling.
      </p>
      <CodeBlock
        lang="ts"
        code={`// Rehearse a Squads v4 proposal
const bundle = await client.rehearse({
  proposal: "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  rpc: "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY",
  failOn: "high",   // throws if any HIGH/CRITICAL alarm fires
});

console.log(bundle.grade);              // "A"
console.log(bundle.alarms);            // [{ rule, severity, message }]
console.log(bundle.semanticDiff);      // [{ type, ...fields }]
console.log(bundle.preStateRoot);      // hex SHA-256 Merkle root
console.log(bundle.postStateRoot);
console.log(bundle.signerPubkey);      // attester Ed25519 pubkey

// Rehearse a raw transaction
const bundle2 = await client.rehearse({
  payload: "<BASE64_TX>",
  rpc: process.env.MAINNET_RPC,
});

// Built-in demo (no RPC needed)
const demo = await client.rehearse({ demo: true });

// Verify a bundle offline (no network)
const report = await client.verify({
  bundle: bundle,          // EvidenceBundle object or JSON string
  closure: closureJson,    // portable closure JSON string
});
console.log(report.valid, report.grade); // true, "A"`}
      />

      <H3 id="other-methods">Other methods</H3>
      <p>
        <strong>Stagenet & dashboard methods</strong> — these target the persistent stagenet
        dev server (<code>rustag serve</code> or <code>rustag start</code>):
      </p>
      <ul>
        <li>
          <code>health()</code> → <code>{"{ status, version }"}</code>
        </li>
        <li>
          <code>getStagenet()</code> → <code>StagenetInfo</code> — id, name, network, slot,
          rpcUrl, wsUrl, accounts, transactions.
        </li>
        <li>
          <code>listAccounts({"{ limit?, offset? }"})</code> → <code>AccountInfo[]</code>
        </li>
        <li>
          <code>getAccount(pubkey)</code> → <code>AccountInfo</code> — lazily mirrored from
          mainnet if not local.
        </li>
        <li>
          <code>listTransactions({"{ limit? }"})</code> → <code>TransactionInfo[]</code>
        </li>
        <li>
          <code>airdrop(pubkey, sol)</code> → <code>{"{ signature, lamports }"}</code>
        </li>
        <li>
          <code>overrideAccount(params)</code> → <code>{"{ ok }"}</code>
        </li>
        <li>
          <code>preload(programs)</code> → <code>{"{ loaded, unknown }"}</code>
        </li>
      </ul>
      <p>
        <strong>Phase 2</strong> <PhaseBadge phase={2} className="ml-1" /> — scheduler /
        analytics / simulation:
      </p>
      <ul>
        <li>
          <code>listSchedules()</code>, <code>createSchedule(params)</code>,{" "}
          <code>deleteSchedule(id)</code>, <code>toggleSchedule(id, enabled)</code>
        </li>
        <li>
          <code>getMetrics({"{ series?, limit? }"})</code> → analytics time-series, each point{" "}
          <code>{"{ t, v }"}</code>
        </li>
        <li>
          <code>simulate(transactions, {"{ label?, encoding? }"})</code> →{" "}
          <code>ScenarioReport</code>
        </li>
      </ul>
      <CodeBlock
        lang="ts"
        code={`// Full GroundTruth CI workflow in TypeScript
import { RustagClient } from "@rustag/sdk";

const client = new RustagClient({ baseUrl: process.env.RUSTAG_API_URL });

// Rehearse and gate on severity
const bundle = await client.rehearse({
  proposal: process.env.PROPOSAL_PUBKEY,
  rpc: process.env.MAINNET_RPC,
  failOn: "high",
});

// Write the bundle + closure for archiving
await fs.writeFile("bundle.json", JSON.stringify(bundle, null, 2));

// Offline verify (zero network)
const report = await client.verify({ bundle, closure: closureJson });
if (!report.valid) process.exit(1);`}
      />
      <Callout variant="info">
        Non-2xx responses are thrown as <code>Error</code> of the form{" "}
        <code>{"RustAG API <status> <statusText>: <body>"}</code>. The{" "}
        <code>failOn</code> option causes <code>rehearse()</code> to throw before returning if
        any alarm meets or exceeds the given severity.
      </Callout>

      <H2 id="rest-api">REST API</H2>

      <H3 id="rest-core">Core GroundTruth endpoints</H3>
      <p>
        These endpoints are served on <code>http://127.0.0.1:9000</code> by default (or{" "}
        <code>$PORT</code> on Render). CORS is permissive.
      </p>
      <RestTable rows={REST_CORE} />
      <CodeBlock
        lang="bash"
        filename="rehearse via REST"
        code={`# Rehearse a Squads v4 proposal
curl -s http://127.0.0.1:9000/api/rehearse \\
  -H 'content-type: application/json' \\
  -d '{
    "proposal": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
    "rpc": "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
  }' | jq '{grade, alarms: .alarms | length}'

# Verify a bundle offline
curl -s http://127.0.0.1:9000/api/verify \\
  -H 'content-type: application/json' \\
  -d '{"bundle": <BUNDLE_JSON>, "closure": <CLOSURE_JSON>}'`}
      />

      <H3 id="rest-stagenet">Stagenet dev endpoints</H3>
      <p>
        Used by the dashboard and the stagenet development surface. Require a running stagenet
        (<code>rustag serve</code> or <code>rustag start</code>):
      </p>
      <RestTable rows={REST_STAGENET} />

      <H2 id="rpc">JSON-RPC (closure resolution)</H2>
      <p>
        The stagenet speaks a Solana-compatible JSON-RPC dialect on{" "}
        <code>http://127.0.0.1:8899</code>. This is primarily used by the ingest layer during
        closure resolution (<code>getMultipleAccounts</code>) — but you can also point any{" "}
        <code>@solana/web3.js</code> <code>Connection</code> at it for integration testing.
        Both single requests and batch arrays are supported.
      </p>

      <div className="my-6 overflow-x-auto rounded-[4px] border border-border">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border bg-white/[0.015]">
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                Method
              </th>
              <th className="px-4 py-2.5 text-left font-mono text-[11px] uppercase tracking-[0.12em] text-faint">
                Notes
              </th>
            </tr>
          </thead>
          <tbody>
            {RPC_ROWS.map((r) => (
              <tr key={r[0]} className="border-b border-border/60 align-top last:border-0">
                <td className="whitespace-nowrap px-4 py-2.5 font-mono text-[12px] text-brand">{r[0]}</td>
                <td className="px-4 py-2.5 text-[13px] leading-relaxed text-muted">{r[1]}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p>
        Any method outside this list returns JSON-RPC error <code>-32601</code>{" "}
        (&ldquo;method not found&rdquo;). Encoded transactions are decoded as base58 first,
        falling back to base64, unless an explicit <code>encoding</code> is supplied.
      </p>
      <CodeBlock
        lang="ts"
        code={`import { Connection } from "@solana/web3.js";

// Point at the stagenet RPC for integration testing
const connection = new Connection("http://127.0.0.1:8899");
const balance = await connection.getBalance(pubkey);
await connection.requestAirdrop(pubkey, 2_000_000_000); // 2 SOL, no faucet limit`}
      />

      <H3 id="websocket">WebSocket subscriptions</H3>
      <p>
        The WebSocket server listens on <code>ws://127.0.0.1:8900</code>. In Phase 1
        subscriptions are poll-based (≈1s interval):
      </p>
      <ul>
        <li>
          <code>accountSubscribe</code> → returns a subscription id, then pushes an{" "}
          <code>accountNotification</code> whenever the account&apos;s{" "}
          <code>(lamports, data length)</code> fingerprint changes.
        </li>
        <li>
          <code>signatureSubscribe</code> → one-shot; fires once the transaction is found, then
          auto-cancels. This is what <code>@solana/web3.js</code> uses, so{" "}
          <code>sendAndConfirmTransaction</code> / <code>confirmTransaction</code> work out of
          the box.
        </li>
        <li>
          <code>slotSubscribe</code> → accepted for compatibility; no slot stream is emitted.
        </li>
        <li>
          <code>accountUnsubscribe</code> / <code>signatureUnsubscribe</code> /{" "}
          <code>slotUnsubscribe</code> → cancel a subscription.
        </li>
      </ul>
      <Callout variant="early" title="Realtime push is Phase 2">
        An optional <code>realtime</code> Cargo feature adds a server-side push mirror: when
        enabled (and a <code>realtime_ws</code> upstream is configured) the server subscribes
        to the oracle registry over <code>accountSubscribe</code> upstream with a reconnect
        loop, replacing the poll. Build with <code>--features realtime</code>.
      </Callout>
    </DocArticle>
  );
}
