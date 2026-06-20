import type { ReactNode } from "react";

import { cn } from "@/lib/cn";

/** A landing-page section with consistent vertical rhythm + optional header. */
export function Section({
  id,
  eyebrow,
  title,
  description,
  children,
  className,
  containerClassName,
  align = "center",
}: {
  id?: string;
  eyebrow?: string;
  title?: ReactNode;
  description?: ReactNode;
  children: ReactNode;
  className?: string;
  containerClassName?: string;
  align?: "center" | "left";
}) {
  const centered = align === "center";
  return (
    <section
      id={id}
      className={cn("relative scroll-mt-24 px-6 py-20 sm:py-28 [content-visibility:auto]", className)}
    >
      <div className={cn("mx-auto w-full max-w-6xl", containerClassName)}>
        {(eyebrow || title || description) && (
          <div className={cn("mb-12 max-w-2xl", centered ? "mx-auto text-center" : "text-left")}>
            {eyebrow ? (
              <div className={cn("label mb-4 text-brand", centered && "justify-center")}>{eyebrow}</div>
            ) : null}
            {title ? (
              <h2 className="text-balance font-display text-4xl font-semibold tracking-tight text-fg sm:text-5xl">
                {title}
              </h2>
            ) : null}
            {description ? (
              <p className="mt-4 text-pretty text-base leading-relaxed text-muted">{description}</p>
            ) : null}
          </div>
        )}
        {children}
      </div>
    </section>
  );
}
