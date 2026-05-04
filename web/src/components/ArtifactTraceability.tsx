import { useCallback, useMemo, useState } from "react";
import type { ManifestArtifact, ManifestResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

type TraceView = "by-entity" | "by-artifact";

export function ArtifactTraceability() {
  const [traceView, setTraceView] = useState<TraceView>("by-entity");
  const [selectedEntity, setSelectedEntity] = useState<string | null>(null);
  const [selectedArtifact, setSelectedArtifact] = useState<string | null>(null);

  const fetcher = useCallback((c: KnowNowClient) => c.getManifest(), []);
  const { data, loading, error } = useApi<ManifestResponse>(fetcher);

  const manifest = data?.manifest;

  const entityToArtifacts = useMemo(() => {
    if (!manifest) return new Map<string, ManifestArtifact[]>();
    const map = new Map<string, ManifestArtifact[]>();
    for (const artifact of manifest.artifacts) {
      for (const objId of artifact.metadata_object_ids) {
        const existing = map.get(objId) ?? [];
        existing.push(artifact);
        map.set(objId, existing);
      }
    }
    return map;
  }, [manifest]);

  const entityIds = useMemo(
    () => [...entityToArtifacts.keys()].sort(),
    [entityToArtifacts],
  );

  if (loading) return <div className="kn-loading" aria-live="polite">Loading traceability data...</div>;
  if (error) return <div className="kn-error" role="alert">Failed to load: {error.message}</div>;
  if (!manifest) return <p className="kn-empty">No manifest available. Run generation first.</p>;

  return (
    <section className="kn-trace" aria-label="Artifact traceability">
      <div className="kn-trace__controls">
        <fieldset className="kn-graph-toggle" aria-label="Traceability view">
          <legend className="kn-sr-only">View mode</legend>
          <label>
            <input
              type="radio"
              name="traceView"
              value="by-entity"
              checked={traceView === "by-entity"}
              onChange={() => { setTraceView("by-entity"); }}
            />
            By Entity
          </label>
          <label>
            <input
              type="radio"
              name="traceView"
              value="by-artifact"
              checked={traceView === "by-artifact"}
              onChange={() => { setTraceView("by-artifact"); }}
            />
            By Artifact
          </label>
        </fieldset>
      </div>

      {traceView === "by-entity" && (
        <div className="kn-trace__split">
          <nav className="kn-trace__list" aria-label="Metadata objects">
            <ul role="listbox">
              {entityIds.map((id) => (
                <li
                  key={id}
                  role="option"
                  aria-selected={selectedEntity === id}
                  className={`kn-trace__item${selectedEntity === id ? " kn-trace__item--selected" : ""}`}
                  onClick={() => { setSelectedEntity(id); }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      setSelectedEntity(id);
                    }
                  }}
                  tabIndex={0}
                >
                  <code>{id}</code>
                  <span className="kn-trace__badge">
                    {String(entityToArtifacts.get(id)?.length ?? 0)} artifacts
                  </span>
                </li>
              ))}
            </ul>
          </nav>
          <div className="kn-trace__detail" aria-live="polite">
            {selectedEntity ? (
              <EntityArtifacts
                entityId={selectedEntity}
                artifacts={entityToArtifacts.get(selectedEntity) ?? []}
              />
            ) : (
              <p className="kn-docs__placeholder">Select a metadata object to see its generated artifacts.</p>
            )}
          </div>
        </div>
      )}

      {traceView === "by-artifact" && (
        <div className="kn-trace__split">
          <nav className="kn-trace__list" aria-label="Artifacts">
            <ul role="listbox">
              {manifest.artifacts.map((a) => (
                <li
                  key={a.artifact_id}
                  role="option"
                  aria-selected={selectedArtifact === a.artifact_id}
                  className={`kn-trace__item${selectedArtifact === a.artifact_id ? " kn-trace__item--selected" : ""}`}
                  onClick={() => { setSelectedArtifact(a.artifact_id); }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      setSelectedArtifact(a.artifact_id);
                    }
                  }}
                  tabIndex={0}
                >
                  <code>{a.path}</code>
                  <span className="kn-trace__badge">{a.kind}</span>
                </li>
              ))}
            </ul>
          </nav>
          <div className="kn-trace__detail" aria-live="polite">
            {selectedArtifact ? (
              <ArtifactDetail
                artifact={manifest.artifacts.find((a) => a.artifact_id === selectedArtifact)}
              />
            ) : (
              <p className="kn-docs__placeholder">Select an artifact to see its metadata sources.</p>
            )}
          </div>
        </div>
      )}
    </section>
  );
}

function EntityArtifacts({
  entityId,
  artifacts,
}: {
  entityId: string;
  artifacts: ManifestArtifact[];
}) {
  return (
    <article>
      <h3>Artifacts for <code>{entityId}</code></h3>
      <table className="kn-manifest__artifacts">
        <thead>
          <tr>
            <th scope="col">Path</th>
            <th scope="col">Kind</th>
            <th scope="col">Generator</th>
          </tr>
        </thead>
        <tbody>
          {artifacts.map((a) => (
            <tr key={a.artifact_id}>
              <td><code>{a.path}</code></td>
              <td>{a.kind}</td>
              <td>{a.generator} v{a.generator_version}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </article>
  );
}

function ArtifactDetail({ artifact }: { artifact: ManifestArtifact | undefined }) {
  if (!artifact) return <p className="kn-empty">Artifact not found.</p>;

  return (
    <article>
      <h3><code>{artifact.path}</code></h3>
      <dl className="kn-gen-status__details">
        <dt>Kind</dt>
        <dd>{artifact.kind}</dd>
        <dt>Artifact ID</dt>
        <dd><code>{artifact.artifact_id}</code></dd>
        <dt>Generator</dt>
        <dd>{artifact.generator} v{artifact.generator_version}</dd>
        <dt>Hash</dt>
        <dd><code>{artifact.hash}</code></dd>
      </dl>

      <h4>Metadata Objects</h4>
      <ul className="kn-trace__obj-list">
        {artifact.metadata_object_ids.map((id) => (
          <li key={id}><code>{id}</code></li>
        ))}
      </ul>
    </article>
  );
}
