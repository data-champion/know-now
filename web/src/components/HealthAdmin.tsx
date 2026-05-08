import { useCallback } from "react";
import type {
  GenerationStatusResponse,
  ManifestResponse,
  StatusResponse,
} from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";
import { Card } from "./ui/Card";
import { PageHeader } from "./ui/PageHeader";

export function HealthAdmin() {
  const fetchStatus = useCallback((c: KnowNowClient) => c.getStatus(), []);
  const fetchGenStatus = useCallback((c: KnowNowClient) => c.getGenerationStatus(), []);
  const fetchManifest = useCallback((c: KnowNowClient) => c.getManifest(), []);

  const { data: status, loading: l1 } = useApi<StatusResponse>(fetchStatus);
  const { data: genStatus, loading: l2 } = useApi<GenerationStatusResponse>(fetchGenStatus);
  const { data: manifestResp, loading: l3 } = useApi<ManifestResponse>(fetchManifest);

  const loading = l1 || l2 || l3;
  const manifest = manifestResp?.manifest;

  return (
    <div className="kn-page">
      <div className="kn-page__inner">
        <PageHeader
          title="Health & Admin"
          description="Operational status, server metadata, and generation health."
        />

        {loading ? (
          <div className="kn-loading" aria-live="polite">Loading health data...</div>
        ) : (
          <div className="kn-health__grid">
            <Card title="Server">
              <dl className="kn-health__dl">
                <dt>Server</dt>
                <dd>{status?.server ?? "unknown"}</dd>
                <dt>Version</dt>
                <dd><code>{status?.version ?? "—"}</code></dd>
                <dt>Write Mode</dt>
                <dd>
                  <StatusBadge active={status?.write_mode ?? false} label={status?.write_mode ? "enabled" : "disabled"} />
                </dd>
              </dl>
            </Card>

            <Card title="Security">
              <dl className="kn-health__dl">
                <dt>Bind Address</dt>
                <dd><code>localhost</code></dd>
                <dt>Session</dt>
                <dd><StatusBadge active label="active" /></dd>
                <dt>Write Endpoint</dt>
                <dd>
                  <StatusBadge
                    active={status?.write_mode ?? false}
                    label={status?.write_mode ? "enabled (allow-generate)" : "disabled"}
                  />
                </dd>
              </dl>
            </Card>

            <Card title="Generation">
              <dl className="kn-health__dl">
                <dt>Status</dt>
                <dd>
                  <StatusBadge
                    active={genStatus?.has_generation ?? false}
                    label={genStatus?.has_generation ? "generated" : "none"}
                  />
                </dd>
                <dt>Engine Version</dt>
                <dd><code>{genStatus?.engine_version || "—"}</code></dd>
                <dt>Artifact Count</dt>
                <dd>{String(genStatus?.artifact_count ?? 0)}</dd>
                {genStatus?.last_generated_epoch != null && (
                  <>
                    <dt>Last Generated</dt>
                    <dd>{new Date(genStatus.last_generated_epoch * 1000).toLocaleString()}</dd>
                  </>
                )}
                {(genStatus?.warnings.length ?? 0) > 0 && (
                  <>
                    <dt>Warnings</dt>
                    <dd className="kn-health__warning">{String(genStatus?.warnings.length)} warning(s)</dd>
                  </>
                )}
              </dl>
            </Card>

            <Card title="Metadata Schema">
              <dl className="kn-health__dl">
                <dt>Schema Version</dt>
                <dd><code>{manifest?.metadata_schema_version ?? "—"}</code></dd>
                <dt>Contract Version</dt>
                <dd><code>{manifest?.generator_contract_version ?? "—"}</code></dd>
              </dl>
            </Card>

            <Card title="Target Database">
              <dl className="kn-health__dl">
                <dt>Kind</dt>
                <dd>{manifest?.target_database.kind ?? "—"}</dd>
                <dt>Version</dt>
                <dd><code>{manifest?.target_database.version ?? "—"}</code></dd>
                <dt>Compat Floor</dt>
                <dd><code>{manifest?.target_database.compatibility_floor ?? "—"}</code></dd>
              </dl>
            </Card>

            <Card title="Policy">
              <dl className="kn-health__dl">
                <dt>Pack</dt>
                <dd>{manifest?.policy.pack ?? "—"}</dd>
                <dt>Version</dt>
                <dd><code>{manifest?.policy.version ?? "—"}</code></dd>
                <dt>Hash</dt>
                <dd><code className="kn-health__hash">{manifest?.policy.hash ?? "—"}</code></dd>
              </dl>
            </Card>

            <Card title="Template Renderers">
              {manifest?.template_renderers && manifest.template_renderers.length > 0 ? (
                <ul className="kn-health__list">
                  {manifest.template_renderers.map((r, i) => (
                    <li key={i}>
                      <strong>{r.profile}</strong> — {r.engine} v{r.profile_version}
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="kn-health__muted">No template renderers configured.</p>
              )}
            </Card>

            {(genStatus?.warnings.length ?? 0) > 0 && (
              <Card title="Warnings" wide>
                <ul className="kn-health__warnings">
                  {genStatus?.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              </Card>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function StatusBadge({ active, label }: { active: boolean; label: string }) {
  return (
    <span className={`kn-health__badge${active ? " kn-health__badge--active" : ""}`}>
      {label}
    </span>
  );
}
