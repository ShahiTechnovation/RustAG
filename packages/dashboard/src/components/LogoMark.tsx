import { cn } from "@/lib/cn";

/**
 * RustAG emblem — "the mirror mark".
 *
 * A flat-top hexagon badge (authority) holding two slanted parallelogram bars
 * (a Solana-ecosystem nod) that are mirrored across a central lime axis with a
 * sync-node at the center: mainnet state, mirrored into a stagenet. Lime on
 * black, sharp, legible down to favicon size. Color follows --brand.
 */
export function LogoMark({
  size = 32,
  className,
  animated = false,
}: {
  size?: number;
  className?: string;
  animated?: boolean;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      className={cn("shrink-0", className)}
      aria-hidden
    >
      {/* hexagon badge */}
      <polygon
        points="45,24 34.5,5.8 13.5,5.8 3,24 13.5,42.2 34.5,42.2"
        fill="var(--brand)"
        fillOpacity="0.06"
        stroke="var(--brand)"
        strokeWidth="1.6"
        strokeLinejoin="round"
      />

      {/* upper bars — "mainnet", solid lime */}
      <polygon points="18,10.8 33,10.8 29,14 14,14" fill="var(--brand)" />
      <polygon points="18,16 33,16 29,19.2 14,19.2" fill="var(--brand)" />

      {/* mirror axis + sync node */}
      <line x1="12.5" y1="24" x2="35.5" y2="24" stroke="var(--brand)" strokeWidth="1" strokeOpacity="0.55" />
      <polygon points="24,21.4 26.6,24 24,26.6 21.4,24" fill="var(--brand)" />

      {/* lower bars — "stagenet" reflection, faded */}
      <g
        fill="var(--brand)"
        fillOpacity="0.32"
        className={animated ? "animate-pulse" : undefined}
      >
        <polygon points="18,37.2 33,37.2 29,34 14,34" />
        <polygon points="18,32 33,32 29,28.8 14,28.8" />
      </g>
    </svg>
  );
}
