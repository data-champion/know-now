import type { MetadataErrorResponse } from "../api/client";
import { Card } from "./ui/Card";
import { PageHeader } from "./ui/PageHeader";

interface Props {
  error: MetadataErrorResponse;
  onRetry?: () => void;
}

export function MetadataErrorView({ error, onRetry }: Props) {
  const formatLocation = (file: string, line?: number, column?: number) => {
    if (!file) return "";
    if (line === undefined) return file;
    if (column === undefined) return `${file}:${String(line)}`;
    return `${file}:${String(line)}:${String(column)}`;
  };

  return (
    <div className="kn-page kn-page--error">
      <PageHeader
        title="Project metadata could not be loaded"
        description={error.summary}
        actions={
          onRetry && (
            <button
              type="button"
              className="kn-btn"
              onClick={onRetry}
            >
              Retry
            </button>
          )
        }
      />

      {error.errors.length > 0 ? (
        <Card title={`${String(error.errors.length)} parse error${error.errors.length === 1 ? "" : "s"}`}>
          <ul className="kn-error-list">
            {error.errors.map((entry, idx) => (
              <li key={`${entry.file}-${entry.code}-${String(idx)}`} className="kn-error-list__item">
                <div className="kn-error-list__header">
                  <code className="kn-error-list__code">{entry.code}</code>
                  <span className="kn-error-list__location">
                    {formatLocation(entry.file, entry.line, entry.column)}
                  </span>
                </div>
                <pre className="kn-error-list__message">{entry.message}</pre>
              </li>
            ))}
          </ul>
        </Card>
      ) : (
        <Card>
          <p>
            Run <code>know-now validate</code> in your project for full
            diagnostics.
          </p>
        </Card>
      )}
    </div>
  );
}
