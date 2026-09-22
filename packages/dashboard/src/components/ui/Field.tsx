import type { InputHTMLAttributes, ReactNode } from "react";

import { cn } from "@/lib/cn";

/** Tokenized text input — flat, sharp, lime focus. */
export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "h-10 w-full rounded-[3px] border border-border-strong bg-subtle px-3 text-sm text-fg",
        "placeholder:text-faint transition-colors",
        "focus:border-brand focus:outline-none focus:ring-2 focus:ring-[var(--ring)]/30",
        className,
      )}
      {...props}
    />
  );
}

/** Labelled field wrapper. */
export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <span className="label mb-2 block">{label}</span>
      {children}
      {hint ? <span className="mt-1 block text-xs text-faint">{hint}</span> : null}
    </label>
  );
}
