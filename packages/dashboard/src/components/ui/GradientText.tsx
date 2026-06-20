import type { ElementType, ReactNode } from "react";

import { cn } from "@/lib/cn";

/** Renders its children with the brand gradient clipped to the text. */
export function GradientText({
  as: Tag = "span",
  className,
  children,
}: {
  as?: ElementType;
  className?: string;
  children: ReactNode;
}) {
  return <Tag className={cn("text-gradient", className)}>{children}</Tag>;
}
