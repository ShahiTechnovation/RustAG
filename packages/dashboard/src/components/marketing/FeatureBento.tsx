"use client";

import {
  BadgeCheck,
  CalendarClock,
  FileSearch,
  FlaskConical,
  GitBranch,
  History,
  Layers,
  LineChart,
  Radio,
  ShieldAlert,
  Squircle,
} from "lucide-react";

import type { AccountInfo, SyncState } from "@rustag/sdk";

import { AnimatedNumber, BentoCard, Reveal, Section, StatePill } from "@/components/ui";
import { useAccounts, useStagenet } from "@/lib/hooks";

function short(pk: string) {
  return pk.length > 10 ? `${pk.slice(0, 4)}…${pk.slice(-4)}` : pk;
}

/** Real transaction count from the live demo backend. */
function LiveTxCount() {
  const { data } = useStagenet();
  return (
    <div className="font-mono text-sm text-brand">
      <AnimatedNumber value={data?.transactions ?? 0} />{" "}
      <span className="text-faint">rehearsals on the live demo</span>
    </div>
  );
}

const FALLBACK_PILLS: { pubkey: string; state: SyncState }[] = [
  { pubkey: "H6AR…jcW9", state: "Clean" },
  { pubkey: "US51…LFx", state: "Dirty" },
  { pubkey: "Toke…5DA", state: "Clean" },
];

/** Real mirrored accounts (pubkey + Clean/Dirty/Pinned) from the live demo. */
function MirrorPills() {
  const { data } = useAccounts(6);
  const pills =
    data && data.length
      ? data.slice(0, 3).map((a: AccountInfo) => ({ pubkey: short(a.pubkey), state: a.syncState }))
      : FALLBACK_PILLS;
  return (
    <div className="mt-2 space-y-2">
      {pills.map((p, i) => (
        <div
          key={`${p.pubkey}-${i}`}
          className="flex items-center justify-between rounded-[3px] border border-border bg-subtle px-3 py-2 text-xs"
        >
          <span className="font-mono text-faint">{p.pubkey}</span>
          <StatePill state={p.state} />
        </div>
      ))}
    </div>
  );
}

/** Simulated alarm pills for the invariant card. */
function AlarmPills() {
  const alarms = [
    { rule: "upgrade-authority", sev: "HIGH", color: "text-yellow-400" },
    { rule: "nonce-authority-combo", sev: "CRITICAL", color: "text-red-400" },
    { rule: "large-sol-drain", sev: "HIGH", color: "text-yellow-400" },
  ];
  return (
    <div className="mt-2 space-y-2">
      {alarms.map((a) => (
        <div
          key={a.rule}
          className="flex items-center justify-between rounded-[3px] border border-border bg-subtle px-3 py-2 text-xs"
        >
          <span className="font-mono text-faint">{a.rule}</span>
          <span className={`font-mono font-bold ${a.color}`}>{a.sev}</span>
        </div>
      ))}
    </div>
  );
}

export function FeatureBento() {
  return (
    <Section
      id="features"
      eyebrow="Everything you need"
      title="The GroundTruth assurance stack"
      description="From Squads proposal decoding to counterfactual forensics — the complete toolkit for pre-execution assurance on Solana."
    >
      <Reveal>
        <div className="grid auto-rows-[minmax(0,1fr)] grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">

          {/* Featured: Sealed Rehearsal */}
          <BentoCard
            className="lg:col-span-2 lg:row-span-2"
            index="01"
            accent="var(--brand)"
            icon={<BadgeCheck size={20} />}
            title="Sealed pre-execution rehearsal"
            description="Two-pass deterministic execution in a sealed LiteSVM sandbox. Pass 1 discovers all touched accounts. Pass 2 re-executes with zero live RPC calls, producing a SHA-256 content-addressed state root — Grade A means anyone can re-verify."
            media={<MirrorPills />}
          />

          {/* Squads v4 Integration */}
          <BentoCard
            index="02"
            icon={<Squircle size={20} />}
            accent="var(--brand)"
            title="Squads v4 native"
            description="Paste a VaultTransaction proposal pubkey. RustAG fetches, Borsh-decodes, and rehearses it — threshold, approval count, and all — before a single signer touches Approve."
          />

          {/* Invariant Alarms */}
          <BentoCard
            index="03"
            icon={<ShieldAlert size={20} />}
            accent="var(--brand)"
            title="6-rule invariant policy"
            description="Upgrade authority rotation, program freeze, new durable-nonce, SOL drain (>80%), and the Drift attack pattern (nonce + authority combo) all trigger typed alarms."
            media={<AlarmPills />}
          />

          {/* N-of-M Provenance */}
          <BentoCard
            index="04"
            icon={<Radio size={20} />}
            accent="var(--brand)"
            title="N-of-M RPC provenance"
            description="Cross-fetch the closure from N independent RPC endpoints, require M-of-N agreement. InputProvenance is embedded in every bundle — the closure can't be silently manipulated."
          />

          {/* Forensics / Counterfactual */}
          <BentoCard
            index="05"
            icon={<FileSearch size={20} />}
            accent="var(--brand)"
            title="Counterfactual forensics"
            description="Re-execute a historical transaction by signature. Use --patch to substitute a candidate ELF — get back BLOCKED or REPRODUCED. Answer: would our fix have stopped the Drift attack?"
            media={<LiveTxCount />}
          />

          {/* Upgrade CI Gate */}
          <BentoCard
            index="06"
            icon={<GitBranch size={20} />}
            accent="var(--brand)"
            title="Upgrade-rehearsal CI gate"
            description="GitHub Action records real mainnet traffic for a watched program, replays against the candidate bytecode, and fails CI on new invariant alarms. PR comment shows the semantic diff."
          />

          {/* Verifiable Attestation */}
          <BentoCard
            index="07"
            icon={<Layers size={20} />}
            accent="var(--brand)"
            title="Ed25519-signed EvidenceBundle"
            description="Every rehearsal produces a tamper-evident, offline-verifiable bundle: pre/post state roots, semantic diff, alarms, compute, signer pubkey. Verify with zero network dependency."
          />

          {/* Time-series analytics */}
          <BentoCard
            index="08"
            icon={<LineChart size={20} />}
            accent="var(--brand)"
            title="Time-series analytics"
            description="TVL, transaction volume, and mirror growth — sampled and charted in real time against the live demo."
          />

          {/* Simulation & stress */}
          <BentoCard
            index="09"
            icon={<FlaskConical size={20} />}
            accent="var(--brand)"
            title="Fork, stress & replay"
            description="Fork the stagenet, replay real traffic corpora, and compare outcomes across candidate program versions without mutating the base."
          />

        </div>
      </Reveal>
    </Section>
  );
}
