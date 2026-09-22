"use client";
import { useState } from "react";
import { Check, ArrowUpRight, Download, Copy, ChevronRight } from "lucide-react";
import type { Call, RehearsalResult, Severity } from "@/lib/rehearsal/types";
import { downloadEvidence } from "@/lib/rehearsal/client";
import { explorerUrl, shorten } from "@/lib/evm/explorer";
import type { Network } from "@/lib/chains/robinhood";

export function SeverityBadge({ severity }: { severity: Severity }) {
  return <span className={`severity ${severity}`}>{severity}</span>;
}

function AddressLink({ value, network }: { value: string; network: Network }) {
  return (
    <a className="mono address-link" title={value} href={explorerUrl(network, "address", value)} target="_blank" rel="noreferrer">
      {shorten(value)} <ArrowUpRight size={12} />
    </a>
  );
}

function Trace({ call, network }: { call: Call; network: Network }) {
  return (
    <details className="trace" open>
      <summary>
        <ChevronRight size={13} />
        <span className={call.type === "DELEGATECALL" ? "call-type delegate" : "call-type"}>{call.type}</span>
        <strong>{call.label}</strong>
        <span className="trace-gas">{call.gas.toLocaleString("en-US")} gas</span>
        <span className="green">{call.success ? "Success" : "Revert"}</span>
      </summary>
      <div className="trace-body">
        <AddressLink value={call.address} network={network} />
        <code>{call.function}</code>
        <div className="trace-detail">Value: {call.value} &middot; Calldata: {call.data} &middot; Return: {call.returnData}</div>
        {call.children.map((child, i) => <Trace key={i} call={child} network={network} />)}
      </div>
    </details>
  );
}

export function EvidenceDetails({ result }: { result: RehearsalResult }) {
  const [copyStatus, setCopyStatus] = useState("");
  const e = result.evidence;
  const network: Network = e.chainId === 4663 ? "mainnet" : "testnet";
  const fields: [string, string | number | null][] = [
    ["Chain ID", e.chainId], ["Block number", e.blockNumber ?? "Not pinned \xb7 demo"],
    ["Block hash", e.blockHash ?? "Unavailable"], ["Transaction payload hash", e.payloadHash],
    ["Pre-state commitment", e.preStateCommitment ?? "Not produced"], ["Post-state commitment", e.postStateCommitment ?? "Not produced"],
    ["Timestamp", e.timestamp], ["RPC provenance", e.rpcProvenance ?? "None \xb7 fixture only"],
    ["Attestation / signature", e.attestation ?? "Unsigned"],
    ["Execution result", `${result.success ? "Success" : "Revert"} \xb7 ${result.mode}`],
    ["Findings", `${result.findings.length} illustrative findings`],
  ];
  return (
    <div className="evidence-details">
      <div className="notice">
        {result.mode === "demo" ? "Demo evidence is unsigned and contains no RPC execution, block provenance or state commitments. The payload hash is computed locally from the submitted request." : "Inspect provenance and attestation before relying on this result."}
      </div>
      <dl className="evidence-grid">
        {fields.map(([label, value]) => (
          <div key={String(label)}><dt>{label}</dt><dd>{String(value ?? "\u2014")}</dd></div>
        ))}
      </dl>
      <div className="cta-row">
        <button className="button" onClick={async () => {
          try { await navigator.clipboard.writeText(e.payloadHash); setCopyStatus("Hash copied"); }
          catch { setCopyStatus("Clipboard unavailable. Select the hash above to copy."); }
        }}><Copy size={14} /> Copy Hash</button>
        <button className="button" onClick={() => downloadEvidence(result)}><Download size={14} /> Download Evidence</button>
        {e.blockNumber && <a className="button" href={explorerUrl(network, "block", e.blockNumber)} target="_blank" rel="noreferrer">View Block <ArrowUpRight size={14} /></a>}
        <a className="button" href={explorerUrl(network, "address", result.request.to)} target="_blank" rel="noreferrer">View Address <ArrowUpRight size={14} /></a>
      </div>
      {copyStatus && <p role="status" className="muted-text">{copyStatus}</p>}
    </div>
  );
}

const TABS = ["Overview", "Call Trace", "State Diff", "Token / ETH Changes", "Permissions", "Approvals", "Storage Changes", "Gas", "Reverts", "Risk Signals", "Evidence"] as const;
type Tab = (typeof TABS)[number];
const KIND_MAP: Partial<Record<Tab, RehearsalResult["changes"][number]["kind"]>> = {
  "Token / ETH Changes": "balance", Permissions: "permission", Approvals: "approval", "Storage Changes": "storage",
};

