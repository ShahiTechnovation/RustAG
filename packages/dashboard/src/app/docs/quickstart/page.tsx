import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";

export const metadata: Metadata = {
  title: "Quickstart",
  description:
    "From a built binary to a running mainnet-mirroring stagenet in about two minutes — install, configure the mirror, start a stagenet, and point any Solana client at it.",
};

const TOC: TocItem[] = [
  { id: "prerequisites", title: "Prerequisites" },
  { id: "build", title: "Build the CLI" },
  { id: "mirror", title: "Configure the mirror" },
  { id: "first-stagenet", title: "Your first stagenet" },
  { id: "drive", title: "Airdrop & inspect" },
  { id: "connect", title: "Connect your tooling" },
  { id: "dashboard", title: "SDK & dashboard" },
  { id: "limits", title: "Early-access limits" },
];

export default function QuickstartPage() {
  return (
    <DocArticle
      eyebrow="Get started"
      title="Quickstart"
      lead="Two minutes from a built binary to a running, mainnet-mirroring stagenet that any Solana client can talk to. RustAG ships as a single Rust binary you build from source — there is no published package yet."
      toc={TOC}
    >
      <H2 id="prerequisites">Prerequisites</H2>
      <ul>
        <li>
          <strong>Rust 1.96+</strong> — pinned in <code>rust-toolchain.toml</code>, so <code>rustup</code>{" "}
          selects the right toolchain automatically.
        </li>
        <li>
          <strong>Node 22+</strong> and <strong>pnpm 10+</strong> — only needed for the TypeScript SDK
          (<code>@rustag/sdk</code>) and the Next.js dashboard. The CLI and runtime do not require them.
        </li>
        <li>
          <strong>A mainnet RPC endpoint</strong> — the source the lazy mirror fetches real state from. The
          public Solana endpoint works but is heavily rate-limited; a free{" "}
          <a href="https://helius.dev" target="_blank" rel="noreferrer">
            Helius
          </a>{" "}
          or{" "}
          <a href="https://triton.one" target="_blank" rel="noreferrer">
            Triton
          </a>{" "}
          key is strongly recommended.
        </li>
      </ul>

      <CodeBlock
        lang="bash"
        filename="verify your toolchain"
        code={`rustc --version    # 1.96.0+
node --version     # v22+
pnpm --version     # 10+`}
      />

      <H2 id="build">Build the CLI</H2>
      <p>From the repository root, build the release binary:</p>
      <CodeBlock
        lang="bash"
        code={`git clone https://github.com/ShahiTechnovation/RustAG && cd RustAG
cargo build --release            # produces target/release/rustag`}
      />
      <p>
        On Windows the binary is <code>target/release/rustag.exe</code>. Put <code>target/release</code> on
        your <code>PATH</code> so you can call <code>rustag</code> directly:
      </p>
      <CodeBlock
        lang="bash"
        code={`# macOS/Linux
export PATH="$PWD/target/release:$PATH"
# Windows PowerShell
$env:PATH = "$PWD\\target\\release;$env:PATH"

rustag --version        # rustag 0.1.0
rustag --help           # lists all commands`}
      />

      <H2 id="mirror">Configure the mirror</H2>
      <p>
        Every stagenet reads its mirror endpoint from the <code>RUSTAG_MAINNET_RPC</code> environment
        variable (bound on the <code>create</code> command via clap&apos;s <code>env</code> attribute). Set
        it before creating stagenets:
      </p>
      <CodeBlock
        lang="bash"
        code={`cp .env.example .env.local      # then edit .env.local

# macOS/Linux
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
# Windows PowerShell
$env:RUSTAG_MAINNET_RPC = "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"`}
      />
      <p>
        In <code>RustAG.toml</code> the same value is referenced as{" "}
        <code>mirror.mainnet_rpc = &quot;${"{RUSTAG_MAINNET_RPC}"}&quot;</code>, so a committed project
        config and the environment stay in sync. If you skip it, RustAG falls back to a built-in default
        mainnet RPC — fine for a quick look, but rate-limited.
      </p>

      <H2 id="first-stagenet">Your first stagenet</H2>
      <p>
        <code>rustag start</code> is a long-running foreground process — run it in its own terminal.{" "}
        <code>--preload</code> loads real mainnet programs/oracles on startup. A stagenet&apos;s data lives
        in <code>./.rustag/db.sqlite</code> (project-local), so run these from your project directory.
      </p>
      <CodeBlock
        lang="bash"
        filename="terminal A"
        code={`rustag create demo
rustag start demo --preload pyth raydium`}
      />
      <p>On startup it prints the three endpoints:</p>
      <CodeBlock
        lang="text"
        code={`  ✓ Opened stagenet 'demo' (id: a1b2c3d4)
  ✓ Preloaded 4 accounts from mainnet
  ✓ RPC endpoint: http://127.0.0.1:8899
  ✓ WebSocket:    ws://127.0.0.1:8900
  ✓ REST API:     http://127.0.0.1:9000`}
      />
      <Callout variant="info" title="Long-running">
        <code>start</code> stays in the foreground. Press <code>Ctrl-C</code> to stop it, or run{" "}
        <code>rustag stop -s demo</code> from another terminal.
      </Callout>

      <H2 id="drive">Airdrop &amp; inspect</H2>
      <p>
        The client commands (<code>airdrop</code>, <code>override</code>, <code>preload</code>,{" "}
        <code>logs</code>, <code>status</code>) talk to the running stagenet&apos;s REST API, so run them
        from a second terminal while <code>start</code> runs. Airdrops are unlimited — no faucet caps.
      </p>
      <CodeBlock
        lang="bash"
        filename="terminal B"
        code={`rustag airdrop -s demo <YOUR_WALLET> 1000     # 1000 SOL, no faucet limit
rustag status  -s demo                        # counts, ports, running state
rustag logs    -s demo --follow               # live transaction feed`}
      />
      <p>
        You can read a <em>real</em> mainnet oracle straight through the stagenet&apos;s Solana RPC — the
        lazy mirror fetches and caches it on first read:
      </p>
      <CodeBlock
        lang="bash"
        code={`curl -s http://127.0.0.1:8899 -H 'content-type: application/json' \\
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE",{"encoding":"base64"}]}'`}
      />

      <H2 id="connect">Connect your tooling</H2>
      <p>
        The JSON-RPC endpoint at <code>http://127.0.0.1:8899</code> is a drop-in cluster URL. Change one
        line in your test setup:
      </p>
      <CodeBlock
        lang="bash"
        code={`ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor test
solana config set --url http://127.0.0.1:8899
solana balance <YOUR_WALLET> --url http://127.0.0.1:8899`}
      />
      <p>
        From <code>@solana/web3.js</code>, construct a <code>Connection</code> against the RPC URL;{" "}
        <code>connection.requestAirdrop</code> works like a validator faucet, but unlimited:
      </p>
      <CodeBlock
        lang="ts"
        code={`import { Connection } from "@solana/web3.js";

const connection = new Connection("http://127.0.0.1:8899");
await connection.requestAirdrop(pubkey, 2_000_000_000); // 2 SOL`}
      />

      <H2 id="dashboard">SDK &amp; dashboard</H2>
      <p>The TypeScript SDK and the Next.js dashboard only need the Node toolchain:</p>
      <CodeBlock
        lang="bash"
        code={`pnpm install
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
# open http://localhost:3000`}
      />

      <H2 id="limits">Early-access limits</H2>
      <Callout variant="early">
        Phase 1 — the working local MVP — runs <strong>your own</strong> deployed program against real
        mirrored mainnet state today. Executing an <em>arbitrary foreign</em> mainnet program end-to-end
        (e.g. a full Jupiter swap CPI) needs the BPF bytecode loading planned for Phase 2. WebSocket pub/sub
        is poll-based in Phase 1; sub-second push over <code>accountSubscribe</code> is a Phase 2{" "}
        <code>--features realtime</code> build.
      </Callout>
    </DocArticle>
  );
}
