"use client";

import { motion } from "motion/react";
import { ArrowUpRight, ShieldCheck } from "lucide-react";

import { AnimatedNumber, ButtonLink, Eyebrow, GridBackground, RingField } from "@/components/ui";
import { useStagenet } from "@/lib/hooks";
import { MirrorVisual } from "./MirrorVisual";
import { ParticleField } from "./ParticleField";

const ease = [0.22, 1, 0.36, 1] as const;

export function Hero() {
  const { data, isError } = useStagenet();
  const live = !!data && !isError;
  const stats: { value: React.ReactNode; label: string }[] = [
    { value: "0", label: "SOL spent" },
    { value: "Grade A", label: "Fidelity" },
    { value: live ? <AnimatedNumber value={data.accounts} /> : "—", label: "Accounts mirrored" },
    { value: live ? <AnimatedNumber value={data.slot} /> : "—", label: "Live slot" },
  ];

  return (
    <section className="relative overflow-hidden px-6 pb-24 pt-36 sm:pt-40">
      <RingField />
      <GridBackground className="opacity-40" />
      <ParticleField className="absolute inset-0 -z-10 size-full opacity-30 [mask-image:radial-gradient(ellipse_70%_60%_at_50%_40%,#000,transparent)]" />

      <div className="mx-auto flex max-w-4xl flex-col items-center text-center">
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease }}
        >
          <Eyebrow className="justify-center text-brand">
            <ShieldCheck size={13} className="mr-1" />
            GroundTruth · Pre-Execution Assurance for Solana
          </Eyebrow>
        </motion.div>

        <motion.h1
          className="font-display mt-6 text-balance text-5xl font-bold leading-[0.98] tracking-tight text-fg sm:text-6xl lg:text-7xl"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.05 }}
        >
          Know what a transaction{" "}
          <em className="font-serif italic font-normal text-brand">does</em>{" "}
          before you sign it.
        </motion.h1>

        <motion.p
          className="mt-7 max-w-xl text-pretty text-lg leading-relaxed text-muted"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.12 }}
        >
          RustAG rehearses any Solana transaction against faithful mainnet state in a sealed sandbox
          — then emits a{" "}
          <span className="text-fg">cryptographically signed EvidenceBundle</span>{" "}
          with a semantic diff, invariant alarms, and compute used. Verify it offline, before a single
          multisig signer approves.
        </motion.p>

        <motion.div
          className="mt-9 flex flex-wrap items-center justify-center gap-3"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.19 }}
        >
          <ButtonLink href="/app" size="lg" className="group">
            Open the live demo
            <ArrowUpRight size={18} className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
          </ButtonLink>
          <ButtonLink href="/early-access" size="lg" variant="secondary" className="group">
            Request early access
            <ArrowUpRight size={16} className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
          </ButtonLink>
        </motion.div>

        <motion.div
          className="mt-12 grid w-full max-w-2xl grid-cols-2 divide-x divide-y divide-border border border-border sm:grid-cols-4 sm:divide-y-0"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.26 }}
        >
          {stats.map((s) => (
            <div key={s.label} className="flex flex-col items-center gap-1 px-4 py-5">
              <span className="font-display text-3xl font-bold tracking-tight text-fg tabular-nums sm:text-4xl">
                {s.value}
              </span>
              <span className="label">{s.label}</span>
            </div>
          ))}
        </motion.div>
      </div>

      <motion.div
        className="mx-auto mt-16 flex max-w-4xl justify-center"
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease, delay: 0.34 }}
      >
        <div className="w-full max-w-md border border-border bg-surface/40 p-3">
          <div className="label mb-3 flex items-center justify-between px-1">
            <span className="text-brand">Live Mirror</span>
            <span className="text-faint">Faithful mainnet state</span>
          </div>
          <MirrorVisual />
        </div>
      </motion.div>
    </section>
  );
}
