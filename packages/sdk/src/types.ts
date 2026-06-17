/** Sync state of an account in a stagenet. */
export type SyncState = "Unknown" | "Clean" | "Dirty" | "Pinned";

/** Coarse classification of a known account. */
export type AccountCategory = "Oracle" | "Program" | "TokenMint" | "Data";

/** Summary of a running stagenet. */
export interface StagenetInfo {
  id: string;
  name: string;
  network: string;
  slot: number;
  rpcUrl: string;
  wsUrl: string;
  mirrorEnabled: boolean;
  mainnetRpc: string;
  accounts: number;
  transactions: number;
  dirtyAccounts: number;
}

/** A single account record. */
export interface AccountInfo {
  pubkey: string;
  lamports: number;
  sol: number;
  owner: string;
  executable: boolean;
  rentEpoch: number;
  dataLen: number;
  dataBase64: string;
  syncState: SyncState;
  category: AccountCategory | null;
}

/** A single transaction record. */
export interface TransactionInfo {
  signature: string;
  slot: number;
  success: boolean;
  fee: number;
  computeUnits: number | null;
  programs: string[];
  logs: string[];
  err: string | null;
  createdAt: string;
}

export interface AirdropResult {
  signature: string;
  lamports: number;
}

export interface PreloadResult {
  loaded: number;
  unknown: string[];
}

export interface OverrideParams {
  pubkey: string;
  /** Set the lamport balance. */
  lamports?: number;
  /** Set an SPL token account's amount (raw, not UI). */
  tokenBalance?: number;
}

export interface RustagClientOptions {
  /** Base URL of the stagenet REST API (default `http://localhost:9000`). */
  baseUrl?: string;
  /** Optional custom fetch implementation. */
  fetch?: typeof fetch;
}
