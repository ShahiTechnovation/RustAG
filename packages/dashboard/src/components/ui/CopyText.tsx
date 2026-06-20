"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";

import { cn } from "@/lib/cn";

/** Monospace value that copies its full text to the clipboard on click. */
export function CopyText({
  value,
  display,
  className,
  title,
}: {
  value: string;
  display: string;
  className?: string;
  title?: string;
}) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch {
      /* clipboard unavailable */
    }
  };

  return (
    <button
      onClick={copy}
      title={title ?? value}
      className={cn(
        "group inline-flex items-center gap-1.5 font-mono transition-colors hover:text-fg cursor-pointer",
        className,
      )}
    >
      {display}
      {copied ? (
        <Check size={12} className="text-accent" />
      ) : (
        <Copy size={12} className="opacity-0 transition-opacity group-hover:opacity-60" />
      )}
    </button>
  );
}
