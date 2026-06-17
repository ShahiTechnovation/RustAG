import type {
  AccountInfo,
  AirdropResult,
  OverrideParams,
  PreloadResult,
  RustagClientOptions,
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
    this.fetchImpl = options.fetch ?? globalThis.fetch;
    if (typeof this.fetchImpl !== "function") {
      throw new Error("No fetch implementation available; pass one via options.fetch");
    }
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
}