export function Results({ result }: { result: RehearsalResult }) {
  const [tab, setTab] = useState<Tab>("Overview");
  const network: Network = result.evidence.chainId === 4663 ? "mainnet" : "testnet";
  const kindFilter = KIND_MAP[tab];
  const changes = kindFilter ? result.changes.filter((c) => c.kind === kindFilter) : result.changes;

  return (
    <section className="results panel">
      <div className="panel-title">
        <div>
          <span className="eyebrow">EXECUTION REVIEW</span>
          <h2><Check className="green" size={22} />Execution {result.success ? "succeeds" : "reverts"}</h2>
        </div>
        <span className="demo-badge">{result.mode.toUpperCase()} RESULT</span>
      </div>
      <div className="notice">Fixed illustrative scenario. These outcomes do not describe your submitted transaction. No transaction was executed or signed.</div>
      <div className="tabs" role="tablist" aria-label="Execution result sections">
        {TABS.map((t) => (
          <button key={t} role="tab" id={`tab-${t.replaceAll(" ", "-")}`} aria-controls="result-panel" aria-selected={tab === t} onClick={() => setTab(t)}>{t}</button>
        ))}
      </div>
      <div id="result-panel" role="tabpanel" aria-labelledby={`tab-${tab.replaceAll(" ", "-")}`} className="tab-content">
        {tab === "Overview" && (
          <>
            <div className="summary-grid">
              {([[result.changes.length, "State changes"], [result.changes.filter(c => c.kind === "balance").length, "Balance changes"], [result.changes.filter(c => c.kind === "permission").length, "Privileged role changed"], [result.gas.toLocaleString("en-US"), "Estimated gas"]] as [string | number, string][]).map(([value, label]) => (
                <div key={label}><strong>{value}</strong><span>{label}</span></div>
              ))}
            </div>
            <h3>Review before approval</h3>
            <p className="muted-text">This sample upgrade includes a permission change and an unlimited approval. A successful execution alone does not make a transaction safe.</p>
            <div className="findings">
              {result.findings.map((f) => (
                <article key={f.title}><SeverityBadge severity={f.severity} /><div><h4>{f.title}</h4><p>{f.detail}</p></div></article>
              ))}
            </div>
          </>
        )}
        {tab === "Call Trace" && (
          <><p className="muted-text">Illustrative calls. Expand each node to inspect its children. Calldata and return bytes are placeholders.</p>
            {result.calls.map((call, i) => <Trace key={i} call={call} network={network} />)}</>
        )}
        {(tab === "State Diff" || kindFilter) && (
          <div className="diff-table">
            <div className="diff-header"><span>FIELD / CONTRACT</span><span>BEFORE</span><span>AFTER</span></div>
            {changes.map((c) => (
              <div className="diff-row" key={`${c.address}-${c.field}`}>
                <div><strong>{c.field}</strong><AddressLink value={c.address} network={network} /></div>
                <div className="before">{c.before.startsWith("0x") && c.before.length === 42 ? <AddressLink value={c.before} network={network} /> : c.before}</div>
                <div className="after">{c.after.startsWith("0x") && c.after.length === 42 ? <AddressLink value={c.after} network={network} /> : c.after}</div>
              </div>
            ))}
          </div>
        )}
        {tab === "Gas" && <><div className="gas-number">{result.gas.toLocaleString("en-US")} <span>gas</span></div><p className="muted-text">Illustrative execution estimate. Not a live fee quote; data availability fees and network pricing are not included.</p></>}
        {tab === "Reverts" && <div className="empty-inline"><Check size={24} /><h3>{result.revert ?? "No revert in the demo scenario"}</h3><p>No conclusion can be drawn about your submitted transaction.</p></div>}
        {tab === "Risk Signals" && (
          <div className="findings">
            {result.findings.map((f) => <article key={f.title}><SeverityBadge severity={f.severity} /><div><h4>{f.title}</h4><p>{f.detail}</p></div></article>)}
            <div className="severity-legend">Severity scale: {(["critical", "high", "medium", "low", "info"] as Severity[]).map((s) => <SeverityBadge key={s} severity={s} />)}</div>
          </div>
        )}
        {tab === "Evidence" && <EvidenceDetails result={result} />}
      </div>
    </section>
  );
}
