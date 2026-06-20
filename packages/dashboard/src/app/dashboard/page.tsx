"use client";

import { useEffect } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";

export default function DashboardRedirect() {
  const router = useRouter();
  useEffect(() => {
    router.replace("/app");
  }, [router]);

  return (
    <div className="grid min-h-screen place-items-center text-sm text-muted">
      Redirecting to{" "}
      <Link href="/app" className="ml-1 text-brand">
        /app
      </Link>
    </div>
  );
}
