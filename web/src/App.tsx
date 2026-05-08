import { useCallback, useEffect, useRef, useState } from "react";
import type {
  Entity,
  GraphResponse,
  ProjectResponse,
  RelationshipsResponse,
} from "./api/client";
import { ApiError, KnowNowClient } from "./api/client";
import { useApi } from "./hooks/useApi";
import { ArtifactTraceability } from "./components/ArtifactTraceability";
import { DocsViewer } from "./components/DocsViewer";
import { EntityDetail } from "./components/EntityDetail";
import { EntityList } from "./components/EntityList";
import { GenerationStatus } from "./components/GenerationStatus";
import { HealthAdmin } from "./components/HealthAdmin";
import { ManifestViewer } from "./components/ManifestViewer";
import { MetadataErrorView } from "./components/MetadataErrorView";
import { RelationshipGraph } from "./components/RelationshipGraph";
import { RelationshipTable } from "./components/RelationshipTable";
import { ReviewSummary } from "./components/ReviewSummary";
import { Sidebar, getViewLabel, type View } from "./components/ui/Sidebar";
import { TopBar } from "./components/ui/TopBar";

type GraphMode = "visual" | "table";

export function App() {
  const mainRef = useRef<HTMLElement>(null);
  const [view, setView] = useState<View>("entities");
  const [selectedEntity, setSelectedEntity] = useState<Entity | null>(null);
  const [graphMode, setGraphMode] = useState<GraphMode>("visual");
  const [navOpen, setNavOpen] = useState(false);

  const fetchProject = useCallback((c: KnowNowClient) => c.getProject(), []);
  const fetchGraph = useCallback((c: KnowNowClient) => c.getGraph(), []);
  const fetchRelationships = useCallback((c: KnowNowClient) => c.getRelationships(), []);

  const { error: projectError, reload: reloadProject } =
    useApi<ProjectResponse>(fetchProject);
  const { data: graphData } = useApi<GraphResponse>(fetchGraph);
  const { data: relData } = useApi<RelationshipsResponse>(fetchRelationships);

  const metadataError =
    projectError instanceof ApiError ? projectError.metadataError : undefined;

  useEffect(() => {
    mainRef.current?.focus();
  }, [view]);

  const handleNodeSelect = useCallback((nodeId: string) => {
    setSelectedEntity(null);
    setView("entities");
    window.location.hash = `#entity/${nodeId}`;
  }, []);

  const topbarActions = view === "graph" ? (
    <fieldset className="kn-segmented" aria-label="Relationship view mode">
      <legend className="kn-sr-only">Display mode</legend>
      <label className={`kn-segmented__btn${graphMode === "visual" ? " kn-segmented__btn--active" : ""}`}>
        <input
          type="radio"
          name="graphMode"
          value="visual"
          checked={graphMode === "visual"}
          onChange={() => { setGraphMode("visual"); }}
          className="kn-sr-only"
        />
        Graph
      </label>
      <label className={`kn-segmented__btn${graphMode === "table" ? " kn-segmented__btn--active" : ""}`}>
        <input
          type="radio"
          name="graphMode"
          value="table"
          checked={graphMode === "table"}
          onChange={() => { setGraphMode("table"); }}
          className="kn-sr-only"
        />
        Table
      </label>
    </fieldset>
  ) : null;

  if (metadataError) {
    return (
      <div className="kn-app kn-app--error">
        <a href="#main-content" className="kn-skip-link">Skip to main content</a>
        <main className="kn-main" ref={mainRef} tabIndex={-1} id="main-content">
          <MetadataErrorView error={metadataError} onRetry={reloadProject} />
        </main>
      </div>
    );
  }

  return (
    <div className="kn-app">
      <a href="#main-content" className="kn-skip-link">Skip to main content</a>
      <Sidebar
        view={view}
        onChange={setView}
        open={navOpen}
        onClose={() => { setNavOpen(false); }}
      />
      <TopBar
        title={getViewLabel(view)}
        actions={topbarActions}
        onMenuToggle={() => { setNavOpen((v) => !v); }}
      />
      <main className="kn-main" ref={mainRef} tabIndex={-1} id="main-content">
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

        {view === "generation" && <GenerationStatus />}
        {view === "docs" && <DocsViewer />}
        {view === "manifest" && <ManifestViewer />}
        {view === "traceability" && <ArtifactTraceability />}
        {view === "health" && <HealthAdmin />}
        {view === "review" && <ReviewSummary />}
      </main>
    </div>
  );
}
