import { NextResponse } from "next/server";

/**
 * Early-access waitlist intake.
 *
 * Validates a submission and, if `EARLY_ACCESS_WEBHOOK_URL` is configured,
 * forwards it (e.g. to a Google Sheet / Zapier / Discord webhook). Without a
 * webhook it logs the lead server-side and still returns success so the form
 * works out of the box. No secrets are committed — wire the store via env.
 */

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
// Solana addresses are base58-encoded 32-byte public keys (no 0,O,I,l).
const SOLANA_RE = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

function cleanHandle(raw: string): string {
  return raw.trim().replace(/^@/, "").replace(/^https?:\/\/(x|twitter)\.com\//i, "");
}

export async function POST(request: Request) {
  let body: Record<string, unknown>;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: "Invalid JSON body." }, { status: 400 });
  }

  const email = String(body.email ?? "").trim();
  const wallet = String(body.wallet ?? "").trim();
  const twitter = cleanHandle(String(body.twitter ?? ""));
  const followed = Boolean(body.followed);

  const errors: Record<string, string> = {};
  if (!EMAIL_RE.test(email)) errors.email = "Enter a valid email address.";
  if (!SOLANA_RE.test(wallet)) errors.wallet = "Enter a valid Solana wallet address.";
  if (!twitter || /\s/.test(twitter)) errors.twitter = "Enter your X (Twitter) handle.";
  if (!followed) errors.followed = "Please confirm you follow @RustAG_xyz.";

  if (Object.keys(errors).length > 0) {
    return NextResponse.json({ error: "Validation failed.", errors }, { status: 422 });
  }

  const lead = {
    email,
    wallet,
    twitter: `@${twitter}`,
    followed,
    submittedAt: new Date().toISOString(),
  };

  const webhook = process.env.EARLY_ACCESS_WEBHOOK_URL;
  if (webhook) {
    try {
      const res = await fetch(webhook, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(lead),
      });
      if (!res.ok) throw new Error(`webhook responded ${res.status}`);
    } catch (err) {
      console.error("[early-access] webhook forward failed", err);
      return NextResponse.json(
        { error: "Could not record your request. Please try again shortly." },
        { status: 502 },
      );
    }
  } else {
    console.info("[early-access] new lead (no webhook configured):", lead);
  }

  return NextResponse.json({ ok: true });
}
