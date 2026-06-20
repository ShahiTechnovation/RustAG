import type { MetricPoint } from "@rustag/sdk";

/**
 * A dependency-free inline-SVG sparkline. Renders a metric series as a smooth
 * area chart - no charting library, so the dashboard stays lightweight.
 */
export function Sparkline({
  points,
  width = 520,
  height = 96,
  stroke = "#818cf8",
}: {
  points: MetricPoint[];
  width?: number;
  height?: number;
  stroke?: string;
}) {
  if (points.length === 0) {
    return (
      <div className="flex h-24 items-center justify-center text-xs text-zinc-600">
        no data yet
      </div>
    );
  }

  const values = points.map((p) => p.v);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const pad = 4;
  const stepX = points.length > 1 ? (width - pad * 2) / (points.length - 1) : 0;

  const xy = (v: number, i: number): [number, number] => {
    const x = pad + i * stepX;
    const y = height - pad - ((v - min) / span) * (height - pad * 2);
    return [x, y];
  };

  const line = values.map((v, i) => xy(v, i).join(",")).join(" ");
  const [, firstY] = xy(values[0], 0);
  const [lastX] = xy(values[values.length - 1], values.length - 1);
  const area = `${pad},${height - pad} ${line} ${lastX},${height - pad}`;
  void firstY;

  return (
    <svg width="100%" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" className="block">
      <polygon points={area} fill={stroke} fillOpacity={0.12} />
      <polyline points={line} fill="none" stroke={stroke} strokeWidth={1.75} strokeLinejoin="round" />
    </svg>
  );
}
