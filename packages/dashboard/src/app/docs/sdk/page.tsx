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
    "The @rustag/sdk TypeScript client, the REST API contract, and the Solana JSON-RPC + WebSocket methods a stagenet implements.",
};

const TOC: TocItem[] = [
  { id: "typescript-sdk", title: "TypeScript SDK" },
  { id: "construction", title: "Construction", depth: 3 },
  { id: "methods", title: "Client methods", depth: 3 },
  { id: "rest-api", title: "REST API" },
  { id: "rpc", title: "JSON-RPC compatibility" },
  { id: "websocket", title: "WebSocket subscriptions", depth: 3 },
];

const REST_ROWS: [string, string, string][] = [
  ["GET /api/health", "—", "{ status: \"ok\" }"],
  [
    "GET /api/stagenet",
    "—",
    "id, name, network, slot, rpcUrl, wsUrl, mirrorEnabled, mainnetRpc, accounts, transactions, dirtyAccounts",
  ],
  ["GET /api/accounts", "?limit (1–1000, def 100) &offset", "{ accounts: [...] }, newest-touched first"],
  [
    "GET /api/accounts/{pubkey}",
    "—",
    "one account (lazily mirrored); 404 if missing, 400 on invalid pubkey",
  ],
  ["GET /api/transactions", "?limit (1–500, def 50)", "{ transactions: [...] }, newest first"],
  ["POST /api/airdrop", "{ pubkey, sol }", "{ signature, lamports }"],
  ["POST /api/override", "{ pubkey, lamports?, tokenBalance? }", "{ ok: true }"],
  ["POST /api/preload", "{ programs: [...] }", "{ loaded, unknown }"],
];

