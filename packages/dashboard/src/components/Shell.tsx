"use client";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState, type ReactNode } from "react";
import { ArrowUpRight, Menu, X, ShieldCheck, Wallet, ChevronDown } from "lucide-react";
import { useNetwork } from "./NetworkProvider";
import { explorerUrl, shorten } from "@/lib/evm/explorer";

export function NetworkSelect() {
  const { network, setNetwork } = useNetwork();
  return (
    <label className="network-select">
      <span className="dot" />
      <span className="sr-only">Selected Robinhood network</span>
      <select value={network} onChange={(e) => setNetwork(e.target.value as "mainnet" | "testnet")}>
        <option value="mainnet">RH Mainnet</option>
        <option value="testnet">RH Testnet</option>
      </select>
      <ChevronDown size={12} />
    </label>
  );
}

export function NetworkStatus() {
  const { network, chain, status, walletChain, address } = useNetwork();
  return (
    <div className="network-status">
      <span className={status.state === "connected" ? "dot" : "dot muted"} />
      <span>{chain.name}</span>
      <span className="status-divider" />
      <span>
        {status.state === "loading" ? "Checking RPC\u2026" : status.state === "error" ? "RPC unavailable \xb7 retrying" : status.block ? (
          <>Block <a href={explorerUrl(network, "block", status.block)} target="_blank" rel="noreferrer">#{Number(status.block).toLocaleString("en-US")} \u2197</a></>
        ) : null}
      </span>
      <span className="chain-id">Chain ID {chain.id}{address ? ` \xb7 Wallet ${walletChain ?? "\u2014"}` : ""}</span>
    </div>
  );
}

const NAV_LINKS = [["Rehearse", "/rehearse"], ["Forensics", "/forensics"], ["Evidence", "/evidence"], ["Activity", "/activity"], ["Docs", "/docs"]] as const;

export function Shell({ children }: { children: ReactNode }) {
  const n = useNetwork();
  const path = usePathname();
  const [menu, setMenu] = useState(false);
  const wrongNetwork = n.address !== undefined && n.walletChain !== n.chain.id;

  return (
    <>
      <a href="#main" className="skip-link">Skip to content</a>
      <header className="site-header">
        <div className="header-inner">
          <Link className="brand" href="/" aria-label="RustAG home">
            <span className="brand-mark"><ShieldCheck size={23} /></span>
            RustAG
            <span className="brand-context">/ Robinhood Chain</span>
          </Link>
          <nav className={menu ? "nav open" : "nav"} aria-label="Main navigation">
            {NAV_LINKS.map(([label, href]) => (
              <Link key={href} aria-current={path === href ? "page" : undefined} href={href} onClick={() => setMenu(false)}>{label}</Link>
            ))}
            <a href="https://github.com/ShahiTechnovation/RustAG/tree/robinhood" target="_blank" rel="noreferrer">GitHub <ArrowUpRight size={12} /></a>
          </nav>
          <div className="header-actions">
            <NetworkSelect />
            {n.address ? (
              <button className="button wallet-button" onClick={n.disconnect} title="Disconnect from this app">
                {shorten(n.address)} <span>Disconnect</span>
              </button>
            ) : (
              <button className="button wallet-button" disabled={n.busy} onClick={n.connect}>
                <Wallet size={14} />{n.busy ? "Connecting\u2026" : "Connect Wallet"}
              </button>
            )}
            <button className="menu-button" aria-expanded={menu} aria-label="Toggle navigation" onClick={() => setMenu(!menu)}>
              {menu ? <X /> : <Menu />}
            </button>
          </div>
        </div>
      </header>

      {(wrongNetwork || n.error) && (
        <div className="wallet-notice" role="status">
          {n.error || `Wallet is on chain ${n.walletChain}. Switch to ${n.chain.name} (${n.chain.id}) to match this workspace.`}
          {wrongNetwork && <button disabled={n.busy} onClick={n.switchWallet}>Switch wallet network \u2197</button>}
        </div>
      )}

      <main id="main">{children}</main>

      <footer>
        <Link href="/" className="brand"><ShieldCheck size={20} />RustAG</Link>
        <p>Independent infrastructure built for Robinhood Chain.</p>
        <span>FRONTEND PREVIEW <span className="dot" /></span>
      </footer>
    </>
  );
}
