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
    desc: "Never fetched. Pulled on first access during rehearsal closure resolution.",
  },
  {
    state: "Clean",
    dot: "bg-state-clean",
    ring: "border-state-clean/30",
    text: "text-state-clean",
    desc: "Content-addressed snapshot from mainnet. Used as the sealed pre-state root.",
  },
  {
    state: "Dirty",
    dot: "bg-state-dirty",
    ring: "border-state-dirty/30",
    text: "text-state-dirty",
    desc: "Modified by the rehearsed payload. Frozen — captured in the post-state root.",
  },
  {
    state: "Pinned",
    dot: "bg-state-pinned",
    ring: "border-state-pinned/35",
    text: "text-state-pinned",
    desc: "Explicitly overridden via --patch in forensics mode. Locked to the patched ELF.",
  },
];

export function MirrorExplainer() {
  return (
    <section id="the-mirror" className="relative scroll-mt-24 px-6 py-24 sm:py-32">
      <div className="mx-auto max-w-5xl text-center">
        <Reveal>
          <div className="label mb-5 justify-center text-brand">The faithful pre-state model</div>
          <p className="mx-auto max-w-3xl text-balance font-serif text-3xl italic leading-snug text-fg sm:text-[2.6rem]">
            "Pinned before execution. Sealed during. Signed after."
          </p>
          <p className="mx-auto mt-5 max-w-2xl text-pretty text-base leading-relaxed text-muted">
            Every account in the rehearsal closure carries a sync state. This is how RustAG
            produces a{" "}
            <span className="text-fg">content-addressed, tamper-evident pre-state root</span>{" "}
            — the cryptographic foundation that makes an EvidenceBundle independently verifiable
            by anyone, with no trust in the rehearser.
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
