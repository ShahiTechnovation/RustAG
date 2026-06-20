"use client";

import { useState } from "react";
import { Pause, Play, Plus, Trash2 } from "lucide-react";

import { Badge, Button, Card, Field, GlowCard, Input } from "@/components/ui";
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
    create.mutate({ name, schedule, action: { type: "airdrop", pubkey, sol } });
  };

  return (
    <div className="space-y-8">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Scheduler</h1>
        <p className="mt-1 text-sm text-muted">
          Recurring on-chain activities - simulate steady usage with periodic airdrops, transfers, or
          replayed transactions.
        </p>
      </div>

      <GlowCard className="p-6">
        <form onSubmit={submit} className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5 lg:items-end">
          <Field label="Name">
            <Input value={name} onChange={(e) => setName(e.target.value)} />
          </Field>
          <Field label="Schedule">
            <Input
              value={schedule}
              onChange={(e) => setSchedule(e.target.value)}
              placeholder="@every 30s or */5 * * * *"
            />
          </Field>
          <Field label="Airdrop to (pubkey)">
            <Input value={pubkey} onChange={(e) => setPubkey(e.target.value)} />
          </Field>
          <Field label="SOL">
            <Input
              type="number"
              step="0.1"
              value={sol}
              onChange={(e) => setSol(Number(e.target.value))}
            />
          </Field>
          <Button type="submit" disabled={create.isPending || !pubkey}>
            <Plus size={16} />
            {create.isPending ? "Adding…" : "Add activity"}
          </Button>
        </form>
        {create.isError ? (
          <p className="mt-2 text-xs text-red-400">{(create.error as Error).message}</p>
        ) : null}
      </GlowCard>

      {isError ? (
        <Card className="text-sm text-muted">Could not reach the stagenet REST API.</Card>
      ) : (
        <div className="space-y-2">
          {(schedules ?? []).length === 0 ? (
            <Card className="text-sm text-muted">No activities yet - add one above.</Card>
          ) : (
            (schedules ?? []).map((s) => (
              <Card key={s.id} className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="font-medium text-fg">{s.name}</span>
                    <Badge tone={s.enabled ? "emerald" : "zinc"}>{s.enabled ? "enabled" : "paused"}</Badge>
                    <code className="rounded-[3px] bg-white/5 px-1.5 py-0.5 font-mono text-xs text-muted">
                      {s.schedule}
                    </code>
                  </div>
                  <div className="mt-1 truncate text-xs text-faint">
                    {s.action.type} · {s.runCount} runs · last:{" "}
                    {s.lastStatus ? <span className="text-muted">{s.lastStatus}</span> : "-"}
                  </div>
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => toggle.mutate({ id: s.id, enabled: !s.enabled })}
                  >
                    {s.enabled ? <Pause size={14} /> : <Play size={14} />}
                    {s.enabled ? "Pause" : "Resume"}
                  </Button>
                  <button
                    onClick={() => remove.mutate(s.id)}
                    className="inline-flex items-center gap-1.5 rounded-[3px] border border-red-900/60 px-3 py-1.5 text-xs text-red-400 transition-colors hover:bg-red-950/40 cursor-pointer"
                  >
                    <Trash2 size={14} />
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
