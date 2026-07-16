import { Reveal, Section, Terminal } from "@/components/ui";

const COPY_TEXT = `# rehearse a Squads multisig proposal
rustag rehearse \\
  --proposal 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU \\
  --rpc $HELIUS_RPC

# or rehearse a raw transaction
rustag rehearse --payload <base64_tx> --rpc $RPC

# forensics: was this historical exploit stoppable?
rustag forensics <SIGNATURE> \\
  --rpc $RPC \\
  --patch ./patched-program.so \\
  --patch-program <PROGRAM_ID>

# verify the bundle offline (no RPC needed)
rustag verify groundtruth-bundle.json \\
  --closure groundtruth-closure.json`;

const C = {
  comment: "text-faint",
  kw: "text-brand",
  str: "text-fg/80",
  fn: "text-accent-2",
  punct: "text-muted",
  plain: "text-fg",
  flag: "text-brand/70",
};

export function TerminalShowcase() {
  return (
    <Section
      eyebrow="CLI-first"
      title="Rehearse, attest, and verify in minutes"
      description="Paste a Squads proposal address — RustAG fetches it, rehearses it against live mainnet state, and returns a signed EvidenceBundle you can verify offline."
      containerClassName="max-w-3xl"
    >
      <Reveal>
        <Terminal title="bash" copyText={COPY_TEXT}>
          <pre className="whitespace-pre text-xs leading-relaxed sm:text-sm">
            <span className={C.comment}># rehearse a Squads multisig proposal{"\n"}</span>
            <span className={C.plain}>rustag </span>
            <span className={C.fn}>rehearse</span>
            {" \\\n  "}
            <span className={C.flag}>--proposal </span>
            <span className={C.str}>7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU</span>
            {" \\\n  "}
            <span className={C.flag}>--rpc </span>
            <span className={C.str}>$HELIUS_RPC</span>
            {"\n\n"}
            <span className={C.comment}># forensics: was this exploit stoppable?{"\n"}</span>
            <span className={C.plain}>rustag </span>
            <span className={C.fn}>forensics</span>
            <span className={C.punct}> &lt;SIGNATURE&gt;</span>
            {" \\\n  "}
            <span className={C.flag}>--rpc </span>
            <span className={C.str}>$RPC</span>
            {" \\\n  "}
            <span className={C.flag}>--patch </span>
            <span className={C.str}>./patched-program.so</span>
            {"\n\n"}
            <span className={C.comment}># verify the bundle offline (no RPC needed){"\n"}</span>
            <span className={C.plain}>rustag </span>
            <span className={C.fn}>verify</span>
            <span className={C.punct}> groundtruth-bundle.json</span>
            {" \\\n  "}
            <span className={C.flag}>--closure </span>
            <span className={C.str}>groundtruth-closure.json</span>
            {"\n"}
            <span className={C.kw}>✓ </span>
            <span className={C.plain}>Grade A · signature valid · pre-state root matches</span>
          </pre>
        </Terminal>
      </Reveal>
    </Section>
  );
}
