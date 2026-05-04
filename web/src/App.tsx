import { useCallback, useState } from "react";
import type { Entity, GraphResponse, RelationshipsResponse } from "./api/client";
import { KnowNowClient } from "./api/client";
import { useApi } from "./hooks/useApi";
import { EntityDetail } from "./components/EntityDetail";
import { EntityList } from "./components/EntityList";
import { RelationshipGraph } from "./components/RelationshipGraph";
import { RelationshipTable } from "./components/RelationshipTable";

type View = "entities" | "graph";
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
          <button
            className={`kn-nav__tab${view === "entities" ? " kn-nav__tab--active" : ""}`}
            onClick={() => { setView("entities"); }}
            aria-current={view === "entities" ? "page" : undefined}
          >
            Entities
          </button>
          <button
            className={`kn-nav__tab${view === "graph" ? " kn-nav__tab--active" : ""}`}
            onClick={() => { setView("graph"); }}
            aria-current={view === "graph" ? "page" : undefined}
          >
            Relationships
          </button>
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
      </main>
    </div>
  );
}
