import Link from "next/link";

import { cn } from "@/lib/cn";
import { LogoMark } from "./LogoMark";

/** RustAG wordmark + mirror-mark emblem. */
export function Logo({
  href = "/",
  className,
  showTag = true,
}: {
  href?: string;
  className?: string;
  showTag?: boolean;
}) {
  return (
    <Link href={href} className={cn("group flex items-center gap-2.5", className)}>
      <LogoMark size={30} className="transition-transform duration-300 group-hover:scale-105" />
      <span className="flex items-baseline gap-1.5">
        <span className="font-display text-[15px] font-bold uppercase tracking-tight text-fg">
          RustAG
        </span>
        {showTag ? (
          <span className="label text-[9px] tracking-[0.18em] text-faint">stagenet</span>
        ) : null}
      </span>
    </Link>
  );
}
