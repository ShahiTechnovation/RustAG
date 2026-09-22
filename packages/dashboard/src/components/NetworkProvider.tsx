'use client';
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import type { Address } from 'viem';
import { chainFor, defaultNetwork, type Network } from '@/lib/chains/robinhood';
import { readNetwork } from '@/lib/evm/rpc';
import { switchNetwork, walletAddress } from '@/lib/evm/wallet';
import type { RehearsalResult } from '@/lib/rehearsal/types';

function useNetworkState() {
  const [network, setNetwork] = useState<Network>(defaultNetwork);
  const [address, setAddress] = useState<Address>();
  const [walletChain, setWalletChain] = useState<number>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState<{ state: 'loading' | 'connected' | 'error'; block?: string }>({ state: 'loading' });
  const [result, setResult] = useState<RehearsalResult>();
  const connected = useRef(false);
  const chain = chainFor(network);
  useEffect(() => {
    let active = true;
    setStatus({ state: 'loading' });
    async function refresh() {
      if (document.visibilityState === 'hidden') return;
      try { const info = await readNetwork(network); if (active) setStatus({ state: 'connected', block: info.block }); }
      catch { if (active) setStatus({ state: 'error' }); }
    }
    void refresh(); const timer = setInterval(refresh, 30000);
    return () => { active = false; clearInterval(timer); };
  }, [network]);
  useEffect(() => {
    const wallet = window.ethereum;
    const accounts = (...args: unknown[]) => { if (connected.current) setAddress(walletAddress(args[0])); };
    const chainChanged = (...args: unknown[]) => { setWalletChain(Number(args[0])); };
    const disconnected = () => { connected.current = false; setAddress(undefined); setWalletChain(undefined); };
    wallet?.on?.('accountsChanged', accounts); wallet?.on?.('chainChanged', chainChanged); wallet?.on?.('disconnect', disconnected);
    return () => { wallet?.removeListener?.('accountsChanged', accounts); wallet?.removeListener?.('chainChanged', chainChanged); wallet?.removeListener?.('disconnect', disconnected); };
  }, []);
  async function connect() {
    setError(''); setBusy(true);
    try {
      if (!window.ethereum) throw new Error('No browser wallet found. Open this site in an EVM wallet browser or install an EIP-1193 wallet.');
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
      const id = await window.ethereum.request({ method: 'eth_chainId' });
      const account = walletAddress(accounts);
      if (!account) throw new Error('The wallet did not share an account.');
      connected.current = true; setAddress(account); setWalletChain(Number(id));
    } catch (e) { setError((e as { code?: number }).code === 4001 ? 'Wallet connection was declined. You can try again.' : e instanceof Error ? e.message : 'Could not connect wallet.'); }
    finally { setBusy(false); }
  }
  function disconnect() { connected.current = false; setAddress(undefined); setWalletChain(undefined); setError(''); }
  async function switchWallet() {
    setBusy(true); setError('');
    try { if (!window.ethereum) throw new Error('Browser wallet unavailable.'); await switchNetwork(window.ethereum, network); setWalletChain(Number(await window.ethereum.request({ method: 'eth_chainId' }))); }
    catch { setError('Network switch was declined or unavailable. Try again in your wallet.'); }
    finally { setBusy(false); }
  }
  return { network, setNetwork, chain, address, walletChain, busy, error, status, connect, disconnect, switchWallet, result, setResult };
}
const Context = createContext<ReturnType<typeof useNetworkState> | null>(null);
export function NetworkProvider({ children }: { children: ReactNode }) { return <Context.Provider value={useNetworkState()}>{children}</Context.Provider>; }
export function useNetwork() { const state = useContext(Context); if (!state) throw new Error('NetworkProvider is required'); return state; }
