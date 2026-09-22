"use client";

import { FileSearch, ArrowUpRight, Check } from "lucide-react";
import Link from "next/link";
import { useNetwork } from "@/components/NetworkProvider";
import { explorerUrl } from "@/lib/evm/explorer";
import { DEMO_ADDRESSES } from "@/lib/rehearsal/client";


const DEMO_TX = "0xabcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234";

export default function ForensicsPage() {
  const { network } = useNetwork();

  return (
    <div className="workspace">
      <div className="page-heading">
        <div>
          <div className="eyebrow">TRANSACTION FORENSICS</div>
          <h1>Inspect what a transaction did.</h1>
          <p>
            Replay a historical Robinhood Chain transaction and see exactly what
            changed — calls, state diffs, balance changes and permissions.
          </p>
        </div>
        <span className="demo-badge">FRONTEND PREVIEW</span>
      </div>

      {/* Concept cards */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit,minmax(240px,1fr))",
          gap: "16px",
          marginBottom: "36px",
        }}
      >
        {[
          {
            icon: "01",
            title: "Rehearse",
            body: "Inspect a transaction before execution. Know what it will do before signing.",
            href: "/rehearse",
          },
          {
            icon: "02",
            title: "Forensics",
            body: "Replay what a historical EVM transaction actually did. Diff state before and after.",
            href: null,
          },
          {
            icon: "03",
            title: "Evidence",
            body: "Keep a verifiable record: chain ID, block hash, payload hash and findings.",
            href: "/evidence",
          },
        ].map(({ icon, title, body, href }) => (
          <div key={icon} className="panel" style={{ padding: "22px" }}>
            <div
              className="eyebrow"
              style={{ color: "var(--muted)", marginBottom: "14px" }}
            >
              {icon}
            </div>
            <h3 style={{ fontSize: "17px", letterSpacing: "-0.4px" }}>
              {title}
            </h3>
            <p className="muted-text" style={{ marginTop: "8px" }}>
              {body}
            </p>
            {href && (
              <Link
                href={href}
                className="button"
                style={{ marginTop: "16px", fontSize: "11px" }}
              >
                Open {title} <ArrowUpRight size={13} />
              </Link>
            )}
            {!href && (
              <span className="demo-badge" style={{ marginTop: "14px" }}>
                EVM engine coming
              </span>
            )}
          </div>
        ))}
      </div>

      {/* Demo trace */}
      <section className="panel" style={{ marginBottom: "32px" }}>
        <div className="panel-title">
          <div>
            <span className="eyebrow">ILLUSTRATIVE EXAMPLE</span>
            <h2>
              <FileSearch size={19} />
              Transaction replay
            </h2>
          </div>
          <span className="demo-badge">DEMO · NOT A REAL EXECUTION</span>
        </div>

        <div style={{ padding: "24px" }}>
          <div className="notice" style={{ marginBottom: "24px" }}>
            Forensics replay is frontend-ready. The EVM execution engine is not
            connected yet. The trace below is illustrative and does not describe
            any real historical transaction.
          </div>

          {/* Tx info */}
          <dl className="transaction-fields" style={{ padding: "0 0 20px" }}>
            {[
              [
                "Transaction hash",
                <a
                  key="tx"
                  href={explorerUrl(network, "tx", DEMO_TX)}
                  target="_blank"
                  rel="noreferrer"
                  className="mono"
                  style={{ fontSize: "10px" }}
                >
                  {DEMO_TX.slice(0, 18)}… ↗
                </a>,
              ],
              ["Network", network === "mainnet" ? "Robinhood Chain" : "Robinhood Chain Testnet"],
              ["Block", "Demo · not pinned"],
              ["Gas used", "148,392"],
              ["Status", <span key="ok" className="green">Success</span>],
            ].map(([label, value]) => (
              <div key={String(label)}>
                <dt>{label}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>

          {/* Verdict cards */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "16px",
              marginTop: "8px",
            }}
          >
            <div
              style={{
                padding: "20px",
                border: "1px solid #35522f",
                borderRadius: "5px",
                background: "#0e1c0b",
              }}
            >
              <p
                className="mono"
                style={{ fontSize: "13px", fontWeight: 700, color: "#c4f649" }}
              >
                <Check size={14} style={{ marginRight: "8px" }} />
                EXECUTION SUCCEEDED
              </p>
              <p
                className="muted-text"
                style={{ marginTop: "10px", lineHeight: 1.7 }}
              >
                The transaction completed without revert. State changes took
                effect.
              </p>
            </div>
            <div
              style={{
                padding: "20px",
                border: "1px solid var(--border)",
                borderRadius: "5px",
                background: "#0d100c",
              }}
            >
              <p
                className="mono"
                style={{ fontSize: "13px", fontWeight: 700, color: "#e5c97d" }}
              >
                FINDINGS: 2
              </p>
              <p
                className="muted-text"
                style={{ marginTop: "10px", lineHeight: 1.7 }}
              >
                Illustrative: permission change detected. Unlimited approval
                granted.
              </p>
            </div>
          </div>

          {/* Explorer links */}
          <div className="cta-row" style={{ marginTop: "28px" }}>
            <a
              className="button"
              href={explorerUrl(network, "address", DEMO_ADDRESSES.safe)}
              target="_blank"
              rel="noreferrer"
            >
              View executor on explorer <ArrowUpRight size={14} />
            </a>
            <a
              className="button"
              href={explorerUrl(network, "address", DEMO_ADDRESSES.proxy)}
              target="_blank"
              rel="noreferrer"
            >
              View target on explorer <ArrowUpRight size={14} />
            </a>
          </div>
        </div>
      </section>

      {/* How it will work */}
      <section className="panel">
        <div className="panel-title">
          <div>
            <span className="eyebrow">ARCHITECTURE</span>
            <h2>How forensics works</h2>
          </div>
        </div>
        <div style={{ padding: "24px" }}>
          <ol
            style={{
              paddingLeft: "18px",
              color: "var(--muted)",
              fontSize: "13px",
              lineHeight: 1.9,
            }}
          >
            <li>
              Fetch the historical transaction by hash from a Robinhood Chain
              RPC.
            </li>
            <li>
              Reconstruct the block context (block hash, timestamp, coinbase).
            </li>
            <li>
              Re-execute the transaction against the pre-transaction state using
              the EVM engine.
            </li>
            <li>
              Diff pre-state vs post-state: balances, allowances, storage,
              ownership.
            </li>
            <li>
              In counterfactual mode, override a contract with a patched version
              and compare the outcome.
            </li>
            <li>
              Emit an evidence record: chain ID, block hash, payload hash,
              findings.
            </li>
          </ol>
          <Link
            href="/docs"
            className="button"
            style={{ marginTop: "20px", fontSize: "11px" }}
          >
            Read the integration docs <ArrowUpRight size={13} />
          </Link>
        </div>
      </section>
    </div>
  );
}
