import Link from "next/link";
import { ArrowUpRight } from "lucide-react";

export default function EarlyAccessPage() {
  return (
    <div className="workspace">
      <div className="page-heading">
        <div>
          <div className="eyebrow">EARLY ACCESS</div>
          <h1>Join the waitlist.</h1>
          <p>
            RustAG for Robinhood Chain is in active development. Request early
            access to be notified when the live EVM rehearsal engine is
            available.
          </p>
        </div>
      </div>
      <div className="panel" style={{ maxWidth: "520px", padding: "32px" }}>
        <h2 style={{ fontSize: "20px", marginBottom: "12px" }}>
          Stay in the loop
        </h2>
        <p className="muted-text" style={{ marginBottom: "24px" }}>
          Follow the{" "}
          <a
            href="https://github.com/ShahiTechnovation/RustAG"
            target="_blank"
            rel="noreferrer"
            style={{ color: "var(--green)" }}
          >
            GitHub repository ↗
          </a>{" "}
          for updates, or reach out directly.
        </p>
        <div className="cta-row">
          <a
            href="https://github.com/ShahiTechnovation/RustAG"
            className="button primary"
            target="_blank"
            rel="noreferrer"
          >
            View on GitHub <ArrowUpRight size={15} />
          </a>
          <Link href="/rehearse" className="button">
            Explore demo workspace
          </Link>
        </div>
      </div>
    </div>
  );
}
