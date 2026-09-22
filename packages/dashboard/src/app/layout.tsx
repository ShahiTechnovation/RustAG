import type { Metadata } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import { NetworkProvider } from '@/components/NetworkProvider';
import { Shell } from '@/components/Shell';
import './globals.css';
const sans = Geist({ subsets: ['latin'], variable: '--font-sans' });
const mono = Geist_Mono({ subsets: ['latin'], variable: '--font-mono' });
const title = 'RustAG | Pre-execution assurance for Robinhood Chain';
const description = 'Rehearse privileged Robinhood Chain transactions before execution. Inspect calls, state changes, permissions and risk signals before signing.';
export const metadata: Metadata = { title, description, metadataBase: new URL('https://rh.rustag.xyz'), openGraph: { title, description, type: 'website' }, twitter: { card: 'summary', title, description } };
export default function RootLayout({ children }: { children: React.ReactNode }) { return <html lang="en" className={`${sans.variable} ${mono.variable}`}><body><NetworkProvider><Shell>{children}</Shell></NetworkProvider></body></html>; }
