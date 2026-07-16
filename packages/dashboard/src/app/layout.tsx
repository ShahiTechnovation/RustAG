import { Analytics } from "@vercel/analytics/next";
import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Archivo, Geist, Geist_Mono, Instrument_Serif } from "next/font/google";

import { Grain } from "@/components/ui";

import "./globals.css";
import { Providers } from "./providers";

const sans = Geist({
  subsets: ["latin"],
  variable: "--font-geist-sans",
  display: "swap",
});

const display = Archivo({
  subsets: ["latin"],
  variable: "--font-archivo",
  display: "swap",
});

const mono = Geist_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
  display: "swap",
});

const serif = Instrument_Serif({
  subsets: ["latin"],
  weight: "400",
  style: ["normal", "italic"],
  variable: "--font-instrument-serif",
  display: "swap",
});

export const metadata: Metadata = {
  title: {
    default: "RustAG · Attested Pre-Execution Assurance for Solana",
    template: "%s · RustAG",
  },
  description:
    "RustAG is the GroundTruth layer for Solana — a cryptographically attested pre-execution rehearsal engine. Know exactly what a transaction does before any multisig signer approves it.",
  metadataBase: new URL(
    process.env.NEXT_PUBLIC_SITE_URL ??
      (process.env.VERCEL_PROJECT_PRODUCTION_URL
        ? `https://${process.env.VERCEL_PROJECT_PRODUCTION_URL}`
        : "https://rustag.dev"),
  ),
  openGraph: {
    title: "RustAG · GroundTruth Pre-Execution Assurance for Solana",
    description:
      "Rehearse any Solana transaction against faithful mainnet state. Get a signed, offline-verifiable EvidenceBundle before a single multisig signer approves.",
    type: "website",
    images: [{ url: "/og.png", width: 1200, height: 630, alt: "RustAG — pre-execution assurance for Solana" }],
  },
  twitter: {
    card: "summary_large_image",
    title: "RustAG · GroundTruth for Solana",
    description:
      "Know exactly what a privileged transaction does before you sign it. Signed EvidenceBundle, semantic diff, invariant alarms.",
    images: ["/og.png"],
  },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html
      lang="en"
      className={`${sans.variable} ${display.variable} ${mono.variable} ${serif.variable}`}
    >
      <body className="min-h-screen bg-bg text-fg antialiased">
        <Providers>{children}</Providers>
        <Grain />
        <Analytics />
      </body>
    </html>
  );
}
