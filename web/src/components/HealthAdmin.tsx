import { useCallback } from "react";
import type {
  GenerationStatusResponse,
  ManifestResponse,
  StatusResponse,
} from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

export function HealthAdmin() {
  const fetchStatus = useCallback((c: KnowNowClient) => c.getStatus(), []);
  const fetchGenStatus = useCallback((c: KnowNowClient) => c.getGenerationStatus(), []);
  const fetchManifest = useCallback((c: KnowNowClient) => c.getManifest(), []);

  const { data: status, loading: l1 } = useApi<StatusResponse>(fetchStatus);
  const { data: genStatus, loading: l2 } = useApi<GenerationStatusResponse>(fetchGenStatus);
  const { data: manifestResp, loading: l3 } = useApi<ManifestResponse>(fetchManifest);

  if (l1 || l2 || l3) return <div className="kn-loading" aria-live="polite">Loading health data...</div>;

  const manifest = manifestResp?.manifest;

  return (
    <section className="kn-health" aria-label="Health and administration">
      <h2>Health &amp; Admin</h2>

      <div className="kn-health__grid">
        <HealthCard title="Server">
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
        </HealthCard>

        <HealthCard title="Security">
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
        </HealthCard>

        <HealthCard title="Generation">
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
        </HealthCard>

        <HealthCard title="Metadata Schema">
          <dl className="kn-health__dl">
            <dt>Schema Version</dt>
            <dd><code>{manifest?.metadata_schema_version ?? "—"}</code></dd>
            <dt>Contract Version</dt>
            <dd><code>{manifest?.generator_contract_version ?? "—"}</code></dd>
          </dl>
        </HealthCard>

        <HealthCard title="Target Database">
          <dl className="kn-health__dl">
            <dt>Kind</dt>
            <dd>{manifest?.target_database.kind ?? "—"}</dd>
            <dt>Version</dt>
            <dd><code>{manifest?.target_database.version ?? "—"}</code></dd>
            <dt>Compat Floor</dt>
            <dd><code>{manifest?.target_database.compatibility_floor ?? "—"}</code></dd>
          </dl>
        </HealthCard>

        <HealthCard title="Policy">
          <dl className="kn-health__dl">
            <dt>Pack</dt>
            <dd>{manifest?.policy.pack ?? "—"}</dd>
            <dt>Version</dt>
            <dd><code>{manifest?.policy.version ?? "—"}</code></dd>
            <dt>Hash</dt>
            <dd><code className="kn-health__hash">{manifest?.policy.hash ?? "—"}</code></dd>
          </dl>
        </HealthCard>

        <HealthCard title="Template Renderers">
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
        </HealthCard>

        {(genStatus?.warnings.length ?? 0) > 0 && (
          <HealthCard title="Warnings" wide>
            <ul className="kn-health__warnings">
              {genStatus?.warnings.map((w, i) => (
                <li key={i}>{w}</li>
              ))}
            </ul>
          </HealthCard>
        )}
      </div>
    </section>
  );
}

function HealthCard({
  title,
  children,
  wide,
}: {
  title: string;
  children: React.ReactNode;
  wide?: boolean;
}) {
  return (
    <div className={`kn-health__card${wide ? " kn-health__card--wide" : ""}`}>
      <h3 className="kn-health__card-title">{title}</h3>
      {children}
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
