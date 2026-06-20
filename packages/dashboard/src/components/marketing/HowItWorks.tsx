import { Reveal, Section } from "@/components/ui";

const STEPS = [
  {
    n: "01",
    title: "Start a stagenet",
    body: "rustag start spins up a persistent, mainnet-mirroring SVM with a Solana-compatible JSON-RPC, WebSocket, and REST API.",
  },
  {
    n: "02",
    title: "Point your Connection",
    body: "Swap your RPC endpoint to the stagenet URL. Anchor tests, web3.js, and your existing tooling just work - unchanged.",
  },
  {
    n: "03",
    title: "Test, replay, attest",
    body: "Airdrop freely, fork and stress-test, time-travel through checkpoints, then export a signed attestation of the exact state you tested.",
  },
];

export function HowItWorks() {
  return (
    <Section
      id="how-it-works"
      eyebrow="How it works"
      title="From zero to mirrored mainnet in three steps"
    >
      <Reveal>
        <div className="border-t border-border">
          {STEPS.map((s) => (
            <div
              key={s.n}
              className="grid grid-cols-[auto_1fr] items-baseline gap-x-6 gap-y-2 border-b border-border py-8 sm:grid-cols-[6rem_18rem_1fr] sm:py-10"
            >
              <div className="font-mono text-3xl font-bold tabular-nums text-brand sm:text-4xl">
                {s.n}
              </div>
              <h3 className="font-display text-xl font-bold uppercase tracking-tight text-fg sm:text-2xl">
                {s.title}
              </h3>
              <p className="col-span-2 text-sm leading-relaxed text-muted sm:col-span-1">
                {s.body}
              </p>
            </div>
          ))}
        </div>
      </Reveal>
    </Section>
  );
}
