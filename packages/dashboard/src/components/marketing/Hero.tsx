"use client";

import { motion } from "motion/react";
import { ArrowUpRight } from "lucide-react";

import { ButtonLink, Eyebrow, GridBackground, RingField } from "@/components/ui";
import { MirrorVisual } from "./MirrorVisual";
import { ParticleField } from "./ParticleField";

const ease = [0.22, 1, 0.36, 1] as const;

const STATS = [
  { value: "0", label: "SOL Spent" },
  { value: "∞", label: "Airdrops" },
  { value: "<1s", label: "Oracle" },
  { value: "100%", label: "Accurate" },
];

export function Hero() {
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
            Solana Mainnet · Mirrored On Demand
          </Eyebrow>
        </motion.div>

        <motion.h1
          className="font-display mt-6 text-balance text-5xl font-bold leading-[0.98] tracking-tight text-fg sm:text-6xl lg:text-7xl"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.05 }}
        >
          A staging Solana that mirrors{" "}
          <em className="font-serif italic font-normal text-brand">mainnet</em>.
        </motion.h1>

        <motion.p
          className="mt-7 max-w-xl text-pretty text-lg leading-relaxed text-muted"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.12 }}
        >
          RustAG lazily mirrors real mainnet accounts into a persistent stagenet — test against live
          Pyth prices and Raydium pools with unlimited free airdrops and{" "}
          <span className="text-fg">zero SOL spent</span>.
        </motion.p>

        <motion.div
          className="mt-9 flex flex-wrap items-center justify-center gap-3"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.19 }}
        >
          <ButtonLink href="/app" size="lg" className="group">
            Launch the dashboard
            <ArrowUpRight size={18} className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
          </ButtonLink>
          <ButtonLink href="#features" size="lg" variant="secondary" className="group">
            Explore features
            <ArrowUpRight size={16} className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
          </ButtonLink>
        </motion.div>

        <motion.div
          className="mt-12 grid w-full max-w-2xl grid-cols-2 divide-x divide-y divide-border border border-border sm:grid-cols-4 sm:divide-y-0"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease, delay: 0.26 }}
        >
          {STATS.map((s) => (
            <div key={s.label} className="flex flex-col items-center gap-1 px-4 py-5">
              <span className="font-display text-3xl font-bold tracking-tight text-fg sm:text-4xl">
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
            <span className="text-faint">Product Proof</span>
          </div>
          <MirrorVisual />
        </div>
      </motion.div>
    </section>
  );
}
