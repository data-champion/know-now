import { useCallback, useState } from "react";
import type { Entity, GraphResponse, RelationshipsResponse } from "./api/client";
import { KnowNowClient } from "./api/client";
import { useApi } from "./hooks/useApi";
import { ArtifactTraceability } from "./components/ArtifactTraceability";
import { DocsViewer } from "./components/DocsViewer";
import { EntityDetail } from "./components/EntityDetail";
import { EntityList } from "./components/EntityList";
import { GenerationStatus } from "./components/GenerationStatus";
import { HealthAdmin } from "./components/HealthAdmin";
import { ManifestViewer } from "./components/ManifestViewer";
import { RelationshipGraph } from "./components/RelationshipGraph";
import { RelationshipTable } from "./components/RelationshipTable";
import { ReviewSummary } from "./components/ReviewSummary";

type View = "entities" | "graph" | "generation" | "docs" | "manifest" | "traceability" | "health" | "review";
type GraphMode = "visual" | "table";

export function App() {
  const [view, setView] = useState<View>("entities");
  const [selectedEntity, setSelectedEntity] = useState<Entity | null>(null);
  const [graphMode, setGraphMode] = useState<GraphMode>("visual");

  const fetchGraph = useCallback((c: KnowNowClient) => c.getGraph(), []);
  const fetchRelationships = useCallback((c: KnowNowClient) => c.getRelationships(), []);

  const { data: graphData } = useApi<GraphResponse>(fetchGraph);
  const { data: relData } = useApi<RelationshipsResponse>(fetchRelationships);

  const handleNodeSelect = useCallback((nodeId: string) => {
    setSelectedEntity(null);
    setView("entities");
    window.location.hash = `#entity/${nodeId}`;
  }, []);

  return (
    <div className="kn-app">
      <header className="kn-header">
        <h1 className="kn-header__title">know-now</h1>
        <nav className="kn-nav" aria-label="Main navigation">
          <NavTab view={view} target="entities" label="Entities" onClick={setView} />
          <NavTab view={view} target="graph" label="Relationships" onClick={setView} />
          <NavTab view={view} target="generation" label="Generation" onClick={setView} />
          <NavTab view={view} target="docs" label="Docs" onClick={setView} />
          <NavTab view={view} target="manifest" label="Manifest" onClick={setView} />
          <NavTab view={view} target="traceability" label="Traceability" onClick={setView} />
          <NavTab view={view} target="health" label="Health" onClick={setView} />
          <NavTab view={view} target="review" label="Review" onClick={setView} />
        </nav>
      </header>

      <main className="kn-main">
        {view === "entities" && (
          <div className="kn-layout">
            <EntityList
              onSelect={(e) => { setSelectedEntity(e); }}
              selectedId={selectedEntity?.id ?? selectedEntity?.name ?? null}
            />
            {selectedEntity && (
              <EntityDetail
                entity={selectedEntity}
                onClose={() => { setSelectedEntity(null); }}
              />
            )}
          </div>
        )}

        {view === "graph" && (
          <div className="kn-graph-view">
            <div className="kn-graph-view__controls">
              <fieldset className="kn-graph-toggle" aria-label="Graph display mode">
                <legend className="kn-sr-only">Display mode</legend>
                <label>
                  <input
                    type="radio"
                    name="graphMode"
                    value="visual"
                    checked={graphMode === "visual"}
                    onChange={() => { setGraphMode("visual"); }}
                  />
                  Graph
                </label>
                <label>
                  <input
                    type="radio"
                    name="graphMode"
                    value="table"
                    checked={graphMode === "table"}
                    onChange={() => { setGraphMode("table"); }}
                  />
                  Table
                </label>
              </fieldset>
            </div>

            {graphMode === "visual" ? (
              <RelationshipGraph
                nodes={graphData?.nodes ?? []}
                edges={graphData?.edges ?? []}
                onNodeSelect={handleNodeSelect}
                selectedNodeId={null}
              />
            ) : (
              <RelationshipTable relationships={relData?.relationships ?? []} />
            )}
          </div>
        )}

        {view === "generation" && (
          <div className="kn-page-content">
            <GenerationStatus />
          </div>
        )}

        {view === "docs" && <DocsViewer />}

        {view === "manifest" && (
          <div className="kn-page-content">
            <ManifestViewer />
          </div>
        )}

        {view === "traceability" && <ArtifactTraceability />}

        {view === "health" && (
          <div className="kn-page-content">
            <HealthAdmin />
          </div>
        )}

        {view === "review" && (
          <div className="kn-page-content">
            <ReviewSummary />
          </div>
        )}
      </main>
    </div>
  );
}

function NavTab({
  view,
  target,
  label,
  onClick,
}: {
  view: View;
  target: View;
  label: string;
  onClick: (v: View) => void;
}) {
  return (
    <button
      className={`kn-nav__tab${view === target ? " kn-nav__tab--active" : ""}`}
      onClick={() => { onClick(target); }}
      aria-current={view === target ? "page" : undefined}
    >
      {label}
    </button>
  );
}
