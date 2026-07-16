import type { Metadata } from "next";
import { FileSearch, ArrowUpRight } from "lucide-react";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Forensics",
  description: "Re-execute historical Solana transactions and run counterfactual patch analysis.",
};

const CLI_EXAMPLES = [
  {
    label: "Re-execute a historical transaction",
    cmd: "rustag forensics <SIGNATURE> --rpc $HELIUS_RPC",
  },
  {
    label: "Counterfactual: would this fix have stopped it?",
    cmd: `rustag forensics <SIGNATURE> \\
  --rpc $RPC \\
  --patch ./patched-program.so \\
  --patch-program <PROGRAM_ID>`,
  },
  {
    label: "Machine-readable JSON output",
    cmd: "rustag forensics <SIGNATURE> --rpc $RPC --json",
  },
];

export default function ForensicsPage() {
  return (
    <div className="mx-auto max-w-3xl space-y-10">
      {/* Header */}
      <div>
        <div className="flex items-center gap-3">
          <FileSearch className="text-brand" size={28} />
          <h1 className="font-display text-3xl font-bold tracking-tight text-fg">
            Forensics
          </h1>
        </div>
        <p className="mt-3 max-w-xl text-base leading-relaxed text-muted">
          Re-execute any historical mainnet transaction deterministically. In{" "}
          <span className="text-fg font-medium">counterfactual mode</span>, substitute the deployed
          program with a patched ELF and find out if it would have{" "}
          <span className="text-green-400 font-medium">BLOCKED</span> or{" "}
          <span className="text-red-400 font-medium">REPRODUCED</span> the incident.
        </p>
      </div>

      {/* Verdict cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <div className="rounded-[3px] border border-green-500/30 bg-green-500/5 p-5">
          <p className="font-mono text-sm font-bold text-green-400">BLOCKED ✓</p>
          <p className="mt-2 text-xs text-muted leading-relaxed">
            The patched program caused the transaction to fail. The fix would have prevented the incident.
          </p>
        </div>
        <div className="rounded-[3px] border border-red-500/30 bg-red-500/5 p-5">
          <p className="font-mono text-sm font-bold text-red-400">REPRODUCED ✗</p>
          <p className="mt-2 text-xs text-muted leading-relaxed">
            The patched program did not prevent the transaction from succeeding. Further investigation needed.
          </p>
        </div>
      </div>

      {/* CLI examples */}
      <div className="space-y-3">
        <p className="label text-muted">CLI · rustag forensics</p>
        <div className="space-y-2">
          {CLI_EXAMPLES.map((ex) => (
            <div key={ex.label} className="rounded-[3px] border border-border bg-surface/40 p-4">
              <p className="mb-2 text-xs text-faint">{ex.label}</p>
              <pre className="overflow-x-auto text-xs text-brand whitespace-pre-wrap">{ex.cmd}</pre>
            </div>
          ))}
        </div>
      </div>

      {/* How it works */}
      <div className="rounded-[3px] border border-border bg-surface/40 p-5 space-y-3">
        <p className="label text-muted">How counterfactual mode works</p>
        <ol className="space-y-2 text-xs text-muted leading-relaxed list-decimal list-inside">
          <li>Fetch the historical transaction by signature from mainnet RPC.</li>
          <li>Reconstruct the Clock sysvar at the transaction&apos;s slot and blockTime.</li>
          <li>Load the current deployed program; then override with the patched ELF.</li>
          <li>Re-execute in a sealed LiteSVM — the SVM has no network access.</li>
          <li>If the transaction now <span className="text-red-400">fails</span>, verdict = BLOCKED. If it still <span className="text-green-400">succeeds</span>, verdict = REPRODUCED.</li>
          <li>Emit a signed EvidenceBundle with the semantic diff and invariant alarms.</li>
        </ol>
      </div>

      {/* Docs link */}
      <Link
        href="/docs"
        className="inline-flex items-center gap-1.5 text-sm text-brand hover:text-brand-strong transition-colors"
      >
        Read the Forensics docs
        <ArrowUpRight size={15} />
      </Link>
    </div>
  );
}
