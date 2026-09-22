"use client";

import { TxFeed } from "@/components/TxFeed";

export default function TransactionsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Transactions</h1>
        <p className="mt-1 text-sm text-muted">
          Live transaction feed - compute units, programs, status.
        </p>
      </div>
      <TxFeed limit={100} />
    </div>
  );
}
