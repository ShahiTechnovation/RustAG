import type { ReactNode } from "react";

import { AppShell } from "@/components/AppShell";
import { RingField } from "@/components/ui";

export default function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="relative">
      <div className="pointer-events-none fixed inset-0 -z-10 overflow-hidden">
        <RingField />
      </div>
      <AppShell>{children}</AppShell>
    </div>
  );
}
