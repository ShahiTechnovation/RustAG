import type {
  AccountInfo,
  AirdropResult,
  CreateScheduleParams,
  Metrics,
  OverrideParams,
  PreloadResult,
  RustagClientOptions,
  ScenarioReport,
  Schedule,
  StagenetInfo,
  TransactionInfo,
} from "./types";

const DEFAULT_BASE_URL = "http://localhost:9000";

/**
 * Client for a running RustAG stagenet's REST API.
 *
 * ```ts
 * const client = new RustagClient({ baseUrl: "http://localhost:9000" });
 * const stagenet = await client.getStagenet();
 * await client.airdrop(wallet, 1000);
 *
 * // Drop-in Solana connection against the stagenet:
 * import { Connection } from "@solana/web3.js";
 * const connection = new Connection(stagenet.rpcUrl);
 * ```
 */
export class RustagClient {
  readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: RustagClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? DEFAULT_BASE_URL).replace(/\/+$/, "");
    const resolvedFetch = options.fetch ?? globalThis.fetch;
    if (typeof resolvedFetch !== "function") {
      throw new Error("No fetch implementation available; pass one via options.fetch");
    }
    // The browser's native `fetch` throws "Illegal invocation" unless called with
    // `this === window`. Calling `this.fetchImpl(...)` would rebind `this` to the
    // client instance, so bind the default global fetch to the global object. A
    // caller-supplied fetch is used as-is (it may legitimately rely on its own `this`).
    this.fetchImpl = options.fetch ? resolvedFetch : resolvedFetch.bind(globalThis);
  }

  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const res = await this.fetchImpl(`${this.baseUrl}${path}`, {
      ...init,
      headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
    });
    if (!res.ok) {
      const body = await res.text().catch(() => "");
      throw new Error(`RustAG API ${res.status} ${res.statusText}: ${body}`);
    }
    return (await res.json()) as T;
  }

  /** Liveness check. */
  async health(): Promise<{ status: string }> {
    return this.request("/api/health");
  }

  /** Summary of the running stagenet (id, ports, counts). */
  async getStagenet(): Promise<StagenetInfo> {
    return this.request("/api/stagenet");
  }

  /** List accounts, newest-touched first. */
  async listAccounts(params: { limit?: number; offset?: number } = {}): Promise<AccountInfo[]> {
    const query = new URLSearchParams();
    if (params.limit != null) query.set("limit", String(params.limit));
    if (params.offset != null) query.set("offset", String(params.offset));
    const suffix = query.toString() ? `?${query}` : "";
    const data = await this.request<{ accounts: AccountInfo[] }>(`/api/accounts${suffix}`);
    return data.accounts;
  }

  /** Fetch a single account (lazily mirrored from mainnet if needed). */
  async getAccount(pubkey: string): Promise<AccountInfo> {
    return this.request(`/api/accounts/${pubkey}`);
  }

  /** Recent transactions, newest first. */
  async listTransactions(params: { limit?: number } = {}): Promise<TransactionInfo[]> {
    const query = new URLSearchParams();
    if (params.limit != null) query.set("limit", String(params.limit));
    const suffix = query.toString() ? `?${query}` : "";
    const data = await this.request<{ transactions: TransactionInfo[] }>(`/api/transactions${suffix}`);
    return data.transactions;
  }

  /** Airdrop SOL to a wallet. Unlimited, instant, free. */
  async airdrop(pubkey: string, sol: number): Promise<AirdropResult> {
    return this.request("/api/airdrop", {
      method: "POST",
      body: JSON.stringify({ pubkey, sol }),
    });
  }

  /** Override account state (lamports and/or SPL token amount). */
  async overrideAccount(params: OverrideParams): Promise<{ ok: boolean }> {
    return this.request("/api/override", {
      method: "POST",
      body: JSON.stringify({
        pubkey: params.pubkey,
        lamports: params.lamports,
        tokenBalance: params.tokenBalance,
      }),
    });
  }

  /** Preload known mainnet programs/oracles (e.g. `["jupiter", "pyth"]`). */
  async preload(programs: string[]): Promise<PreloadResult> {
    return this.request("/api/preload", {
      method: "POST",
      body: JSON.stringify({ programs }),
    });
  }

  // --- Phase 2: Activity Scheduler -----------------------------------------

  /** List all recurring activities. */
  async listSchedules(): Promise<Schedule[]> {
    const data = await this.request<{ schedules: Schedule[] }>("/api/schedules");
    return data.schedules;
  }

  /** Create a recurring activity (interval or cron). */
  async createSchedule(params: CreateScheduleParams): Promise<Schedule> {
    return this.request("/api/schedules", {
      method: "POST",
      body: JSON.stringify(params),
    });
  }

  /** Remove an activity by id. */
  async deleteSchedule(id: string): Promise<{ ok: boolean }> {
    return this.request(`/api/schedules/${id}`, { method: "DELETE" });
  }

  /** Enable or disable an activity. */
  async toggleSchedule(id: string, enabled: boolean): Promise<{ ok: boolean; enabled: boolean }> {
    return this.request(`/api/schedules/${id}/toggle`, {
      method: "POST",
      body: JSON.stringify({ enabled }),
    });
  }

  // --- Phase 2: Analytics --------------------------------------------------

  /** Fetch analytics time-series. Pass a `series` to restrict to one. */
  async getMetrics(params: { series?: string; limit?: number } = {}): Promise<Metrics> {
    const query = new URLSearchParams();
    if (params.series) query.set("series", params.series);
    if (params.limit != null) query.set("limit", String(params.limit));
    const suffix = query.toString() ? `?${query}` : "";
    const data = await this.request<{ metrics: Metrics }>(`/api/metrics${suffix}`);
    return data.metrics;
  }

  // --- Phase 2: Simulation -------------------------------------------------

  /**
   * Replay signed transactions against an isolated fork of the stagenet and
   * return per-transaction outcomes plus aggregate statistics. The base
   * stagenet is never mutated.
   *
   * @param transactions base64 (default) or base58 encoded signed transactions.
   */
  async simulate(
    transactions: string[],
    options: { label?: string; encoding?: "base64" | "base58" } = {},
  ): Promise<ScenarioReport> {
    return this.request("/api/simulate", {
      method: "POST",
      body: JSON.stringify({ transactions, label: options.label, encoding: options.encoding }),
    });
  }
}
