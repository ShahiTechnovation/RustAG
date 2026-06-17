"use client";

import { useState } from "react";

import { useAirdrop, useOverride, usePreload } from "@/lib/hooks";
import { Card } from "./ui";

const PRELOAD_TARGETS = ["jupiter", "pyth", "raydium", "orca", "marinade", "spl-token"];

const inputClass =
  "w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600 focus:border-indigo-500 focus:outline-none";
const buttonClass =
  "rounded-md bg-indigo-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500 disabled:opacity-50";

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
      <Card>
        <h3 className="text-sm font-semibold text-zinc-100">Airdrop SOL</h3>
        <p className="mt-1 text-xs text-zinc-500">Unlimited, instant, free.</p>
        <div className="mt-4 space-y-2">
          <input
            className={inputClass}
            placeholder="Wallet address"
            value={airdropPubkey}
            onChange={(e) => setAirdropPubkey(e.target.value)}
          />
          <input
            className={inputClass}
            placeholder="Amount (SOL)"
            inputMode="decimal"
            value={airdropSol}
            onChange={(e) => setAirdropSol(e.target.value)}
          />
          <button
            className={buttonClass}
            disabled={!airdropPubkey || airdrop.isPending}
            onClick={() => airdrop.mutate({ pubkey: airdropPubkey.trim(), sol: Number(airdropSol) || 0 })}
          >
            {airdrop.isPending ? "Airdropping…" : "Airdrop"}
          </button>
          {airdrop.isError ? <p className="text-xs text-red-400">{String(airdrop.error)}</p> : null}
          {airdrop.isSuccess ? <p className="text-xs text-emerald-400">Done.</p> : null}
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-semibold text-zinc-100">Override balance</h3>
        <p className="mt-1 text-xs text-zinc-500">Pin an account&apos;s lamports.</p>
        <div className="mt-4 space-y-2">
          <input
            className={inputClass}
            placeholder="Account address"
            value={overridePubkey}
            onChange={(e) => setOverridePubkey(e.target.value)}
          />
          <input
            className={inputClass}
            placeholder="Lamports"
            inputMode="numeric"
            value={overrideLamports}
            onChange={(e) => setOverrideLamports(e.target.value)}
          />
          <button
            className={buttonClass}
            disabled={!overridePubkey || !overrideLamports || override.isPending}
            onClick={() =>
              override.mutate({
                pubkey: overridePubkey.trim(),
                lamports: Number(overrideLamports) || 0,
              })
            }
          >
            {override.isPending ? "Setting…" : "Set balance"}
          </button>
          {override.isError ? <p className="text-xs text-red-400">{String(override.error)}</p> : null}
          {override.isSuccess ? <p className="text-xs text-emerald-400">Done.</p> : null}
        </div>
      </Card>

      <Card>
        <h3 className="text-sm font-semibold text-zinc-100">Preload mainnet state</h3>
        <p className="mt-1 text-xs text-zinc-500">Pull real DeFi accounts.</p>
        <div className="mt-4 flex flex-wrap gap-2">
          {PRELOAD_TARGETS.map((target) => (
            <button
              key={target}
              className="rounded-md border border-zinc-700 px-2.5 py-1.5 text-xs text-zinc-300 transition-colors hover:border-indigo-500 hover:text-indigo-300 disabled:opacity-50"
              disabled={preload.isPending}
              onClick={() => preload.mutate([target])}
            >
              {target}
            </button>
          ))}
        </div>
        {preload.isPending ? <p className="mt-3 text-xs text-zinc-500">Loading…</p> : null}
        {preload.isSuccess ? (
          <p className="mt-3 text-xs text-emerald-400">Loaded {preload.data.loaded} accounts.</p>
        ) : null}
        {preload.isError ? <p className="mt-3 text-xs text-red-400">{String(preload.error)}</p> : null}
      </Card>
    </div>
  );
}
