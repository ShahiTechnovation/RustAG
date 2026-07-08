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
  /** True when the server runs as a public, capped-interactive demo: reads,
   * capped airdrops, and simulate are live; override/preload/schedule writes
   * are disabled. Absent on older backends. */
  demoMode?: boolean;
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

// --- Phase 2: Activity Scheduler -------------------------------------------

/** What a scheduled activity does when it fires. */
export type ScheduleAction =
  | { type: "airdrop"; pubkey: string; sol: number }
  | { type: "transfer"; secret_key: string; to: string; sol: number }
  | { type: "raw_transaction"; transaction_base64: string };

/** A recurring on-chain activity. */
export interface Schedule {
  id: string;
  name: string;
  /** `@every 30s`, `@hourly`, or a 5-field cron expression. */
  schedule: string;
  action: ScheduleAction;
  enabled: boolean;
  runCount: number;
  lastRun: string | null;
  lastStatus: string | null;
  lastSignature: string | null;
  createdAt: string;
}

export interface CreateScheduleParams {
  name: string;
  schedule: string;
  action: ScheduleAction;
}

// --- Phase 2: Analytics ----------------------------------------------------

/** A single time-series point: `t` is an ISO-8601 timestamp, `v` the value. */
export interface MetricPoint {
  t: string;
  v: number;
}

/** Metrics keyed by series name (e.g. `tvl_lamports`, `transactions`). */
export type Metrics = Record<string, MetricPoint[]>;

// --- Phase 2: Simulation ---------------------------------------------------

export interface SimTxResult {
  index: number;
  signature: string;
  success: boolean;
  err: string | null;
  computeUnits: number;
  fee: number;
}

/** The result of replaying a set of transactions against a fork. */
export interface ScenarioReport {
  label: string;
  total: number;
  succeeded: number;
  failed: number;
  totalComputeUnits: number;
  maxComputeUnits: number;
  totalFees: number;
  durationMs: number;
  outcomes: SimTxResult[];
}
