import type { Metadata } from "next";
import type { ReactNode } from "react";

import { DocsChrome } from "@/components/docs/DocsChrome";

export const metadata: Metadata = {
  title: {
    default: "Documentation",
    template: "%s · RustAG Docs",
  },
  description:
    "RustAG documentation — a persistent, mainnet-mirroring staging environment for Solana programs. Quickstart, CLI reference, SDK, JSON-RPC compatibility, and architecture.",
};

export default function DocsLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-dvh bg-bg">
      <DocsChrome />
      <div className="lg:pl-64">
        <main className="min-h-dvh pt-14">{children}</main>
      </div>
    </div>
  );
}