const REST_ROWS_P2: [string, string, string][] = [
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
  ["getBalance", "[pubkey] → { context, value }; lazily mirrors."],
  ["getAccountInfo", "[pubkey, {encoding}] → base64 account or null; lazily mirrors."],
  ["getMultipleAccounts", "[[pubkey,…], {encoding}]."],
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
      lead="Three ways to drive a stagenet: the @rustag/sdk TypeScript client over REST, the raw REST contract the dashboard and SDK sit on, and the Solana-compatible JSON-RPC your existing tooling already speaks."
      toc={TOC}
    >
      <H2 id="typescript-sdk">TypeScript SDK</H2>
      <p>
        The <code>@rustag/sdk</code> package exposes a single client class, <code>RustagClient</code>, that
        wraps a running stagenet&apos;s REST API (default base <code>http://localhost:9000</code>). It
        targets the REST surface, not the Solana JSON-RPC port — for transactions you point a{" "}
        <code>@solana/web3.js</code> <code>Connection</code> at the stagenet&apos;s RPC URL instead.
      </p>

      <H3 id="construction">Construction</H3>
      <CodeBlock
        lang="ts"
        code={`import { RustagClient } from "@rustag/sdk";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });`}
      />
      <p>
        <code>RustagClientOptions</code> has two optional fields: <code>baseUrl</code> (default{" "}
        <code>http://localhost:9000</code>; trailing slashes are stripped) and <code>fetch</code> (a custom{" "}
        <code>fetch</code> implementation, e.g. for Node runtimes without a global <code>fetch</code>). The
        default global <code>fetch</code> is bound to <code>globalThis</code> to avoid the browser&apos;s
        &ldquo;Illegal invocation&rdquo; error; if no <code>fetch</code> is available, the constructor
        throws. <code>getStagenet()</code> returns the stagenet&apos;s <code>rpcUrl</code>, which you can
        hand straight to <code>@solana/web3.js</code>:
      </p>
      <CodeBlock
        lang="ts"
        code={`const stagenet = await client.getStagenet();
await client.airdrop(wallet, 1000);

import { Connection } from "@solana/web3.js";
const connection = new Connection(stagenet.rpcUrl); // http://127.0.0.1:8899`}
      />

      <H3 id="methods">Client methods</H3>
      <p>
        <strong>Phase 1</strong> — available in the local MVP:
      </p>
      <ul>
        <li>
          <code>health()</code> → <code>{"{ status }"}</code> — liveness check.
        </li>
        <li>
          <code>getStagenet()</code> → <code>StagenetInfo</code> — id, name, network, slot, rpcUrl, wsUrl,
          mirrorEnabled, mainnetRpc, and account/transaction/dirty counts.
        </li>
        <li>
          <code>listAccounts({"{ limit?, offset? }"})</code> → <code>AccountInfo[]</code> — newest-touched
          first.
        </li>
        <li>
          <code>getAccount(pubkey)</code> → <code>AccountInfo</code> — lazily mirrored from mainnet if not
          local.
        </li>
        <li>
          <code>listTransactions({"{ limit? }"})</code> → <code>TransactionInfo[]</code>.
        </li>
        <li>
          <code>airdrop(pubkey, sol)</code> → <code>{"{ signature, lamports }"}</code> — unlimited, instant,
          free.
        </li>
        <li>
          <code>overrideAccount(params)</code> → <code>{"{ ok }"}</code> — set <code>lamports</code> and/or
          raw SPL <code>tokenBalance</code>.
        </li>
        <li>
          <code>preload(programs)</code> → <code>{"{ loaded, unknown }"}</code>.
        </li>
      </ul>
      <p>
        <strong>Phase 2</strong> <PhaseBadge phase={2} className="ml-1" /> — depends on the corresponding
        background workers being enabled on the server:
      </p>
      <ul>
        <li>
          <code>listSchedules()</code>, <code>createSchedule(params)</code>,{" "}
          <code>deleteSchedule(id)</code>, <code>toggleSchedule(id, enabled)</code>.
        </li>
        <li>
          <code>getMetrics({"{ series?, limit? }"})</code> → analytics time-series, each point{" "}
          <code>{"{ t, v }"}</code>.
        </li>
        <li>
          <code>simulate(transactions, {"{ label?, encoding? }"})</code> → <code>ScenarioReport</code> —
          replay signed transactions against an isolated fork (the base is never mutated).
        </li>
      </ul>
      <CodeBlock
        lang="ts"
        code={`await client.getStagenet();
await client.listAccounts({ limit: 100 });
await client.getAccount("<PUBKEY>");
await client.airdrop("<PUBKEY>", 1000);
await client.overrideAccount({ pubkey: "<PUBKEY>", lamports: 5_000_000_000 });
await client.preload(["jupiter", "pyth", "raydium"]);

const report = await client.simulate([signedTxBase64], { label: "swap-scenario" });
console.log(report.succeeded, "/", report.total, "ok in", report.durationMs, "ms");`}
      />
      <Callout variant="info">
        Non-2xx responses are turned into a thrown <code>Error</code> of the form{" "}
        <code>RustAG API &lt;status&gt; &lt;statusText&gt;: &lt;body&gt;</code>. A transfer schedule&apos;s{" "}
        <code>secret_key</code> is redacted to <code>***redacted***</code> on read.
      </Callout>

      <H2 id="rest-api">REST API</H2>
      <p>
        The REST API is an <code>axum</code> router mounted under <code>/api</code>, served on{" "}
        <code>http://127.0.0.1:9000</code> by default. It is the surface consumed by the dashboard and the
        SDK; CORS is permissive.
      </p>
      <RestTable rows={REST_ROWS} />
      <p>
        <strong>Phase 2</strong> <PhaseBadge phase={2} className="ml-1" /> endpoints — require the scheduler
        / metrics / simulation workers to be enabled:
      </p>
      <RestTable rows={REST_ROWS_P2} />
      <CodeBlock
        lang="bash"
        filename="airdrop via REST"
        code={`curl -s http://127.0.0.1:9000/api/airdrop \\
  -H 'content-type: application/json' \\
  -d '{"pubkey":"<PUBKEY>","sol":1000}'
# => { "signature": "...", "lamports": 1000000000000 }`}
      />

      <H2 id="rpc">JSON-RPC compatibility</H2>
      <p>
        A stagenet speaks a Solana-compatible JSON-RPC dialect, so a wallet or a <code>@solana/web3.js</code>{" "}
        <code>Connection</code> can point at it and just work. The RPC server listens on{" "}
        <code>http://127.0.0.1:8899</code> and accepts a JSON-RPC 2.0 body at <code>POST /</code>; both
        single requests and batch arrays are supported. The advertised version is{" "}
        <code>{"{ solana-core: 2.1.0, feature-set: 0 }"}</code>.
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
        Any method outside this list returns JSON-RPC error <code>-32601</code> (&ldquo;method not
        found&rdquo;). Encoded transactions are decoded as base58 first, falling back to base64, unless an
        explicit <code>encoding</code> is supplied.
      </p>
      <CodeBlock
        lang="ts"
        code={`import { Connection } from "@solana/web3.js";

const connection = new Connection("http://127.0.0.1:8899");
const balance = await connection.getBalance(pubkey);
await connection.requestAirdrop(pubkey, 2_000_000_000);`}
      />

      <H3 id="websocket">WebSocket subscriptions</H3>
      <p>
        The WebSocket server listens on <code>ws://127.0.0.1:8900</code>. In Phase 1 subscriptions are
        poll-based (≈1s interval):
      </p>
      <ul>
        <li>
          <code>accountSubscribe</code> → returns a subscription id, then pushes an{" "}
          <code>accountNotification</code> whenever the account&apos;s <code>(lamports, data length)</code>{" "}
          fingerprint changes.
        </li>
        <li>
          <code>signatureSubscribe</code> → one-shot; fires once the transaction is found, then
          auto-cancels. This is what <code>@solana/web3.js</code> uses, so{" "}
          <code>sendAndConfirmTransaction</code> / <code>confirmTransaction</code> work out of the box.
        </li>
        <li>
          <code>slotSubscribe</code> → accepted for compatibility, but no slot stream is emitted.
        </li>
        <li>
          <code>accountUnsubscribe</code> / <code>signatureUnsubscribe</code> / <code>slotUnsubscribe</code>{" "}
          → cancel a subscription.
        </li>
      </ul>
      <Callout variant="early" title="Realtime push is Phase 2">
        An optional <code>realtime</code> feature adds a server-side push mirror: when enabled (and a{" "}
        <code>realtime_ws</code> upstream is configured) the server subscribes to the oracle registry over{" "}
        <code>accountSubscribe</code> upstream with a reconnect loop, replacing the poll. Build with{" "}
        <code>--features realtime</code>.
      </Callout>
    </DocArticle>
  );
}
