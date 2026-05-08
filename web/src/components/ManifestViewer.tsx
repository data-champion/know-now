import { useCallback } from "react";
import type { ManifestResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";
import { Card } from "./ui/Card";
import { PageHeader } from "./ui/PageHeader";

export function ManifestViewer() {
  const fetcher = useCallback((c: KnowNowClient) => c.getManifest(), []);
  const { data, loading, error } = useApi<ManifestResponse>(fetcher);

  const manifest = data?.manifest;

  return (
    <div className="kn-page">
      <div className="kn-page__inner">
        <PageHeader
          title="Manifest"
          description="Build manifest for the current generation, including artifacts and warnings."
        />

        {loading && <div className="kn-loading" aria-live="polite">Loading manifest...</div>}
        {error && <div className="kn-error" role="alert">Failed to load manifest: {error.message}</div>}
        {!loading && !error && !manifest && (
          <p className="kn-empty">No manifest available. Run generation to create one.</p>
        )}

        {manifest && (
          <>
            <Card title="Overview">
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
            </Card>

            {manifest.template_renderers.length > 0 && (
              <Card title={`Template Renderers (${String(manifest.template_renderers.length)})`}>
                <ul className="kn-health__list">
                  {manifest.template_renderers.map((r, i) => (
                    <li key={i}>
                      {r.profile} ({r.engine} v{r.profile_version})
                    </li>
                  ))}
                </ul>
              </Card>
            )}

            <Card title={`Artifacts (${String(manifest.artifacts.length)})`}>
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
            </Card>

            {manifest.warnings.length > 0 && (
              <Card title={`Warnings (${String(manifest.warnings.length)})`}>
                <ul className="kn-manifest__warnings">
                  {manifest.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              </Card>
            )}
          </>
        )}
      </div>
    </div>
  );
}
