import { useCallback } from "react";
import type { ManifestResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

export function ManifestViewer() {
  const fetcher = useCallback((c: KnowNowClient) => c.getManifest(), []);
  const { data, loading, error } = useApi<ManifestResponse>(fetcher);

  if (loading) return <div className="kn-loading" aria-live="polite">Loading manifest...</div>;
  if (error) return <div className="kn-error" role="alert">Failed to load manifest: {error.message}</div>;

  const manifest = data?.manifest;
  if (!manifest) {
    return <p className="kn-empty">No manifest available. Run generation to create one.</p>;
  }

  return (
    <section className="kn-manifest" aria-label="Generation manifest">
      <h2>Manifest</h2>

      <dl className="kn-manifest__meta">
        <dt>Project</dt>
        <dd><code>{manifest.project_id}</code></dd>

        <dt>Engine</dt>
        <dd><code>{manifest.engine_version}</code></dd>

        <dt>Schema Version</dt>
        <dd><code>{manifest.metadata_schema_version}</code></dd>

        <dt>Contract Version</dt>
        <dd><code>{manifest.generator_contract_version}</code></dd>

        <dt>Target</dt>
        <dd>
          {manifest.target_database.kind} {manifest.target_database.version}
          {" "}(floor: {manifest.target_database.compatibility_floor})
        </dd>

        <dt>Policy</dt>
        <dd>{manifest.policy.pack} v{manifest.policy.version}</dd>
      </dl>

      {manifest.template_renderers.length > 0 && (
        <details className="kn-manifest__section">
          <summary>Template Renderers ({String(manifest.template_renderers.length)})</summary>
          <ul>
            {manifest.template_renderers.map((r, i) => (
              <li key={i}>
                {r.profile} ({r.engine} v{r.profile_version})
              </li>
            ))}
          </ul>
        </details>
      )}

      <details className="kn-manifest__section" open>
        <summary>Artifacts ({String(manifest.artifacts.length)})</summary>
        <table className="kn-manifest__artifacts">
          <thead>
            <tr>
              <th scope="col">Path</th>
              <th scope="col">Kind</th>
              <th scope="col">Generator</th>
            </tr>
          </thead>
          <tbody>
            {manifest.artifacts.map((a) => (
              <tr key={a.artifact_id}>
                <td><code>{a.path}</code></td>
                <td>{a.kind}</td>
                <td>{a.generator} v{a.generator_version}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>

      {manifest.warnings.length > 0 && (
        <details className="kn-manifest__section">
          <summary>Warnings ({String(manifest.warnings.length)})</summary>
          <ul className="kn-manifest__warnings">
            {manifest.warnings.map((w, i) => (
              <li key={i}>{w}</li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
