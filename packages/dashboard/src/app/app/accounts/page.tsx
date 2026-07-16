"use client";

import { AccountsTable } from "@/components/AccountsTable";

export default function AccountsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-3xl font-bold tracking-tight text-fg">Accounts</h1>
        <p className="mt-1 text-sm text-muted">
          Every account in the pre-state closure — Clean (pinned from mainnet), Dirty (modified by rehearsal), Pinned (overridden).
        </p>
      </div>
      <AccountsTable />
    </div>
  );
}
