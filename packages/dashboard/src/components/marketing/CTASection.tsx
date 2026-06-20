import { ArrowUpRight } from "lucide-react";

import { ButtonLink, Reveal, RingField } from "@/components/ui";

export function CTASection() {
  return (
    <section className="px-6 py-24 sm:py-32">
      <div className="mx-auto max-w-5xl">
        <Reveal>
          <div className="relative overflow-hidden rounded-card border border-border-strong bg-surface px-6 py-20 text-center">
            <RingField />
            <h2 className="mx-auto max-w-2xl text-balance font-display text-4xl font-bold tracking-tight text-fg sm:text-6xl">
              Spin up your <span className="font-serif italic text-brand">mirror</span>.
            </h2>
            <p className="mx-auto mt-5 max-w-xl text-pretty text-base leading-relaxed text-muted">
              RustAG is in private beta. Join the early-access list and we&apos;ll reach out as we
              open up your staging Solana.
            </p>
            <div className="mt-9 flex flex-wrap items-center justify-center gap-3">
              <ButtonLink href="/early-access" size="lg" className="group">
                Request early access
                <ArrowUpRight
                  size={18}
                  className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
                />
              </ButtonLink>
              <ButtonLink href="https://github.com" external size="lg" variant="secondary">
                View on GitHub
                <ArrowUpRight size={18} />
              </ButtonLink>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
