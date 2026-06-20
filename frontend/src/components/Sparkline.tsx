interface Props {
  values: number[];
  width?: number;
  height?: number;
}

/** A tiny inline-SVG trend line — no chart library. Renders nothing until there
 * are at least two points. Color is inherited via `currentColor`. */
export function Sparkline({ values, width = 130, height = 34 }: Props) {
  if (values.length < 2) return null;

  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const stepX = width / (values.length - 1);
  const y = (v: number) => height - ((v - min) / range) * (height - 4) - 2;

  const d = values
    .map((v, i) => `${i === 0 ? "M" : "L"} ${(i * stepX).toFixed(1)} ${y(v).toFixed(1)}`)
    .join(" ");

  const lastX = (values.length - 1) * stepX;
  const lastY = y(values[values.length - 1]);

  return (
    <svg
      className="spark"
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <path d={d} fill="none" stroke="currentColor" strokeWidth="1.5" />
      <circle cx={lastX.toFixed(1)} cy={lastY.toFixed(1)} r="2" fill="currentColor" />
    </svg>
  );
}
