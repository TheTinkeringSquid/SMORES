// API client + data hooks. TanStack Query handles polling, caching, request
// dedup, and connection/error state — the things a vehicle dashboard on flaky
// connectivity needs — instead of hand-rolled useEffect per card.

import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import type { HistoryPoint } from "./types";

const BASE: string =
  import.meta.env.VITE_API_BASE ?? "http://localhost:8080/api/v1";

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { signal });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as T;
}

/** Poll a JSON endpoint every 3s. Aborts in-flight requests on refetch/unmount. */
export function useApi<T>(key: string, path: string): UseQueryResult<T, Error> {
  return useQuery<T, Error>({
    queryKey: [key],
    queryFn: ({ signal }) => getJson<T>(path, signal),
    refetchInterval: 3000,
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
    refetchInterval: 5000,
  });
}
