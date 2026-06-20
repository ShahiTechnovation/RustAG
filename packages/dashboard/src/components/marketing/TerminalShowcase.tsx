import { Reveal, Section, Terminal } from "@/components/ui";

const COPY_TEXT = `# start a stagenet
rustag start

import { RustagClient } from "@rustag/sdk";
import { Connection } from "@solana/web3.js";

const client = new RustagClient({ baseUrl: "http://localhost:9000" });
const stagenet = await client.getStagenet();

// unlimited, instant, free
await client.airdrop(wallet, 1000);

// drop-in Solana connection against the stagenet
const connection = new Connection(stagenet.rpcUrl);`;

const C = {
  comment: "text-faint",
  kw: "text-brand",
  str: "text-fg/80",
  fn: "text-accent-2",
  punct: "text-muted",
  plain: "text-fg",
};

export function TerminalShowcase() {
  return (
    <Section
      eyebrow="Drop-in"
      title="Point your Connection at it and go"
      description="RustAG speaks the Solana JSON-RPC. Swap your endpoint - your existing tooling and tests just work."
      containerClassName="max-w-3xl"
    >
      <Reveal>
        <Terminal title="bash · typescript" copyText={COPY_TEXT}>
          <pre className="whitespace-pre">
            <span className={C.comment}># start a stagenet{"\n"}</span>
            <span className={C.plain}>rustag </span>
            <span className={C.fn}>start</span>
            {"\n\n"}
            <span className={C.kw}>import</span>
            <span className={C.punct}> {"{"} </span>
            <span className={C.plain}>RustagClient</span>
            <span className={C.punct}> {"}"} </span>
            <span className={C.kw}>from</span> <span className={C.str}>&quot;@rustag/sdk&quot;</span>
            <span className={C.punct}>;</span>
            {"\n"}
            <span className={C.kw}>import</span>
            <span className={C.punct}> {"{"} </span>
            <span className={C.plain}>Connection</span>
            <span className={C.punct}> {"}"} </span>
            <span className={C.kw}>from</span> <span className={C.str}>&quot;@solana/web3.js&quot;</span>
            <span className={C.punct}>;</span>
            {"\n\n"}
            <span className={C.kw}>const</span> <span className={C.plain}>client</span>
            <span className={C.punct}> = </span>
            <span className={C.kw}>new</span> <span className={C.fn}>RustagClient</span>
            <span className={C.punct}>({"{"} baseUrl: </span>
            <span className={C.str}>&quot;http://localhost:9000&quot;</span>
            <span className={C.punct}> {"}"});</span>
            {"\n"}
            <span className={C.kw}>const</span> <span className={C.plain}>stagenet</span>
            <span className={C.punct}> = </span>
            <span className={C.kw}>await</span> <span className={C.plain}>client</span>
            <span className={C.punct}>.</span>
            <span className={C.fn}>getStagenet</span>
            <span className={C.punct}>();</span>
            {"\n\n"}
            <span className={C.comment}>// unlimited, instant, free{"\n"}</span>
            <span className={C.kw}>await</span> <span className={C.plain}>client</span>
            <span className={C.punct}>.</span>
            <span className={C.fn}>airdrop</span>
            <span className={C.punct}>(wallet, </span>
            <span className={C.str}>1000</span>
            <span className={C.punct}>);</span>
            {"\n\n"}
            <span className={C.comment}>// drop-in Solana connection against the stagenet{"\n"}</span>
            <span className={C.kw}>const</span> <span className={C.plain}>connection</span>
            <span className={C.punct}> = </span>
            <span className={C.kw}>new</span> <span className={C.fn}>Connection</span>
            <span className={C.punct}>(stagenet.rpcUrl);</span>
          </pre>
        </Terminal>
      </Reveal>
    </Section>
  );
}
