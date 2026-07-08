"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import { PYTH_FEEDS, decodePythPrice } from "./pyth";

export interface OraclePrice {
  symbol: string;
  pubkey: string;
  price: number | null;
  syncState: string | null;
}

/** Live Pyth prices, decoded in the browser from the mirrored account bytes. */
export function useOraclePrices() {
  return useQuery({
    queryKey: ["oracle-prices"],
    queryFn: async (): Promise<OraclePrice[]> =>
      Promise.all(
        PYTH_FEEDS.map(async (feed) => {
          try {
            const account = await client.getAccount(feed.pubkey);
            return {
              symbol: feed.symbol,
              pubkey: feed.pubkey,
              price: decodePythPrice(account.dataBase64),
              syncState: account.syncState,
            };
          } catch {
            return { symbol: feed.symbol, pubkey: feed.pubkey, price: null, syncState: null };
          }
        }),
      ),
    refetchInterval: 10000,
  });
}

export function useStagenet() {
  return useQuery({
    queryKey: ["stagenet"],
    queryFn: () => client.getStagenet(),
    refetchInterval: 5000,
  });
}

export function useAccounts(limit = 200) {
  return useQuery({
    queryKey: ["accounts", limit],
    queryFn: () => client.listAccounts({ limit }),
    refetchInterval: 5000,
  });
}

export function useTransactions(limit = 50) {
  return useQuery({
    queryKey: ["transactions", limit],
    queryFn: () => client.listTransactions({ limit }),
    refetchInterval: 3000,
  });
}

function useInvalidate() {
  const qc = useQueryClient();
  return () => {
    void qc.invalidateQueries({ queryKey: ["stagenet"] });
    void qc.invalidateQueries({ queryKey: ["accounts"] });
    void qc.invalidateQueries({ queryKey: ["transactions"] });
  };
}

export function useAirdrop() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (vars: { pubkey: string; sol: number }) => client.airdrop(vars.pubkey, vars.sol),
    onSuccess: invalidate,
  });
}

export function useOverride() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (vars: { pubkey: string; lamports?: number; tokenBalance?: number }) =>
      client.overrideAccount(vars),
    onSuccess: invalidate,
  });
}

export function usePreload() {
  const invalidate = useInvalidate();
  return useMutation({
    mutationFn: (programs: string[]) => client.preload(programs),
    onSuccess: invalidate,
  });
}

// --- Phase 2: analytics + scheduler ----------------------------------------

export function useMetrics(limit = 60) {
  return useQuery({
    queryKey: ["metrics", limit],
    queryFn: () => client.getMetrics({ limit }),
    refetchInterval: 5000,
  });
}

export function useSchedules() {
  return useQuery({
    queryKey: ["schedules"],
    queryFn: () => client.listSchedules(),
    refetchInterval: 3000,
  });
}

function useInvalidateSchedules() {
  const qc = useQueryClient();
  return () => void qc.invalidateQueries({ queryKey: ["schedules"] });
}

export function useCreateSchedule() {
  const invalidate = useInvalidateSchedules();
  return useMutation({
    mutationFn: (params: Parameters<typeof client.createSchedule>[0]) =>
      client.createSchedule(params),
    onSuccess: invalidate,
  });
}

export function useDeleteSchedule() {
  const invalidate = useInvalidateSchedules();
  return useMutation({
    mutationFn: (id: string) => client.deleteSchedule(id),
    onSuccess: invalidate,
  });
}

export function useToggleSchedule() {
  const invalidate = useInvalidateSchedules();
  return useMutation({
    mutationFn: (vars: { id: string; enabled: boolean }) =>
      client.toggleSchedule(vars.id, vars.enabled),
    onSuccess: invalidate,
  });
}
