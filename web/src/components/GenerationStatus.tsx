import { useCallback } from "react";
import type { GenerationStatusResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";
import { Card } from "./ui/Card";
import { PageHeader } from "./ui/PageHeader";

export function GenerationStatus() {
  const fetcher = useCallback((c: KnowNowClient) => c.getGenerationStatus(), []);
  const { data, loading, error } = useApi<GenerationStatusResponse>(fetcher);

  return (
    <div className="kn-page">
      <div className="kn-page__inner">
        <PageHeader
          title="Generation"
          description="Snapshot of the most recent generation run."
        />

        {loading && <div className="kn-loading" aria-live="polite">Loading generation status...</div>}
        {error && <div className="kn-error" role="alert">Failed to load status: {error.message}</div>}

        {data && (
          <Card title="Status">
            {data.has_generation ? (
              <dl className="kn-gen-status__details">
                <dt>Engine Version</dt>
                <dd><code>{data.engine_version}</code></dd>

                <dt>Artifacts</dt>
                <dd>{String(data.artifact_count)}</dd>

                {data.last_generated_epoch != null && (
                  <>
                    <dt>Last Generated</dt>
                    <dd>{formatEpoch(data.last_generated_epoch)}</dd>
                  </>
                )}

                {data.warnings.length > 0 && (
                  <>
                    <dt>Warnings</dt>
                    <dd>
                      <ul className="kn-gen-status__warnings" role="list">
                        {data.warnings.map((w, i) => (
                          <li key={i}>{w}</li>
                        ))}
                      </ul>
                    </dd>
                  </>
                )}
              </dl>
            ) : (
              <p className="kn-gen-status__empty">
                No generation has been run yet.
                {!data.generated_dir_exists && " The generated/ directory does not exist."}
              </p>
            )}
          </Card>
        )}
      </div>
    </div>
  );
}

function formatEpoch(epoch: number): string {
  return new Date(epoch * 1000).toLocaleString();
}
