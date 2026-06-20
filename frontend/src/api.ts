// API client + data hooks. TanStack Query handles polling, caching, request
// dedup, and connection/error state — the things a vehicle dashboard on flaky
// connectivity needs — instead of hand-rolled useEffect per card.

import { useEffect } from "react";
import {
  useQuery,
  useQueryClient,
  type UseQueryResult,
} from "@tanstack/react-query";
import type { HistoryPoint } from "./types";

const BASE: string =
  import.meta.env.VITE_API_BASE ?? "http://localhost:8080/api/v1";

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { signal });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as T;
}

/** Fetch a JSON endpoint. Live updates arrive via SSE (see useLiveUpdates); the
 *  15s interval is just a fallback if the stream drops. Aborts in-flight
 *  requests on refetch/unmount. */
export function useApi<T>(key: string, path: string): UseQueryResult<T, Error> {
  return useQuery<T, Error>({
    queryKey: [key],
    queryFn: ({ signal }) => getJson<T>(path, signal),
    refetchInterval: 15000,
  });
}

/** Trend history for one subsystem+metric, polled for the sparklines. */
export function useHistory(
  subsystem: string,
  metric: string,
  limit = 60,
): UseQueryResult<HistoryPoint[], Error> {
  const path = `/history?subsystem=${encodeURIComponent(subsystem)}&metric=${encodeURIComponent(metric)}&limit=${limit}`;
  return useQuery<HistoryPoint[], Error>({
    queryKey: ["history", subsystem, metric],
    queryFn: ({ signal }) => getJson<HistoryPoint[]>(path, signal),
    refetchInterval: 15000,
  });
}

/** Subscribe to the backend SSE stream and invalidate the matching queries on
 *  each `update` event, so cards refetch the instant data changes. The polling
 *  interval above is the fallback if the stream is unavailable. */
export function useLiveUpdates(): void {
  const qc = useQueryClient();
  useEffect(() => {
    const es = new EventSource(`${BASE}/stream`);
    es.addEventListener("update", (e) => {
      const key = (e as MessageEvent).data;
      qc.invalidateQueries({ queryKey: [key] });
      qc.invalidateQueries({ queryKey: ["health"] });
      if (key === "battery" || key === "tanks") {
        qc.invalidateQueries({ queryKey: ["history"] });
      }
    });
    return () => es.close();
  }, [qc]);
}
