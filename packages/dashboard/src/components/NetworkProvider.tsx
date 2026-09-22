"use client";
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { chainFor, defaultNetwork, type Network } from "@/lib/chains/robinhood";
import { readNetwork } from "@/lib/evm/rpc";
import { switchNetwork, walletAddress } from "@/lib/evm/wallet";
import type { RehearsalResult } from "@/lib/rehearsal/types";

function useNetworkState() {
  const [network, setNetwork] = useState<Network>(defaultNetwork);
  const [address, setAddress] = useState<string | undefined>(undefined);
  const [walletChain, setWalletChain] = useState<number | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState<{ state: "loading" | "connected" | "error"; block?: string }>({ state: "loading" });
  const [result, setResult] = useState<RehearsalResult | undefined>(undefined);
  const connected = useRef(false);
  const chain = chainFor(network);

  useEffect(() => {
    let active = true;
    setStatus({ state: "loading" });
    async function refresh() {
      if (typeof document !== "undefined" && document.visibilityState === "hidden") return;
      try {
        const info = await readNetwork(network);
        if (active) setStatus({ state: "connected", block: info.block });
      } catch {
        if (active) setStatus({ state: "error" });
      }
    }
    void refresh();
    const timer = setInterval(refresh, 30_000);
    return () => { active = false; clearInterval(timer); };
  }, [network]);

  useEffect(() => {
    const wallet = typeof window !== "undefined" ? window.ethereum : undefined;
    const onAccounts = (...args: unknown[]) => { if (connected.current) setAddress(walletAddress(args[0])); };
    const onChainChanged = (...args: unknown[]) => { setWalletChain(parseInt(String(args[0]), 16)); };
    const onDisconnect = () => { connected.current = false; setAddress(undefined); setWalletChain(undefined); };
    wallet?.on?.("accountsChanged", onAccounts);
    wallet?.on?.("chainChanged", onChainChanged);
    wallet?.on?.("disconnect", onDisconnect);
    return () => {
      wallet?.removeListener?.("accountsChanged", onAccounts);
      wallet?.removeListener?.("chainChanged", onChainChanged);
      wallet?.removeListener?.("disconnect", onDisconnect);
    };
  }, []);

  async function connect() {
    setError(""); setBusy(true);
    try {
      if (!window.ethereum) throw new Error("No browser wallet detected. Install MetaMask, Rabby, or another EIP-1193 wallet extension.");
      const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });
      const chainIdHex = await window.ethereum.request({ method: "eth_chainId" });
      const account = walletAddress(accounts);
      if (!account) throw new Error("The wallet did not share an account.");
      connected.current = true;
      setAddress(account);
      setWalletChain(parseInt(String(chainIdHex), 16));
    } catch (e) {
      const code = (e as { code?: number }).code;
      setError(code === 4001 ? "Wallet connection was declined. You can try again." : e instanceof Error ? e.message : "Could not connect wallet.");
    } finally { setBusy(false); }
  }

  function disconnect() { connected.current = false; setAddress(undefined); setWalletChain(undefined); setError(""); }

  async function switchWallet() {
    setBusy(true); setError("");
    try {
      if (!window.ethereum) throw new Error("Browser wallet unavailable.");
      await switchNetwork(window.ethereum, network);
      const chainIdHex = await window.ethereum.request({ method: "eth_chainId" });
      setWalletChain(parseInt(String(chainIdHex), 16));
    } catch { setError("Network switch was declined or unavailable. Try again in your wallet."); }
    finally { setBusy(false); }
  }

  return { network, setNetwork, chain, address, walletChain, busy, error, status, connect, disconnect, switchWallet, result, setResult };
}

type NetworkState = ReturnType<typeof useNetworkState>;
const Context = createContext<NetworkState | null>(null);
export function NetworkProvider({ children }: { children: ReactNode }) {
  return <Context.Provider value={useNetworkState()}>{children}</Context.Provider>;
}
export function useNetwork(): NetworkState {
  const state = useContext(Context);
  if (!state) throw new Error("useNetwork must be used inside <NetworkProvider>");
  return state;
}
