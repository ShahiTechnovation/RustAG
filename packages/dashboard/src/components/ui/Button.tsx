import type { ButtonHTMLAttributes, ReactNode } from "react";
import Link from "next/link";

import { cn } from "@/lib/cn";

type Variant = "primary" | "secondary" | "ghost" | "outline";
type Size = "sm" | "md" | "lg";

const VARIANTS: Record<Variant, string> = {
  primary: "bg-brand text-[#0a0a0a] hover:bg-brand-strong",
  secondary: "border border-border-strong text-fg hover:border-brand hover:text-brand",
  outline: "border border-border-strong text-fg hover:border-brand hover:text-brand",
  ghost: "text-muted hover:text-fg hover:bg-white/5",
};

const SIZES: Record<Size, string> = {
  sm: "h-9 px-4 text-[11px] gap-1.5",
  md: "h-11 px-5 text-xs gap-2",
  lg: "h-12 px-6 text-[13px] gap-2",
};

type CommonProps = {
  variant?: Variant;
  size?: Size;
  className?: string;
  children: ReactNode;
};

const base =
  "inline-flex items-center justify-center rounded-[3px] font-semibold uppercase tracking-[0.12em] whitespace-nowrap transition-colors duration-200 disabled:opacity-40 disabled:pointer-events-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ring)] focus-visible:ring-offset-2 focus-visible:ring-offset-bg cursor-pointer";

export function Button({
  variant = "primary",
  size = "md",
  className,
  children,
  ...props
}: CommonProps & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button className={cn(base, VARIANTS[variant], SIZES[size], className)} {...props}>
      {children}
    </button>
  );
}

/** Link styled as a button (for navigation / CTAs). */
export function ButtonLink({
  href,
  variant = "primary",
  size = "md",
  className,
  children,
  external,
}: CommonProps & { href: string; external?: boolean }) {
  const classes = cn(base, VARIANTS[variant], SIZES[size], className);
  if (external) {
    return (
      <a href={href} target="_blank" rel="noreferrer" className={classes}>
        {children}
      </a>
    );
  }
  return (
    <Link href={href} className={classes}>
      {children}
    </Link>
  );
}
