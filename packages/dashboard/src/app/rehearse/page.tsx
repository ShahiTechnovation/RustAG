"use client";
import { useRef, useState } from "react";
import { ArrowUpRight, Code2, FlaskConical } from "lucide-react";
import { useNetwork } from "@/components/NetworkProvider";
import { NetworkSelect, NetworkStatus } from "@/components/Shell";
import { Results } from "@/components/Results";
import { DEMO_ADDRESSES, REHEARSAL_MODE, rehearsalClient } from "@/lib/rehearsal/client";
import { parseTransaction, parseTransactionJson } from "@/lib/rehearsal/validation";

export default function Rehearse() {
  const n = useNetwork();
  const [mode, setMode] = useState("Raw calldata");
  const [form, setForm] = useState({ from: "", to: "", value: "0", data: "0x", block: "" });
  const [json, setJson] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const review = useRef<HTMLDivElement>(null);

  function update(key: keyof typeof form, value: string) {
    setForm((f) => ({ ...f, [key]: value }));
    setError("");
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      const fields = mode === "Raw calldata" ? form : parseTransactionJson(json, n.chain.id);
      const request = parseTransaction(fields, n.chain.id);
      const result = await rehearsalClient.rehearse(request);
      n.setResult(result);
      setTimeout(() => review.current?.scrollIntoView({ behavior: "smooth", block: "start" }), 100);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to prepare this transaction.");
    } finally {
      setBusy(false);
    }
  }

  function loadExample() {
    setMode("Raw calldata");
    setForm({ from: DEMO_ADDRESSES.safe, to: DEMO_ADDRESSES.proxy, value: "0", data: "0x3659cfe6", block: "" });
    setError("");
  }

  return (
    <div className="workspace">
      <div className="page-heading">
        <div>
          <div className="eyebrow">TRANSACTION WORKSPACE</div>
          <h1>Rehearse. Inspect. Decide.</h1>
          <p>Understand a privileged call before it becomes a commitment.</p>
        </div>
        <span className="demo-badge">
          <FlaskConical size={13} /> {REHEARSAL_MODE.toUpperCase()} MODE
        </span>
      </div>

      <NetworkStatus />

      <div className="workspace-grid">
        <form className="panel transaction-form" onSubmit={submit}>
          <div className="panel-title">
            <h2><Code2 size={19} />Transaction input</h2>
            <button className="text-button" type="button" onClick={loadExample}>Load example &#8599;</button>
          </div>

          <div className="input-modes">
            {["Raw calldata", "Paste transaction", "Paste Safe transaction"].map((m) => (
              <button key={m} type="button" aria-pressed={mode === m} className={mode === m ? "active" : ""} onClick={() => { setMode(m); setError(""); }}>{m}</button>
            ))}
          </div>

          <div className="form-content">
            <div className="field">
              <label>Network</label>
              <NetworkSelect />
            </div>

            {mode === "Raw calldata" ? (
              <>
                <div className="field">
                  <div className="label-row">
                    <label htmlFor="from">From / Safe / Executor <span>optional</span></label>
                    {n.address && <button type="button" className="text-button" onClick={() => update("from", n.address!)}>Use wallet</button>}
                  </div>
                  <input id="from" placeholder="0x&#8230;" value={form.from} onChange={(e) => update("from", e.target.value)} spellCheck={false} />
                </div>
                <div className="field">
                  <label htmlFor="to">Target contract</label>
                  <input id="to" required placeholder="0x&#8230;" value={form.to} onChange={(e) => update("to", e.target.value)} spellCheck={false} />
                </div>
                <div className="two-fields">
                  <div className="field">
                    <label htmlFor="value">ETH value</label>
                    <input id="value" required inputMode="decimal" value={form.value} onChange={(e) => update("value", e.target.value)} />
                  </div>
                  <div className="field">
                    <label htmlFor="block">Block number <span>optional</span></label>
                    <input id="block" inputMode="numeric" placeholder="Latest" value={form.block} onChange={(e) => update("block", e.target.value)} />
                  </div>
                </div>
                <div className="field">
                  <label htmlFor="data">Calldata</label>
                  <textarea id="data" required rows={5} value={form.data} onChange={(e) => update("data", e.target.value)} spellCheck={false} />
                  <small>Hex-encoded bytes. Use 0x for an empty payload.</small>
                </div>
              </>
            ) : (
              <div className="field">
                <label htmlFor="json">{mode} JSON</label>
                <textarea
                  id="json"
                  rows={12}
                  required
                  value={json}
                  onChange={(e) => setJson(e.target.value)}
                  placeholder={'{\n  "to": "0x\u2026",\n  "value": "0",\n  "data": "0x"\n}'}
                  spellCheck={false}
                />
                <small>Value is in wei. Single transaction only. Safe exports may use a transactions array with one item.</small>
              </div>
            )}

            {error && <div className="error-notice" role="alert">{error}</div>}

            <button className="button primary submit" disabled={busy} type="submit">
              {busy ? "Preparing preview\u2026" : "Rehearse Transaction"} <ArrowUpRight size={17} />
            </button>
            <p className="form-footnote">
              {REHEARSAL_MODE === "demo"
                ? "Demo preview only. No signing or transaction submission."
                : "Live mode. Connects to the rehearsal backend. No signing or submission."}
            </p>
          </div>
        </form>

        <aside className="workspace-aside">
          <div className="eyebrow">BEFORE YOU BEGIN</div>
          <h3>A preview of your<br />next checkpoint.</h3>
          <p>Wallet connectivity and network status are live. Rehearsal results are a fixed, clearly labeled example.</p>
          {[
            ["01", "Prepare the call", "Enter an EVM transaction or paste a Safe export."],
            ["02", "Explore the review", "Inspect the demo trace, state diff and risk signals."],
            ["03", "Keep the context", "Download unsigned demo evidence with your input hash."],
          ].map(([num, title, body]) => (
            <div className="aside-step" key={num}>
              <span>{num}</span>
              <div><h4>{title}</h4><p>{body}</p></div>
            </div>
          ))}
          {REHEARSAL_MODE === "demo" && (
            <div className="notice">Live EVM rehearsal is not connected yet. Do not use demo results to approve a transaction.</div>
          )}
        </aside>
      </div>

      <div ref={review}>
        {n.result && <Results result={n.result} />}
      </div>
    </div>
  );
}
