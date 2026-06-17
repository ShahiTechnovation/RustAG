"use client";

import { TxFeed } from "@/components/TxFeed";

export default function TransactionsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold text-zinc-50">Transactions</h1>
        <p className="mt-1 text-sm text-zinc-500">Live transaction feed — compute units, programs, status.</p>
      </div>
      <TxFeed limit={100} />
    </div>
  );
}
