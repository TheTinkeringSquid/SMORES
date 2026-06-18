import type { ReactNode } from "react";

interface Props {
  title: string;
  source?: string;
  stale?: boolean;
  loading?: boolean;
  error?: boolean;
  empty?: boolean;
  emptyText?: string;
  children?: ReactNode;
}

/** Shared card shell: title, status tags, and standardized
 * loading/error/empty/stale states so every subsystem card looks consistent. */
export function StatusCard({
  title,
  source,
  stale,
  loading,
  error,
  empty,
  emptyText = "Waiting for telemetry…",
  children,
}: Props) {
  return (
    <section className={`card${stale ? " card--stale" : ""}`}>
      <header className="card__head">
        <h2>{title}</h2>
        <div className="card__tags">
          {source && <span className="tag">{source}</span>}
          {stale && <span className="tag tag--warn">stale</span>}
        </div>
      </header>
      <div className="card__body">
        {error ? (
          <p className="muted">No data — backend unreachable</p>
        ) : loading ? (
          <p className="muted">Loading…</p>
        ) : empty ? (
          <p className="muted">{emptyText}</p>
        ) : (
          children
        )}
      </div>
    </section>
  );
}
