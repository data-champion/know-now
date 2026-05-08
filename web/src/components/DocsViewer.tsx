import { useCallback, useState } from "react";
import { marked } from "marked";
import type { DocFile, DocsContentResponse, DocsListResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

marked.setOptions({ gfm: true, breaks: false });

export function DocsViewer() {
  const [selectedDoc, setSelectedDoc] = useState<string | null>(null);

  const fetcher = useCallback((c: KnowNowClient) => c.getDocsList(), []);
  const { data, loading, error } = useApi<DocsListResponse>(fetcher);

  if (loading) return <div className="kn-loading" aria-live="polite">Loading docs...</div>;
  if (error) return <div className="kn-error" role="alert">Failed to load docs: {error.message}</div>;

  const docs = data?.docs ?? [];

  if (docs.length === 0) {
    return <p className="kn-empty">No generated documentation available.</p>;
  }

  return (
    <div className="kn-docs">
      <nav className="kn-docs__nav" aria-label="Documentation files">
        <ul role="listbox">
          {docs.map((doc) => (
            <li
              key={doc.path}
              role="option"
              aria-selected={selectedDoc === doc.path}
              className={`kn-docs__nav-item${selectedDoc === doc.path ? " kn-docs__nav-item--selected" : ""}`}
              onClick={() => { setSelectedDoc(doc.path); }}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  setSelectedDoc(doc.path);
                }
              }}
              tabIndex={0}
            >
              <span className="kn-docs__nav-path">{doc.path}</span>
              <span className="kn-docs__nav-kind">{doc.kind}</span>
            </li>
          ))}
        </ul>
      </nav>

      <div className="kn-docs__content" aria-live="polite">
        {selectedDoc ? (
          <DocContent path={selectedDoc} docs={docs} />
        ) : (
          <p className="kn-docs__placeholder">Select a document to view.</p>
        )}
      </div>
    </div>
  );
}

function DocContent({ path, docs }: { path: string; docs: DocFile[] }) {
  const fetcher = useCallback(
    (c: KnowNowClient) => c.getDocsContent(path),
    [path],
  );
  const { data, loading, error } = useApi<DocsContentResponse>(fetcher);

  if (loading) return <div className="kn-loading">Loading...</div>;
  if (error) return <div className="kn-error">Failed to load: {error.message}</div>;
  if (!data) return null;

  const doc = docs.find((d) => d.path === path);
  const kind = doc?.kind ?? "md";

  if (kind === "svg") {
    return (
      <div
        className="kn-docs__svg"
        dangerouslySetInnerHTML={{ __html: data.content }}
      />
    );
  }

  if (kind === "html") {
    return (
      <div
        className="kn-docs__html"
        dangerouslySetInnerHTML={{ __html: data.content }}
      />
    );
  }

  return (
    <div className="kn-docs__markdown">
      <MarkdownRenderer content={data.content} />
    </div>
  );
}

function MarkdownRenderer({ content }: { content: string }) {
  const html = marked.parse(content, { async: false });
  return (
    <div
      className="kn-docs__rendered"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
