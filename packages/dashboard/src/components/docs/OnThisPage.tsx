"use client";

import { useEffect, useState } from "react";

import { cn } from "@/lib/cn";

export type TocItem = { id: string; title: string; depth?: 2 | 3 };

/** Sticky "On this page" rail with IntersectionObserver scroll-spy. */
export function OnThisPage({ items }: { items: TocItem[] }) {
  const [active, setActive] = useState<string | undefined>(items[0]?.id);

  useEffect(() => {
    if (!items.length) return;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((e) => e.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible[0]) setActive(visible[0].target.id);
      },
      { rootMargin: "-88px 0px -68% 0px", threshold: 0 },
    );
    items.forEach((i) => {
      const el = document.getElementById(i.id);
      if (el) observer.observe(el);
    });
    return () => observer.disconnect();
  }, [items]);

  if (!items.length) return null;

  return (
    <nav aria-label="On this page" className="sticky top-20 text-sm">
      <div className="label mb-3 text-faint">On this page</div>
      <ul className="space-y-1 border-l border-border">
        {items.map((i) => (
          <li key={i.id}>
            <a
              href={`#${i.id}`}
              className={cn(
                "-ml-px block border-l py-1 leading-snug transition-colors",
                i.depth === 3 ? "pl-7" : "pl-4",
                active === i.id
                  ? "border-brand text-fg"
                  : "border-transparent text-faint hover:text-muted",
              )}
            >
              {i.title}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}
