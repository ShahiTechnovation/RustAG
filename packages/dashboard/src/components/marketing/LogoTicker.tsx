import { Marquee } from "@/components/ui";

const PROTOCOLS = ["UNISWAP", "PLEIADES", "CHAINLINK", "ALCHEMY", "ALLIUM", "LAYERZERO", "BITGO", "TRM LABS", "BLOCKSCOUT", "QUICKNODE"];

export function LogoTicker() {
  return (
    <section className="border-y border-border bg-surface/20 py-10">
      <p className="label mb-7 justify-center text-center">
        Preload real mainnet state from the protocols you build against
      </p>
      <Marquee>
        {PROTOCOLS.map((name) => (
          <span
            key={name}
            className="font-display select-none text-xl font-bold uppercase tracking-tight text-faint transition-colors duration-300 hover:text-fg"
          >
            {name}
          </span>
        ))}
      </Marquee>
    </section>
  );
}
