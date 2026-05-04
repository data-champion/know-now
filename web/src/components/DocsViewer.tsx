import { useCallback, useState } from "react";
import type { DocFile, DocsContentResponse, DocsListResponse } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

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
  const html = markdownToHtml(content);
  return (
    <div
      className="kn-docs__rendered"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function markdownToHtml(md: string): string {
  const lines = md.split("\n");
  const output: string[] = [];
  let inCodeBlock = false;
  let inList = false;

  for (const line of lines) {
    if (line.startsWith("```")) {
      if (inCodeBlock) {
        output.push("</code></pre>");
        inCodeBlock = false;
      } else {
        if (inList) { output.push("</ul>"); inList = false; }
        output.push("<pre><code>");
        inCodeBlock = true;
      }
      continue;
    }

    if (inCodeBlock) {
      output.push(escapeHtml(line));
      continue;
    }

    if (line.startsWith("# ")) {
      if (inList) { output.push("</ul>"); inList = false; }
      output.push(`<h1>${escapeHtml(line.slice(2))}</h1>`);
    } else if (line.startsWith("## ")) {
      if (inList) { output.push("</ul>"); inList = false; }
      output.push(`<h2>${escapeHtml(line.slice(3))}</h2>`);
    } else if (line.startsWith("### ")) {
      if (inList) { output.push("</ul>"); inList = false; }
      output.push(`<h3>${escapeHtml(line.slice(4))}</h3>`);
    } else if (line.startsWith("- ") || line.startsWith("* ")) {
      if (!inList) { output.push("<ul>"); inList = true; }
      output.push(`<li>${escapeHtml(line.slice(2))}</li>`);
    } else if (line.trim() === "") {
      if (inList) { output.push("</ul>"); inList = false; }
      output.push("");
    } else {
      if (inList) { output.push("</ul>"); inList = false; }
      output.push(`<p>${escapeHtml(line)}</p>`);
    }
  }

  if (inList) output.push("</ul>");
  if (inCodeBlock) output.push("</code></pre>");

  return output.join("\n");
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
