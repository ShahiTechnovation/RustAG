import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { RustagClient } from "@rustag/sdk";

/** Options for booting an ephemeral stagenet. */
export interface EphemeralStagenetOptions {
  /** Stagenet name (defaults to a unique `eph-...`). */
  name?: string;
  /** JSON-RPC port (defaults to a random free-ish port). */
  rpcPort?: number;
  /** WebSocket port (defaults to `rpcPort + 1`). */
  wsPort?: number;
  /** REST API port (defaults to `rpcPort + 2`). */
  apiPort?: number;
  /** Mainnet RPC endpoint (defaults to `$RUSTAG_MAINNET_RPC`). */
  mainnetRpc?: string;
  /** Disable mainnet mirroring (fully offline). */
  noMirror?: boolean;
  /** Programs/oracles to preload on startup (e.g. `["jupiter", "pyth"]`). */
  preload?: string[];
  /** Path to the `rustag` binary (defaults to `$RUSTAG_BIN` or `rustag`). */
  rustagBin?: string;
  /** Working directory for isolation (defaults to a fresh temp dir). */
  cwd?: string;
  /** How long to wait for the stagenet to become healthy (ms). */
  readyTimeoutMs?: number;
}

function randomPort(min = 9000, max = 60000): number {
  return Math.floor(Math.random() * (max - min)) + min;
}

/**
 * An ephemeral RustAG stagenet running as a child process, for use in CI and
 * Anchor tests. Boots a real stagenet (JSON-RPC + REST), waits until it is
 * healthy, and tears it down on {@link stop}.
 */
export class EphemeralStagenet {
  readonly name: string;
  readonly rpcUrl: string;
  readonly wsUrl: string;
  readonly apiUrl: string;
  /** SDK client pointed at this stagenet's REST API. */
  readonly client: RustagClient;
  private proc: ChildProcess | null = null;

  private constructor(name: string, rpcPort: number, wsPort: number, apiPort: number) {
    this.name = name;
    this.rpcUrl = `http://127.0.0.1:${rpcPort}`;
    this.wsUrl = `ws://127.0.0.1:${wsPort}`;
    this.apiUrl = `http://127.0.0.1:${apiPort}`;
    this.client = new RustagClient({ baseUrl: this.apiUrl });
  }

  /** Create and start an ephemeral stagenet, resolving once it is healthy. */
  static async start(opts: EphemeralStagenetOptions = {}): Promise<EphemeralStagenet> {
    const bin = opts.rustagBin ?? process.env.RUSTAG_BIN ?? "rustag";
    const rpcPort = opts.rpcPort ?? randomPort();
    const wsPort = opts.wsPort ?? rpcPort + 1;
    const apiPort = opts.apiPort ?? rpcPort + 2;
    const name = opts.name ?? `eph-${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
    const cwd = opts.cwd ?? mkdtempSync(join(tmpdir(), "rustag-"));
    const mainnetRpc = opts.mainnetRpc ?? process.env.RUSTAG_MAINNET_RPC;

    const createArgs = [
      "create",
      name,
      "--rpc-port",
      String(rpcPort),
      "--ws-port",
      String(wsPort),
      "--api-port",
      String(apiPort),
    ];
    if (opts.noMirror) createArgs.push("--no-mirror");
    if (mainnetRpc) createArgs.push("--mainnet-rpc", mainnetRpc);

    const created = spawnSync(bin, createArgs, { cwd, encoding: "utf8" });
    if (created.error) {
      throw new Error(
        `could not run '${bin}' - is the RustAG CLI installed / on PATH? (${created.error.message})`,
      );
    }
    if (created.status !== 0) {
      throw new Error(`'rustag create' failed: ${created.stderr || created.stdout}`);
    }

    const startArgs = ["start", name];
    if (opts.preload?.length) startArgs.push("--preload", ...opts.preload);
    const proc = spawn(bin, startArgs, { cwd, stdio: "ignore" });

    const stagenet = new EphemeralStagenet(name, rpcPort, wsPort, apiPort);
    stagenet.proc = proc;
    await stagenet.waitUntilReady(opts.readyTimeoutMs ?? 20_000);
    return stagenet;
  }

  private async waitUntilReady(timeoutMs: number): Promise<void> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      try {
        const res = await fetch(`${this.apiUrl}/api/health`);
        if (res.ok) return;
      } catch {
        // not listening yet
      }
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
    await this.stop();
    throw new Error(`stagenet '${this.name}' did not become ready within ${timeoutMs}ms`);
  }

  /** Stop the stagenet's child process. */
  async stop(): Promise<void> {
    if (this.proc && !this.proc.killed) {
      this.proc.kill();
    }
    this.proc = null;
  }
}

/** Run `fn` against a fresh ephemeral stagenet, tearing it down afterwards. */
export async function withEphemeralStagenet<T>(
  opts: EphemeralStagenetOptions,
  fn: (stagenet: EphemeralStagenet) => Promise<T>,
): Promise<T> {
  const stagenet = await EphemeralStagenet.start(opts);
  try {
    return await fn(stagenet);
  } finally {
    await stagenet.stop();
  }
}
