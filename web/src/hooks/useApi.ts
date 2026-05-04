import { useCallback, useEffect, useRef, useState } from "react";
import { KnowNowClient } from "../api/client";

let client: KnowNowClient | null = null;

function getClient(): KnowNowClient {
  if (!client) {
    client = new KnowNowClient(window.location.origin);
  }
  return client;
}

export function useApi<T>(fetcher: (client: KnowNowClient) => Promise<T>) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [loading, setLoading] = useState(true);
  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    fetcherRef.current(getClient())
      .then(setData)
      .catch(setError)
      .finally(() => { setLoading(false); });
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  return { data, error, loading, reload: load };
}
