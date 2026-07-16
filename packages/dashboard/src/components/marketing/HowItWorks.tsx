import { Reveal, Section } from "@/components/ui";

const STEPS = [
  {
    n: "01",
    title: "Paste the proposal",
    body: "Give RustAG a Squads v4 proposal address (or a raw base64 transaction). It fetches the exact pre-state — every account, every ProgramData, the Clock sysvar — from N independent mainnet RPC endpoints.",
  },
  {
    n: "02",
    title: "Sealed rehearsal",
    body: "The payload runs in a sealed, deterministic LiteSVM sandbox against the pinned pre-state snapshot. Two-pass execution: discover all touched accounts, then re-execute with no live RPC calls.",
  },
  {
    n: "03",
    title: "Sign and verify",
    body: "Get back a signed EvidenceBundle: semantic diff (11 change types), invariant alarms (ownership, freeze, nonce, drain), compute used, and SHA-256 state roots. Verify offline — independent of the rehearser.",
  },
];

export function HowItWorks() {
  return (
    <Section
      id="how-it-works"
      eyebrow="How it works"
      title="Pre-execution assurance in three steps"
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
