import type { ReactNode } from "react";

import { Footer } from "@/components/Footer";
import { SiteHeader } from "@/components/SiteHeader";

export default function MarketingLayout({ children }: { children: ReactNode }) {
  return (
    <>
      <SiteHeader />
      <main>{children}</main>
      <Footer />
    </>
  );
}
