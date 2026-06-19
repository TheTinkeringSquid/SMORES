// Display-only conversions and formatting. The API/bus stays in canonical SI
// units (kPa, °C); psi/relative-time live here in the frontend.

export const kpaToPsi = (kpa: number): number => kpa * 0.145038;

export function fmt(n: number | undefined | null, digits = 1): string {
  return n === undefined || n === null ? "—" : n.toFixed(digits);
}

export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const secs = Math.max(0, Math.round((Date.now() - new Date(iso).getTime()) / 1000));
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.round(secs / 60);
  if (mins < 60) return `${mins}m ago`;
  return `${Math.round(mins / 60)}h ago`;
}
