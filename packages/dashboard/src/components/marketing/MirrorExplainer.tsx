import { ArrowRight } from "lucide-react";

import type { SyncState } from "@rustag/sdk";

import { Reveal } from "@/components/ui";
import { cn } from "@/lib/cn";

const STATES: { state: SyncState; dot: string; ring: string; text: string; desc: string }[] = [
  {
    state: "Unknown",
    dot: "bg-state-unknown",
    ring: "border-state-unknown/30",
    text: "text-faint",
    desc: "Never touched. Not yet fetched from mainnet.",
  },
  {
    state: "Clean",
    dot: "bg-state-clean",
    ring: "border-state-clean/30",
    text: "text-state-clean",
    desc: "Mirrored from mainnet, unmodified. Auto-refreshes with live state.",
  },
  {
    state: "Dirty",
    dot: "bg-state-dirty",
    ring: "border-state-dirty/30",
    text: "text-state-dirty",
    desc: "Modified locally. Frozen from mainnet sync so your test stays put.",
  },
  {
    state: "Pinned",
    dot: "bg-state-pinned",
    ring: "border-state-pinned/35",
    text: "text-state-pinned",
    desc: "Explicitly overridden. Locked to exactly the value you set.",
  },
];

export function MirrorExplainer() {
  return (
    <section id="the-mirror" className="relative scroll-mt-24 px-6 py-24 sm:py-32">
      <div className="mx-auto max-w-5xl text-center">
        <Reveal>
          <div className="label mb-5 justify-center text-brand">The account state machine</div>
          <p className="mx-auto max-w-3xl text-balance font-serif text-3xl italic leading-snug text-fg sm:text-[2.6rem]">
            “Clean until you touch it. Dirty when you do. Pinned when you mean it.”
          </p>
          <p className="mx-auto mt-5 max-w-2xl text-pretty text-base leading-relaxed text-muted">
            Every account carries a sync state. It&apos;s how RustAG replays mainnet on a local SVM
            with no block hash to fork from - and the single idea the whole product is built around.
          </p>
        </Reveal>

        <Reveal delay={0.1}>
          <div className="mt-14 flex flex-col items-stretch gap-3 lg:flex-row lg:items-stretch">
            {STATES.map((s, i) => (
              <div key={s.state} className="flex flex-1 items-stretch gap-3">
                <div
                  className={cn(
                    "flex-1 rounded-[3px] border bg-surface p-4 text-left",
                    s.ring,
                  )}
                >
                  <div className="flex items-center gap-2">
                    <span className={cn("size-1.5 rounded-[1px]", s.dot)} />
                    <span
                      className={cn(
                        "font-mono text-[11px] uppercase tracking-[0.18em]",
                        s.text,
                      )}
                    >
                      {s.state}
                    </span>
                  </div>
                  <p className="mt-2.5 text-xs leading-relaxed text-muted">{s.desc}</p>
                </div>
                {i < STATES.length - 1 ? (
                  <ArrowRight
                    size={16}
                    className="hidden shrink-0 self-center rotate-90 text-faint lg:block lg:rotate-0"
                  />
                ) : null}
              </div>
            ))}
          </div>
        </Reveal>
      </div>
    </section>
  );
}
