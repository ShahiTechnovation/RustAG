'use client';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import { ArrowUpRight, Menu, X, ShieldCheck, Wallet, ChevronDown } from 'lucide-react';
import { useNetwork } from './NetworkProvider';
import { explorerUrl, shorten } from '@/lib/evm/explorer';
export function NetworkSelect() {
  const { network, setNetwork } = useNetwork();
  return <label className="network-select"><span className="dot"/><span className="sr-only">Selected Robinhood network</span><select value={network} onChange={e => setNetwork(e.target.value as 'mainnet' | 'testnet')}><option value="testnet">RH Testnet</option><option value="mainnet">RH Mainnet</option></select><ChevronDown size={12}/></label>;
}
export function Shell({ children }: { children: React.ReactNode }) {
  const n = useNetwork(); const path = usePathname(); const [menu, setMenu] = useState(false);
  const wrong = n.address && n.walletChain !== n.chain.id;
  return <><a href="#main" className="skip-link">Skip to content</a><header className="site-header"><div className="header-inner"><Link className="brand" href="/" aria-label="RustAG home"><span className="brand-mark"><ShieldCheck size={23}/></span>RustAG<span className="brand-context">/ Robinhood Chain</span></Link><nav className={menu ? 'nav open' : 'nav'} aria-label="Main navigation">{[['Rehearse','/rehearse'],['Evidence','/evidence'],['Activity','/activity'],['Docs','/docs']].map(([label,href]) => <Link key={href} aria-current={path === href ? 'page' : undefined} href={href} onClick={() => setMenu(false)}>{label}</Link>)}<a href="https://github.com/ShahiTechnovation/RustAG/tree/robinhood" target="_blank" rel="noreferrer">GitHub <ArrowUpRight size={12}/></a></nav><div className="header-actions"><NetworkSelect/>{n.address ? <button className="button wallet-button" onClick={n.disconnect} title="Disconnect from this app">{shorten(n.address)} <span>Disconnect</span></button> : <button className="button wallet-button" disabled={n.busy} onClick={n.connect}><Wallet size={14}/>{n.busy ? 'Connecting…' : 'Connect Wallet'}</button>}<button className="menu-button" aria-expanded={menu} aria-label="Toggle navigation" onClick={() => setMenu(!menu)}>{menu ? <X/> : <Menu/>}</button></div></div></header>
    {(wrong || n.error) && <div className="wallet-notice" role="status">{n.error || `Wallet is on chain ${n.walletChain}. Select ${n.chain.name} (${n.chain.id}) to match this workspace.`}{wrong && <button disabled={n.busy} onClick={n.switchWallet}>Switch wallet network ↗</button>}</div>}
    <main id="main">{children}</main><footer><Link href="/" className="brand"><ShieldCheck size={20}/>RustAG</Link><p>Independent infrastructure built for Robinhood Chain.</p><span>FRONTEND PREVIEW <span className="dot"/></span></footer></>;
}
export function NetworkStatus() {
  const { network, chain, status, walletChain, address } = useNetwork();
  return <div className="network-status"><span className={status.state === 'connected' ? 'dot' : 'dot muted'}/><span>{chain.name}</span><span className="status-divider"/><span>{status.state === 'loading' ? 'Checking RPC…' : status.state === 'error' ? 'RPC unavailable · retrying' : <>Block <a href={explorerUrl(network, 'block', status.block!)} target="_blank" rel="noreferrer">#{Number(status.block).toLocaleString('en-US')} ↗</a></>}</span><span className="chain-id">Chain ID {chain.id}{address ? ` · Wallet ${walletChain}` : ''}</span></div>;
}
