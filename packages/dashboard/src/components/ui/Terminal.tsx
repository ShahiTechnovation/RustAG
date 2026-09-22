"use client";

import type { ReactNode } from "react";
import { useState } from "react";
import { Check, Copy } from "lucide-react";

import { cn } from "@/lib/cn";

/** Glass terminal/code window with traffic lights + copy button. */
export function Terminal({
  title = "rustag",
  copyText,
  children,
  className,
}: {
  title?: string;
  copyText?: string;
  children: ReactNode;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    if (!copyText) return;
    try {
      await navigator.clipboard.writeText(copyText);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      /* clipboard unavailable */
    }
  };

  return (
    <div className={cn("panel overflow-hidden", className)}>
      <div className="flex items-center justify-between border-b border-white/5 px-4 py-3">
        <div className="flex items-center gap-2">
          <span className="size-3 rounded-full bg-[#ff5f57]" />
          <span className="size-3 rounded-full bg-[#febc2e]" />
          <span className="size-3 rounded-full bg-[#28c840]" />
        </div>
        <span className="font-mono text-xs text-faint">{title}</span>
        {copyText ? (
          <button
            onClick={copy}
            className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs text-muted transition-colors hover:bg-white/5 hover:text-fg cursor-pointer"
          >
            {copied ? <Check size={13} className="text-accent" /> : <Copy size={13} />}
            {copied ? "Copied" : "Copy"}
          </button>
        ) : (
          <span className="w-12" />
        )}
      </div>
      <div className="overflow-x-auto p-5 font-mono text-[13px] leading-relaxed">{children}</div>
    </div>
  );
}
