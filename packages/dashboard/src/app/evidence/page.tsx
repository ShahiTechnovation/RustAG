'use client';
import { Fingerprint, ArrowUpRight } from 'lucide-react';
import Link from 'next/link';
import { useNetwork } from '@/components/NetworkProvider';
import { demoClient, demoRequest } from '@/lib/rehearsal/client';
import { EvidenceDetails } from '@/components/Results';
export default function EvidencePage() {
  const n = useNetwork();
  return <div className="workspace"><div className="page-heading"><div><div className="eyebrow">REVIEWABLE BY DESIGN</div><h1>Evidence, with context.</h1><p>A clear record of the input, the execution and what changed.</p></div><Fingerprint size={40} className="green"/></div>{n.result ? <section className="panel evidence-panel"><div className="panel-title"><h2>Execution evidence</h2><span className="demo-badge">UNSIGNED DEMO</span></div><EvidenceDetails result={n.result}/></section> : <section className="panel empty-state"><Fingerprint size={38}/><h2>No evidence generated yet.</h2><p>Prepare a transaction in the rehearsal workspace, or explore an unsigned example. Live execution attestations are not available in this phase.</p><div className="cta-row"><button className="button primary" onClick={async()=>n.setResult(await demoClient.rehearse({...demoRequest,chainId:n.chain.id}))}>Explore demo evidence <ArrowUpRight size={16}/></button><Link className="button" href="/rehearse">Prepare a transaction</Link></div></section>}<div className="evidence-principles"><article><span className="eyebrow">01 / PROVENANCE</span><h3>Know the source.</h3><p>The future engine must pin chain ID, block hash and RPC provenance for every execution.</p></article><article><span className="eyebrow">02 / COMMITMENTS</span><h3>Inspect the difference.</h3><p>Pre-state and post-state commitments belong alongside the trace and findings.</p></article><article><span className="eyebrow">03 / ATTESTATION</span><h3>Verify before relying.</h3><p>Unsigned demo evidence is not proof. No signature or verification claim is fabricated.</p></article></div></div>;
}
