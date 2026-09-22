"use client";

import { useState } from "react";
import { Check, Copy } from "lucide-react";

import { cn } from "@/lib/cn";

/**
 * Production code window: language/filename header, copy button, and safe
 * line-level highlighting (comments dimmed, shell prompts in lime). Deliberately
 * tokenizer-free so it never mangles real code.
 */
export function CodeBlock({
  code,
  lang = "bash",
  filename,
  className,
}: {
  code: string;
  lang?: string;
  filename?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);
  const trimmed = code.replace(/\n+$/, "");
  const lines = trimmed.split("\n");

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(trimmed);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard unavailable */
    }
  };

  return (
    <div className={cn("group/code panel my-6 overflow-hidden", className)}>
      <div className="flex items-center justify-between border-b border-white/5 bg-white/[0.015] px-3.5 py-2">
        <span className="font-mono text-[11px] uppercase tracking-[0.16em] text-faint">
          {filename ?? lang}
        </span>
        <button
          onClick={copy}
          className="inline-flex cursor-pointer items-center gap-1.5 rounded-[3px] px-2 py-1 font-mono text-[11px] text-muted transition-colors hover:bg-white/5 hover:text-fg"
          aria-label="Copy code"
        >
          {copied ? <Check size={12} className="text-brand" /> : <Copy size={12} />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="overflow-x-auto p-4 font-mono text-[12.5px] leading-relaxed">
        <code>
          {lines.map((line, i) => {
            const t = line.trimStart();
            const isComment = t.startsWith("#") || t.startsWith("//");
            const isPrompt = t.startsWith("$") || t.startsWith(">");
            return (
              <span
                key={i}
                className={cn(
                  "block",
                  isComment && "text-faint",
                  isPrompt && "text-fg",
                )}
              >
                {isPrompt ? (
                  <>
                    <span className="select-none text-brand">{line.slice(0, line.indexOf(t[0]) + 1)}</span>
                    {line.slice(line.indexOf(t[0]) + 1)}
                  </>
                ) : (
                  line || " "
                )}
              </span>
            );
          })}
        </code>
      </pre>
    </div>
  );
}
