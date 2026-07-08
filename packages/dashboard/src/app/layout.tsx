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
    default: "RustAG - A staging Solana that mirrors mainnet",
    template: "%s · RustAG",
  },
  description:
    "RustAG is Tenderly Virtual TestNets for Solana - a persistent, mainnet-mirroring staging environment. Test against real on-chain state with zero SOL spent and unlimited airdrops.",
  // Prefer an explicit site URL, else the Vercel production URL at build time,
  // so OG/Twitter image URLs resolve to the real deployed domain.
  metadataBase: new URL(
    process.env.NEXT_PUBLIC_SITE_URL ??
      (process.env.VERCEL_PROJECT_PRODUCTION_URL
        ? `https://${process.env.VERCEL_PROJECT_PRODUCTION_URL}`
        : "https://rustag.dev"),
  ),
  openGraph: {
    title: "RustAG - A staging Solana that mirrors mainnet",
    description:
      "Test Solana programs against real mainnet state. Zero SOL spent. Unlimited airdrops. Time-travel replay, verifiable attestation, MEV simulation.",
    type: "website",
    images: [{ url: "/og.png", width: 1200, height: 630, alt: "RustAG - a staging Solana that mirrors mainnet" }],
  },
  twitter: {
    card: "summary_large_image",
    title: "RustAG - A staging Solana that mirrors mainnet",
    description:
      "Test Solana programs against real mainnet state. Zero SOL spent. Unlimited airdrops.",
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
