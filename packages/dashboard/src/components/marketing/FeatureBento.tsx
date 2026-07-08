"use client";

import {
  BadgeCheck,
  CalendarClock,
  Coins,
  FlaskConical,
  History,
  Layers,
  LineChart,
  Radio,
  ShieldAlert,
} from "lucide-react";

import type { AccountInfo, SyncState } from "@rustag/sdk";

import { AnimatedNumber, BentoCard, Reveal, Section, StatePill } from "@/components/ui";
import { useAccounts, useStagenet } from "@/lib/hooks";

function short(pk: string) {
  return pk.length > 10 ? `${pk.slice(0, 4)}…${pk.slice(-4)}` : pk;
}

/** Real transaction count from the live demo (mostly the heartbeat's airdrops). */
function LiveAirdrops() {
  const { data } = useStagenet();
  return (
    <div className="font-mono text-sm text-brand">
      <AnimatedNumber value={data?.transactions ?? 0} />{" "}
      <span className="text-faint">txs on the live demo</span>
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

export function FeatureBento() {
  return (
    <Section
      id="features"
      eyebrow="Everything you need"
      title="A complete staging layer for Solana"
      description="From lazy mirroring to verifiable attestation - the full toolkit for testing programs against real on-chain state."
    >
      <Reveal>
        <div className="grid auto-rows-[minmax(0,1fr)] grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {/* Featured */}
          <BentoCard
            className="lg:col-span-2 lg:row-span-2"
            index="01"
            accent="var(--brand)"
            icon={<Layers size={20} />}
            title="Lazy mainnet mirror"
            description="Accounts are fetched from mainnet on first access, cached, and tracked through a Clean → Dirty → Pinned lifecycle. Replay mainnet on a local SVM - no block hash, no fork required."
            media={<MirrorPills />}
          />

          <BentoCard
            index="02"
            icon={<Radio size={20} />}
            accent="var(--brand)"
            title="Real-time oracle mirror"
            description="Sub-second oracle refresh via accountSubscribe WebSocket - Pyth prices push to your stagenet, not poll."
          />

          <BentoCard
            index="03"
            icon={<Coins size={20} />}
            accent="var(--brand)"
            title="Unlimited free airdrops"
            description="No faucet limits. Credit any wallet instantly with zero mainnet SOL spent."
            media={<LiveAirdrops />}
          />

          <BentoCard
            index="04"
            icon={<CalendarClock size={20} />}
            accent="var(--brand)"
            title="Activity scheduler"
            description="Recurring on-chain actions on @every or cron - simulate steady, lifelike usage."
          />

          <BentoCard
            index="05"
            icon={<FlaskConical size={20} />}
            accent="var(--brand)"
            title="Simulation & stress"
            description="Fork the stagenet, replay thousands of txs, and compare outcomes without mutating the base."
          />

          <BentoCard
            index="06"
            icon={<History size={20} />}
            accent="var(--brand)"
            title="Time-travel replay"
            description="Checkpoints, transaction journals, and fork-of-fork lineage - replay deterministically and diff any two points."
          />

          <BentoCard
            index="07"
            icon={<BadgeCheck size={20} />}
            accent="var(--brand)"
            title="Verifiable attestation"
            description="SHA-256 Merkle commitment + Ed25519 signing. Prove you tested against exactly this state, offline."
          />

          <BentoCard
            index="08"
            icon={<LineChart size={20} />}
            accent="var(--brand)"
            title="Time-series analytics"
            description="TVL, transaction volume, and mirror growth - sampled and charted in real time."
          />

          <BentoCard
            index="09"
            icon={<ShieldAlert size={20} />}
            accent="var(--brand)"
            title="MEV, fuzz & exploit scan"
            description="Jito-style atomic bundles, deterministic invariant fuzzing, and reproducible exploit signatures."
          />
        </div>
      </Reveal>
    </Section>
  );
}
