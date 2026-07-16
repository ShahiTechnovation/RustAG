import type { Metadata } from "next";

import { Callout } from "@/components/docs/Callout";
import { CodeBlock } from "@/components/docs/CodeBlock";
import { DocArticle } from "@/components/docs/DocArticle";
import { H2 } from "@/components/docs/Heading";
import type { TocItem } from "@/components/docs/OnThisPage";

export const metadata: Metadata = {
  title: "Quickstart",
  description:
    "From install to a signed EvidenceBundle in under two minutes — build the CLI, run the built-in demo, then rehearse a real Squads proposal from mainnet.",
};

const TOC: TocItem[] = [
  { id: "prerequisites", title: "Prerequisites" },
  { id: "build", title: "Build the CLI" },
  { id: "demo", title: "Run the built-in demo" },
  { id: "rehearse", title: "Rehearse a real proposal" },
  { id: "verify", title: "Verify the bundle offline" },
  { id: "forensics", title: "Forensics mode" },
  { id: "dashboard", title: "Dashboard & API" },
  { id: "limits", title: "Early-access limits" },
];

export default function QuickstartPage() {
  return (
    <DocArticle
      eyebrow="Get started"
      title="Quickstart"
      lead="Two minutes from a built binary to a signed EvidenceBundle. RustAG ships as a single Rust binary you build from source — there is no published package yet."
      toc={TOC}
    >
      <H2 id="prerequisites">Prerequisites</H2>
      <ul>
        <li>
          <strong>Rust 1.96+</strong> — pinned in <code>rust-toolchain.toml</code>, so{" "}
          <code>rustup</code> selects the right toolchain automatically.
        </li>
        <li>
          <strong>A mainnet RPC endpoint</strong> — the source the closure resolver fetches real
          state from. A free{" "}
          <a href="https://helius.dev" target="_blank" rel="noreferrer">
            Helius
          </a>{" "}
          or{" "}
          <a href="https://triton.one" target="_blank" rel="noreferrer">
            Triton
          </a>{" "}
          key is strongly recommended. The built-in demo works without one.
        </li>
        <li>
          <strong>Node 22+ / pnpm 10+</strong> — only needed for the TypeScript SDK and Next.js
          dashboard. The CLI does not require them.
        </li>
      </ul>

      <H2 id="build">Build the CLI</H2>
      <CodeBlock
        lang="bash"
        code={`git clone https://github.com/ShahiTechnovation/RustAG && cd RustAG
cargo build --release            # produces target/release/rustag

# Add to PATH
export PATH="$PWD/target/release:$PATH"   # macOS/Linux
$env:PATH = "$PWD\\target\\release;$env:PATH"  # Windows PowerShell

rustag --version    # rustag 0.1.0
rustag --help       # lists all subcommands`}
      />

      <H2 id="demo">Run the built-in demo</H2>
      <p>
        The <code>--demo</code> flag runs a self-contained ownership-takeover payload — no RPC
        key, no network. It&apos;s the fastest way to see a complete signed EvidenceBundle end-to-end.
      </p>
      <CodeBlock
        lang="bash"
        code={`rustag rehearse --demo

# Output:
#  ✓ Rehearsal complete · Grade A
#  ✓ Semantic diff:
#    - UpgradeAuthority: 7xKX...→ 3mPQ... (ROTATED)
#  ✓ Alarms: 1 CRITICAL (upgrade-authority)
#  ✓ Bundle written → groundtruth-bundle.json
#  ✓ Closure written → groundtruth-closure.json
#  ✓ Signed by: ephemeral key (pubkey printed above)`}
      />
      <Callout variant="info" title="Ephemeral key">
        If no <code>--signer</code> is provided, an ephemeral Ed25519 keypair is generated and
        its pubkey printed. Pass <code>--signer ./my-key.json</code> to sign with your own key.
      </Callout>

      <H2 id="rehearse">Rehearse a real proposal</H2>
      <p>
        Point RustAG at a live Squads v4 VaultTransaction proposal address. It fetches, decodes,
        and rehearses the proposal against current mainnet state.
      </p>
      <CodeBlock
        lang="bash"
        code={`export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"

rustag rehearse \\
  --proposal 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU \\
  --rpc $RUSTAG_MAINNET_RPC \\
  --out bundle.json \\
  --closure closure.json

# Or rehearse a raw transaction directly:
rustag rehearse --payload <BASE64_TX> --rpc $RUSTAG_MAINNET_RPC`}
      />
      <p>
        Use <code>--fail-on high</code> to make the command exit non-zero if any alarm reaches
        HIGH or above — useful as a CI gate in your multisig approval workflow.
      </p>
      <CodeBlock
        lang="bash"
        code={`rustag rehearse \\
  --proposal <PUBKEY> \\
  --rpc $RPC \\
  --fail-on high      # exit 1 if any HIGH/CRITICAL alarm fires`}
      />

      <H2 id="verify">Verify the bundle offline</H2>
      <p>
        Anyone can re-verify an EvidenceBundle independently — no RPC call, no trust in the
        rehearser. The verifier re-derives the state roots from the closure and checks the
        Ed25519 signature.
      </p>
      <CodeBlock
        lang="bash"
        code={`rustag verify bundle.json --closure closure.json

# Output:
#  ✓ Signature valid
#  ✓ pre_state_root matches closure
#  ✓ Grade A (deterministically re-executable)
#  Signer: <PUBKEY>`}
      />

      <H2 id="forensics">Forensics mode</H2>
      <p>
        Re-execute any historical mainnet transaction by signature. In counterfactual mode,
        substitute the deployed program with a patched ELF to answer: &quot;would this fix have
        stopped the attack?&quot;
      </p>
      <CodeBlock
        lang="bash"
        code={`# Re-execute a historical transaction
rustag forensics <SIGNATURE> --rpc $RPC

# Counterfactual: substitute a patched program ELF
rustag forensics <SIGNATURE> \\
  --rpc $RPC \\
  --patch ./patched-program.so \\
  --patch-program <PROGRAM_ID>

# Output: BLOCKED ✓  or  REPRODUCED ✗`}
      />

      <H2 id="dashboard">Dashboard & API</H2>
      <CodeBlock
        lang="bash"
        code={`# Start the REST + RPC backend
export RUSTAG_MAINNET_RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
rustag serve

# Start the Next.js dashboard (separate terminal)
pnpm install
NEXT_PUBLIC_RUSTAG_API_URL=http://localhost:9000 pnpm --filter dashboard dev
# open http://localhost:3000`}
      />
      <p>
        The REST API exposes <code>POST /api/rehearse</code> (submit a payload, get a signed
        EvidenceBundle) and <code>POST /api/verify</code> (verify a bundle offline). See the{" "}
        <a href="/docs/sdk">SDK & API reference</a> for the full contract.
      </p>

      <H2 id="limits">Early-access limits</H2>
      <Callout variant="early">
        The CLI and REST API are the working Phase 1 deliverable. The Squads web UI integration
        (signer-review embed), Yellowstone gRPC real-time recording, hosted Evidence Registry,
        and per-flow pricing are Phases 2–5 and are not yet released. Everything described on this
        page works today — build from source and run locally.
      </Callout>
    </DocArticle>
  );
}
