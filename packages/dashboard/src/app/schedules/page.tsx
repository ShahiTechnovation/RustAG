"use client";

import { useState } from "react";

import { Badge, Card } from "@/components/ui";
import {
  useCreateSchedule,
  useDeleteSchedule,
  useSchedules,
  useToggleSchedule,
} from "@/lib/hooks";

export default function SchedulesPage() {
  const { data: schedules, isError } = useSchedules();
  const create = useCreateSchedule();
  const toggle = useToggleSchedule();
  const remove = useDeleteSchedule();

  const [name, setName] = useState("faucet-topup");
  const [schedule, setSchedule] = useState("@every 30s");
  const [pubkey, setPubkey] = useState("");
  const [sol, setSol] = useState(1);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!pubkey) return;
    create.mutate({
      name,
      schedule,
      action: { type: "airdrop", pubkey, sol },
    });
  };

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-semibold text-zinc-50">Scheduler</h1>
        <p className="mt-1 text-sm text-zinc-500">
          Recurring on-chain activities — simulate steady usage with periodic airdrops, transfers,
          or replayed transactions.
        </p>
      </div>

      <Card>
        <form onSubmit={submit} className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5 lg:items-end">
          <Field label="Name">
            <input className={inputCls} value={name} onChange={(e) => setName(e.target.value)} />
          </Field>
          <Field label="Schedule">
            <input
              className={inputCls}
              value={schedule}
              onChange={(e) => setSchedule(e.target.value)}
              placeholder="@every 30s or */5 * * * *"
            />
          </Field>
          <Field label="Airdrop to (pubkey)">
            <input className={inputCls} value={pubkey} onChange={(e) => setPubkey(e.target.value)} />
          </Field>
          <Field label="SOL">
            <input
              type="number"
              step="0.1"
              className={inputCls}
              value={sol}
              onChange={(e) => setSol(Number(e.target.value))}
            />
          </Field>
          <button
            type="submit"
            disabled={create.isPending || !pubkey}
            className="h-9 rounded-md bg-indigo-500 px-4 text-sm font-medium text-white transition-colors hover:bg-indigo-400 disabled:opacity-50"
          >
            {create.isPending ? "Adding…" : "Add activity"}
          </button>
        </form>
        {create.isError ? (
          <p className="mt-2 text-xs text-red-400">{(create.error as Error).message}</p>
        ) : null}
      </Card>

      {isError ? (
        <Card>Could not reach the stagenet REST API.</Card>
      ) : (
        <div className="space-y-2">
          {(schedules ?? []).length === 0 ? (
            <Card>No activities yet — add one above.</Card>
          ) : (
            (schedules ?? []).map((s) => (
              <Card key={s.id} className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-zinc-100">{s.name}</span>
                    <Badge tone={s.enabled ? "emerald" : "zinc"}>
                      {s.enabled ? "enabled" : "paused"}
                    </Badge>
                    <code className="rounded bg-zinc-800 px-1.5 py-0.5 text-xs text-zinc-300">
                      {s.schedule}
                    </code>
                  </div>
                  <div className="mt-1 truncate text-xs text-zinc-500">
                    {s.action.type} · {s.runCount} runs · last:{" "}
                    {s.lastStatus ? <span className="text-zinc-400">{s.lastStatus}</span> : "—"}
                  </div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <button
                    onClick={() => toggle.mutate({ id: s.id, enabled: !s.enabled })}
                    className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
                  >
                    {s.enabled ? "Pause" : "Resume"}
                  </button>
                  <button
                    onClick={() => remove.mutate(s.id)}
                    className="rounded-md border border-red-900/60 px-3 py-1.5 text-xs text-red-400 hover:bg-red-950/40"
                  >
                    Delete
                  </button>
                </div>
              </Card>
            ))
          )}
        </div>
      )}
    </div>
  );
}

const inputCls =
  "h-9 w-full rounded-md border border-zinc-700 bg-zinc-950 px-2 text-sm text-zinc-100 outline-none focus:border-indigo-500";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium uppercase tracking-wider text-zinc-500">
        {label}
      </span>
      {children}
    </label>
  );
}
