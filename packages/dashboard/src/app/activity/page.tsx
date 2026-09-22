'use client';
import Link from 'next/link';
import { Activity, ArrowUpRight } from 'lucide-react';
import { useNetwork } from '@/components/NetworkProvider';
import { shorten } from '@/lib/evm/explorer';
export default function ActivityPage() { const {result}=useNetwork(); return <div className="workspace"><div className="page-heading"><div><div className="eyebrow">YOUR WORKSPACE</div><h1>Activity</h1><p>The latest preview in this browser session. Nothing is sent to a backend.</p></div></div>{result ? <div className="panel activity-row"><div><span className="demo-badge">DEMO</span><h3>Transaction preview prepared</h3><p className="mono">{shorten(result.request.to)} · Chain {result.request.chainId}</p><p className="muted-text">{new Date(result.evidence.timestamp).toLocaleString()}</p></div><Link className="button" href="/evidence">View evidence <ArrowUpRight size={15}/></Link></div> : <div className="panel empty-state"><Activity size={36}/><h2>A clean slate.</h2><p>Your latest transaction preview will appear here. Session activity clears when you reload.</p><Link className="button primary" href="/rehearse">Rehearse a transaction <ArrowUpRight size={16}/></Link></div>}</div>; }
