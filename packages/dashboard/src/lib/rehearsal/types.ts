// EVM rehearsal type definitions — plain TypeScript, no external dependencies.

export type EvmAddress = `0x${string}`;
export type EvmHex = `0x${string}`;
export type Severity = "critical" | "high" | "medium" | "low" | "info";

export interface RehearsalRequest {
  chainId: number;
  from?: EvmAddress;
  to: EvmAddress;
  value: string;
  data: EvmHex;
  blockNumber?: string;
}

export interface Finding {
  severity: Severity;
  title: string;
  detail: string;
}

export interface StateChange {
  address: EvmAddress;
  field: string;
  before: string;
  after: string;
  kind: "balance" | "permission" | "approval" | "storage" | "nonce";
}

export interface Call {
  address: EvmAddress;
  label: string;
  type: "CALL" | "DELEGATECALL" | "STATICCALL" | "CREATE" | "CREATE2";
  function: string;
  value: string;
  gas: number;
  success: boolean;
  data: EvmHex;
  returnData: EvmHex;
  children: Call[];
}

export interface Evidence {
  chainId: number;
  blockNumber: string | null;
  blockHash: EvmHex | null;
  payloadHash: EvmHex;
  preStateCommitment: EvmHex | null;
  postStateCommitment: EvmHex | null;
  timestamp: string;
  attestation: string | null;
  rpcProvenance: string | null;
}

export interface RehearsalResult {
  mode: "demo" | "live";
  request: RehearsalRequest;
  success: boolean;
  gas: number;
  calls: Call[];
  changes: StateChange[];
  findings: Finding[];
  revert: string | null;
  evidence: Evidence;
}

export interface RehearsalClient {
  rehearse(request: RehearsalRequest): Promise<RehearsalResult>;
}
