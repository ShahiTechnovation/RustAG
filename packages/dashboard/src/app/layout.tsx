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
  metadataBase: new URL("https://rustag.dev"),
  openGraph: {
    title: "RustAG - A staging Solana that mirrors mainnet",
    description:
      "Test Solana programs against real mainnet state. Zero SOL spent. Unlimited airdrops. Time-travel replay, verifiable attestation, MEV simulation.",
    type: "website",
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
