"use client";

import { AccountsTable } from "@/components/AccountsTable";

export default function AccountsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold text-zinc-50">Accounts</h1>
        <p className="mt-1 text-sm text-zinc-500">
          Every account in the stagenet, with its mainnet-sync state.
        </p>
      </div>
      <AccountsTable />
    </div>
  );
}
