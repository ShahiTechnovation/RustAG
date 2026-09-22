// Rehearsal client — no external dependencies.
//
// NEXT_PUBLIC_RUSTAG_REHEARSAL_MODE=demo  → use the demo fixture (clearly labelled)
// NEXT_PUBLIC_RUSTAG_REHEARSAL_MODE=live  → call the live backend; surface real errors
//
// When mode is "demo": results are a fixed illustrative scenario — never claimed to
// be a real execution.
// When mode is "live": backend failures surface as thrown errors, not silent fallbacks.

import type {
  EvmAddress,
  EvmHex,
  RehearsalClient,
  RehearsalRequest,
  RehearsalResult,
} from "./types";

// ---------------------------------------------------------------------------
// Demo client
// ---------------------------------------------------------------------------

export const DEMO_ADDRESSES = {
  safe: "0x1111111111111111111111111111111111111111" as EvmAddress,
  proxy: "0x2222222222222222222222222222222222222222" as EvmAddress,
  implementation: "0x3333333333333333333333333333333333333333" as EvmAddress,
  token: "0x4444444444444444444444444444444444444444" as EvmAddress,
} as const;

export const DEMO_REQUEST: RehearsalRequest = {
  chainId: 46630, // testnet by default for demos
  from: DEMO_ADDRESSES.safe,
  to: DEMO_ADDRESSES.proxy,
  value: "0",
  data: "0x3659cfe6" as EvmHex,
};

/** Deterministic local payload hash (no crypto dependency). */
function localHash(payload: string): EvmHex {
  let h = 0x811c9dc5;
  for (let i = 0; i < payload.length; i++) {
    h ^= payload.charCodeAt(i);
    h = (Math.imul(h, 0x01000193) >>> 0);
  }
  return `0x${h.toString(16).padStart(64, "0")}` as EvmHex;
}

function makeCall(
  address: EvmAddress,
  label: string,
  type: "CALL" | "DELEGATECALL",
  fn: string,
  gas: number,
  children: RehearsalResult["calls"] = [],
): RehearsalResult["calls"][number] {
  return {
    address,
    label,
    type,
    function: fn,
    value: "0 ETH",
    gas,
    success: true,
    data: "0x" as EvmHex,
    returnData: "0x" as EvmHex,
    children,
  };
}

export const demoClient: RehearsalClient = {
  async rehearse(request: RehearsalRequest): Promise<RehearsalResult> {
    const payloadHash = localHash(JSON.stringify(request));
    return {
      mode: "demo",
      request,
      success: true,
      gas: 148_392,
      calls: [
        makeCall(
          DEMO_ADDRESSES.safe,
          "Treasury Safe",
          "CALL",
          "execTransaction(\u2026)",
          148_392,
          [
            makeCall(
              DEMO_ADDRESSES.proxy,
              "Treasury Proxy",
              "CALL",
              "upgradeToAndCall(\u2026)",
              112_640,
              [
                makeCall(
                  DEMO_ADDRESSES.implementation,
                  "Implementation v2",
                  "DELEGATECALL",
                  "initialize(\u2026)",
                  42_810,
                  [
                    makeCall(
                      DEMO_ADDRESSES.token,
                      "Treasury Token",
                      "CALL",
                      "approve(\u2026)",
                      22_400,
                    ),
                  ],
                ),
              ],
            ),
          ],
        ),
      ],
      changes: [
        {
          address: DEMO_ADDRESSES.proxy,
          field: "Implementation",
          before: "0x5555555555555555555555555555555555555555",
          after: DEMO_ADDRESSES.implementation,
          kind: "storage",
        },
        {
          address: DEMO_ADDRESSES.proxy,
          field: "Owner",
          before: DEMO_ADDRESSES.safe,
          after: "0x6666666666666666666666666666666666666666" as EvmAddress,
          kind: "permission",
        },
        {
          address: DEMO_ADDRESSES.token,
          field: "ERC-20 allowance",
          before: "0 TOKEN",
          after: "Unlimited TOKEN",
          kind: "approval",
        },
        {
          address: DEMO_ADDRESSES.safe,
          field: "ETH balance",
          before: "24.8000 ETH",
          after: "24.7997 ETH",
          kind: "balance",
        },
        {
          address: DEMO_ADDRESSES.safe,
          field: "Token balance",
          before: "125,000 TOKEN",
          after: "120,000 TOKEN",
          kind: "balance",
        },
        {
          address: DEMO_ADDRESSES.safe,
          field: "Nonce",
          before: "41",
          after: "42",
          kind: "nonce",
        },
      ],
      findings: [
        {
          severity: "high",
          title: "Unlimited token approval",
          detail:
            "Illustrative finding: the spender can move the entire token balance. Review the spender and consider a bounded allowance.",
        },
        {
          severity: "medium",
          title: "Upgrade authority changes",
          detail:
            "Illustrative finding: ownership moves to a different executor after the proxy upgrade.",
        },
        {
          severity: "info",
          title: "Delegatecall in execution path",
          detail:
            "Illustrative finding: implementation code executes in the proxy storage context.",
        },
      ],
      revert: null,
      evidence: {
        chainId: request.chainId,
        blockNumber: null,
        blockHash: null,
        payloadHash,
        preStateCommitment: null,
        postStateCommitment: null,
        timestamp: new Date().toISOString(),
        attestation: null,
        rpcProvenance: null,
      },
    };
  },
};

// ---------------------------------------------------------------------------
// Live client (calls the real backend; never falls back to demo on failure)
// ---------------------------------------------------------------------------

const LIVE_API_URL =
  process.env.NEXT_PUBLIC_RUSTAG_API_URL || "http://localhost:9000";

export const liveClient: RehearsalClient = {
  async rehearse(request: RehearsalRequest): Promise<RehearsalResult> {
    const response = await fetch(`${LIVE_API_URL}/api/evm/rehearse`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    if (!response.ok) {
      const text = await response.text().catch(() => response.statusText);
      throw new Error(`Rehearsal backend error ${response.status}: ${text}`);
    }
    return (await response.json()) as RehearsalResult;
  },
};

// ---------------------------------------------------------------------------
// Active client — determined by NEXT_PUBLIC_RUSTAG_REHEARSAL_MODE
// ---------------------------------------------------------------------------

export const REHEARSAL_MODE: "demo" | "live" =
  process.env.NEXT_PUBLIC_RUSTAG_REHEARSAL_MODE === "live" ? "live" : "demo";

export const rehearsalClient: RehearsalClient =
  REHEARSAL_MODE === "live" ? liveClient : demoClient;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/** Download a rehearsal result as a JSON file. */
export function downloadEvidence(result: RehearsalResult): void {
  const blob = new Blob([JSON.stringify(result, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `rustag-${result.mode}-evidence.json`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
