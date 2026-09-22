import { cn } from "@/lib/cn";

/** Full-bleed film-grain overlay. Mount once near the root. */
export function Grain() {
  return <div className="grain" aria-hidden />;
}

/** Faint fine line grid — sits behind sections. */
export function GridBackground({ className }: { className?: string }) {
  return (
    <div className={cn("bg-grid pointer-events-none absolute inset-0 -z-10", className)} aria-hidden />
  );
}

/** Concentric ring motif — the authority backdrop for heroes / feature panels. */
export function RingField({ className }: { className?: string }) {
  return (
    <div className={cn("pointer-events-none absolute inset-0 -z-10 overflow-hidden", className)} aria-hidden>
      <div className="bg-rings absolute inset-0" />
    </div>
  );
}

/**
 * Ambient backdrop. Kept deliberately flat/restrained: a single very faint lime
 * wash plus the ring motif. (Name retained for compatibility with callers.)
 */
export function AuroraBackground({
  className,
  intensity = 1,
}: {
  className?: string;
  intensity?: number;
}) {
  return (
    <div className={cn("pointer-events-none absolute inset-0 -z-10 overflow-hidden", className)} aria-hidden>
      <div className="bg-rings absolute inset-0 opacity-70" />
      <div
        className="absolute left-1/2 top-[-30%] size-[60vw] -translate-x-1/2 rounded-full blur-[140px]"
        style={{
          background: "radial-gradient(circle, var(--brand), transparent 68%)",
          opacity: 0.08 * intensity,
        }}
      />
    </div>
  );
}
