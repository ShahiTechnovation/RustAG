import { NextResponse } from "next/server";

/**
 * Early-access waitlist intake.
 *
 * Persists each submission to Upstash Redis (Vercel KV) via its REST API:
 *   - RPUSH early_access:leads      → ordered log of every submission (JSON)
 *   - SADD  early_access:emails     → unique set of emails (for de-dupe/count)
 *
 * Credentials come from the Upstash/Vercel integration env vars, which use the
 * "Early_access" custom prefix. No secrets are committed — they live in Vercel
 * project env (and .env.local for local dev).
 */

const KV_URL = process.env.Early_access_KV_REST_API_URL;
const KV_TOKEN = process.env.Early_access_KV_REST_API_TOKEN;

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
// Solana addresses are base58-encoded 32-byte public keys (no 0,O,I,l).
const SOLANA_RE = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

function cleanHandle(raw: string): string {
  return raw.trim().replace(/^@/, "").replace(/^https?:\/\/(x|twitter)\.com\//i, "");
}

/** Run a batch of Redis commands against the Upstash REST pipeline endpoint. */
async function kvPipeline(commands: (string | number)[][]): Promise<void> {
  if (!KV_URL || !KV_TOKEN) {
    throw new Error("Upstash KV is not configured (missing Early_access_KV_REST_API_* env).");
  }
  const res = await fetch(`${KV_URL}/pipeline`, {
    method: "POST",
    headers: {
      authorization: `Bearer ${KV_TOKEN}`,
      "content-type": "application/json",
    },
    body: JSON.stringify(commands),
    cache: "no-store",
  });
  if (!res.ok) {
    throw new Error(`Upstash responded ${res.status}: ${await res.text()}`);
  }
}

export async function POST(request: Request) {
  let body: Record<string, unknown>;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: "Invalid JSON body." }, { status: 400 });
  }

  const email = String(body.email ?? "").trim().toLowerCase();
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

  try {
    await kvPipeline([
      ["RPUSH", "early_access:leads", JSON.stringify(lead)],
      ["SADD", "early_access:emails", email],
    ]);
  } catch (err) {
    console.error("[early-access] failed to persist lead", err);
    return NextResponse.json(
      { error: "Could not record your request. Please try again shortly." },
      { status: 502 },
    );
  }

  return NextResponse.json({ ok: true });
}
