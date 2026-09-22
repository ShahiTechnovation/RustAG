import type { ReactNode } from "react";

import { Eyebrow } from "@/components/ui";
import { OnThisPage, type TocItem } from "./OnThisPage";
import { Pagination } from "./Pagination";

/** Standard docs content page: header + prose body + sticky TOC + pagination. */
export function DocArticle({
  eyebrow,
  title,
  lead,
  toc = [],
  children,
}: {
  eyebrow?: string;
  title: string;
  lead?: ReactNode;
  toc?: TocItem[];
  children: ReactNode;
}) {
  return (
    <div className="mx-auto flex max-w-6xl gap-12 px-5 py-10 sm:px-8 lg:py-14">
      <article className="min-w-0 max-w-3xl flex-1">
        <header className="border-b border-border pb-8">
          {eyebrow ? <Eyebrow className="text-brand">{eyebrow}</Eyebrow> : null}
          <h1 className="mt-4 text-balance font-display text-4xl font-bold tracking-tight text-fg sm:text-[2.9rem] sm:leading-[1.05]">
            {title}
          </h1>
          {lead ? <p className="mt-5 text-pretty text-lg leading-relaxed text-muted">{lead}</p> : null}
        </header>

        <div className="doc-content mt-8">{children}</div>

        <Pagination />
      </article>

      {toc.length ? (
        <aside className="hidden w-56 shrink-0 xl:block">
          <OnThisPage items={toc} />
        </aside>
      ) : null}
    </div>
  );
}
