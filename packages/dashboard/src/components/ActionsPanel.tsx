"use client";

import { useState } from "react";
import { Coins, Download, Pin } from "lucide-react";

import { useAirdrop, useOverride, usePreload } from "@/lib/hooks";
import { Button, GlowCard, Input } from "./ui";

const PRELOAD_TARGETS = ["jupiter", "pyth", "raydium", "orca", "marinade", "spl-token"];

function ActionHeader({ icon, title, hint }: { icon: React.ReactNode; title: string; hint: string }) {
  return (
    <div className="flex items-start gap-3">
      <span className="grid size-9 shrink-0 place-items-center rounded-[3px] border border-border bg-white/[0.03] text-brand">
        {icon}
      </span>
      <div>
        <h3 className="font-display text-sm font-semibold uppercase tracking-wide text-fg">{title}</h3>
        <p className="label mt-1 normal-case text-faint">{hint}</p>
      </div>
    </div>
  );
}

export function ActionsPanel() {
  const airdrop = useAirdrop();
  const override = useOverride();
  const preload = usePreload();

  const [airdropPubkey, setAirdropPubkey] = useState("");
  const [airdropSol, setAirdropSol] = useState("1000");
  const [overridePubkey, setOverridePubkey] = useState("");
  const [overrideLamports, setOverrideLamports] = useState("");

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <GlowCard className="space-y-4">
        <ActionHeader icon={<Coins size={17} />} title="Airdrop SOL" hint="Unlimited, instant, free." />
        <div className="space-y-2">
          <Input
            placeholder="Wallet address"
            value={airdropPubkey}
            onChange={(e) => setAirdropPubkey(e.target.value)}
          />
          <Input
            placeholder="Amount (SOL)"
            inputMode="decimal"
            value={airdropSol}
            onChange={(e) => setAirdropSol(e.target.value)}
          />
          <Button
            className="w-full"
            disabled={!airdropPubkey || airdrop.isPending}
            onClick={() => airdrop.mutate({ pubkey: airdropPubkey.trim(), sol: Number(airdropSol) || 0 })}
          >
            {airdrop.isPending ? "Airdropping…" : "Airdrop"}
          </Button>
          {airdrop.isError ? <p className="text-xs text-red-400">{String(airdrop.error)}</p> : null}
          {airdrop.isSuccess ? <p className="text-xs text-accent">Done.</p> : null}
        </div>
      </GlowCard>

      <GlowCard className="space-y-4">
        <ActionHeader icon={<Pin size={17} />} title="Override balance" hint="Pin an account's lamports." />
        <div className="space-y-2">
          <Input
            placeholder="Account address"
            value={overridePubkey}
            onChange={(e) => setOverridePubkey(e.target.value)}
          />
          <Input
            placeholder="Lamports"
            inputMode="numeric"
            value={overrideLamports}
            onChange={(e) => setOverrideLamports(e.target.value)}
          />
          <Button
            variant="secondary"
            className="w-full"
            disabled={!overridePubkey || !overrideLamports || override.isPending}
            onClick={() =>
              override.mutate({ pubkey: overridePubkey.trim(), lamports: Number(overrideLamports) || 0 })
            }
          >
            {override.isPending ? "Setting…" : "Set balance"}
          </Button>
          {override.isError ? <p className="text-xs text-red-400">{String(override.error)}</p> : null}
          {override.isSuccess ? <p className="text-xs text-accent">Done.</p> : null}
        </div>
      </GlowCard>

      <GlowCard className="space-y-4">
        <ActionHeader icon={<Download size={17} />} title="Preload mainnet state" hint="Pull real DeFi accounts." />
        <div className="flex flex-wrap gap-2">
          {PRELOAD_TARGETS.map((target) => (
            <button
              key={target}
              className="rounded-[3px] border border-border px-2.5 py-1.5 font-mono text-[11px] uppercase tracking-wider text-muted transition-colors hover:border-brand hover:text-fg disabled:opacity-50 cursor-pointer"
              disabled={preload.isPending}
              onClick={() => preload.mutate([target])}
            >
              {target}
            </button>
          ))}
        </div>
        {preload.isPending ? <p className="text-xs text-faint">Loading…</p> : null}
        {preload.isSuccess ? (
          <p className="text-xs text-accent">Loaded {preload.data.loaded} accounts.</p>
        ) : null}
        {preload.isError ? <p className="text-xs text-red-400">{String(preload.error)}</p> : null}
      </GlowCard>
    </div>
  );
}
