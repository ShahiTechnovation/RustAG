import type { Metadata } from "next";
import { ShieldCheck, ArrowUpRight, Terminal } from "lucide-react";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Rehearse",
  description: "Rehearse a Squads proposal or raw transaction against faithful mainnet state and get a signed EvidenceBundle.",
};

const CLI_EXAMPLES = [
  {
    label: "Squads v4 proposal",
    cmd: "rustag rehearse --proposal <PROPOSAL_PUBKEY> --rpc $HELIUS_RPC",
  },
  {
    label: "Raw base64 transaction",
    cmd: "rustag rehearse --payload <BASE64_TX> --rpc $HELIUS_RPC",
  },
  {
    label: "Offline (no network)",
    cmd: "rustag rehearse --demo",
  },
  {
    label: "CI gate (fail on HIGH+)",
    cmd: "rustag rehearse --proposal <PUBKEY> --rpc $RPC --fail-on high",
  },
];

export default function RehearsePage() {
  return (
    <div className="mx-auto max-w-3xl space-y-10">
      {/* Header */}
      <div>
        <div className="flex items-center gap-3">
          <ShieldCheck className="text-brand" size={28} />
          <h1 className="font-display text-3xl font-bold tracking-tight text-fg">
            Rehearse
          </h1>
        </div>
        <p className="mt-3 max-w-xl text-base leading-relaxed text-muted">
          Rehearse a Squads v4 proposal or raw transaction against pinned mainnet state in a sealed
          sandbox. Returns a signed, offline-verifiable{" "}
          <span className="text-fg font-medium">EvidenceBundle</span> — semantic diff, invariant
          alarms, compute units, and SHA-256 state roots.
        </p>
      </div>

      {/* REST endpoint */}
      <div className="rounded-[3px] border border-border bg-surface/40 p-5 space-y-3">
        <p className="label text-brand">REST API · POST /api/rehearse</p>
        <pre className="overflow-x-auto rounded-[3px] bg-black/30 p-4 text-xs text-fg/80">
{`curl -X POST $RUSTAG_API_URL/api/rehearse \\
  -H "Content-Type: application/json" \\
  -d '{
    "payload_b64": "<base64 VersionedTransaction>",
    "mainnet_rpc": "https://mainnet.helius-rpc.com/?api-key=XXX",
    "policy_rules": ["upgrade-authority", "large-sol-drain"]
  }'`}
        </pre>
        <p className="text-xs text-faint">
          The backend runs on Render. Set{" "}
          <code className="rounded bg-white/5 px-1 py-0.5 font-mono text-[11px]">NEXT_PUBLIC_RUSTAG_API_URL</code>{" "}
          in your Vercel env vars to point at it.
        </p>
      </div>

      {/* CLI examples */}
      <div className="space-y-3">
        <p className="label text-muted">CLI · rustag rehearse</p>
        <div className="space-y-2">
          {CLI_EXAMPLES.map((ex) => (
            <div key={ex.label} className="rounded-[3px] border border-border bg-surface/40 p-4">
              <p className="mb-2 text-xs text-faint">{ex.label}</p>
              <pre className="overflow-x-auto text-xs text-brand">{ex.cmd}</pre>
            </div>
          ))}
        </div>
      </div>

      {/* Docs link */}
      <Link
        href="/docs"
        className="inline-flex items-center gap-1.5 text-sm text-brand hover:text-brand-strong transition-colors"
      >
        Read the full EvidenceBundle spec
        <ArrowUpRight size={15} />
      </Link>
    </div>
  );
}
