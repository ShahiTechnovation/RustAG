"use client";

import NumberFlow, { type Format } from "@number-flow/react";

/** Odometer-style animated number. Animates smoothly as live data updates. */
export function AnimatedNumber({
  value,
  format,
  prefix,
  suffix,
  className,
}: {
  value: number;
  format?: Format;
  prefix?: string;
  suffix?: string;
  className?: string;
}) {
  return (
    <NumberFlow
      value={value}
      format={format}
      prefix={prefix}
      suffix={suffix}
      className={className}
      willChange
    />
  );
}
