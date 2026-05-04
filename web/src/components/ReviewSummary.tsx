import { useCallback, useMemo, useState } from "react";
import type {
  EntitiesResponse,
  OpenQuestionsResponse,
  RelationshipsResponse,
  ReviewItemStatus,
} from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";

type ReviewTab = "summary" | "questions" | "approval";

const STATUS_OPTIONS: ReviewItemStatus[] = [
  "draft",
  "needs-confirmation",
  "confirmed",
  "rejected",
  "deferred",
];

export function ReviewSummary() {
  const [tab, setTab] = useState<ReviewTab>("summary");

  return (
    <section className="kn-review" aria-label="Review">
      <div className="kn-review__tabs">
        <button
          className={`kn-nav__tab${tab === "summary" ? " kn-nav__tab--active" : ""}`}
          onClick={() => { setTab("summary"); }}
        >
          Summary
        </button>
        <button
          className={`kn-nav__tab${tab === "questions" ? " kn-nav__tab--active" : ""}`}
          onClick={() => { setTab("questions"); }}
        >
          Open Questions
        </button>
        <button
          className={`kn-nav__tab${tab === "approval" ? " kn-nav__tab--active" : ""}`}
          onClick={() => { setTab("approval"); }}
        >
          Change Approval
        </button>
      </div>

      <div className="kn-review__content">
        {tab === "summary" && <SummaryView />}
        {tab === "questions" && <QuestionsView />}
        {tab === "approval" && <ApprovalView />}
      </div>
    </section>
  );
}

function SummaryView() {
  const fetchEntities = useCallback((c: KnowNowClient) => c.getEntities(), []);
  const fetchRelationships = useCallback((c: KnowNowClient) => c.getRelationships(), []);
  const fetchQuestions = useCallback((c: KnowNowClient) => c.getOpenQuestions(), []);

  const { data: entData } = useApi<EntitiesResponse>(fetchEntities);
  const { data: relData } = useApi<RelationshipsResponse>(fetchRelationships);
  const { data: qData } = useApi<OpenQuestionsResponse>(fetchQuestions);

  const entities = entData?.entities ?? [];
  const relationships = relData?.relationships ?? [];
  const questions = qData?.open_questions ?? [];

  const exportMarkdown = useCallback(() => {
    const lines: string[] = [
      "# Review Summary",
      "",
      `## Entities (${String(entities.length)})`,
      "",
      ...entities.map((e) => `- **${e.name}**${e.description ? `: ${e.description}` : ""}`),
      "",
      `## Relationships (${String(relationships.length)})`,
      "",
      ...relationships.map((r) => `- ${r.from_entity} → ${r.to_entity}${r.cardinality ? ` (${r.cardinality})` : ""}`),
      "",
      `## Open Questions (${String(questions.length)})`,
      "",
      ...questions.map((q) => `- ${q.question}${q.context ? ` — _${q.context}_` : ""}`),
    ];
    const blob = new Blob([lines.join("\n")], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "review-summary.md";
    a.click();
    URL.revokeObjectURL(url);
  }, [entities, relationships, questions]);

  return (
    <div className="kn-review__summary">
      <div className="kn-review__actions">
        <button className="kn-btn" onClick={exportMarkdown}>
          Export Markdown
        </button>
        <button
          className="kn-btn kn-btn--secondary"
          onClick={() => {
            void navigator.clipboard.writeText(window.location.origin + window.location.pathname);
          }}
        >
          Copy Stakeholder Link
        </button>
      </div>

      <div className="kn-review__stats">
        <div className="kn-review__stat">
          <span className="kn-review__stat-value">{String(entities.length)}</span>
          <span className="kn-review__stat-label">Entities</span>
        </div>
        <div className="kn-review__stat">
          <span className="kn-review__stat-value">{String(relationships.length)}</span>
          <span className="kn-review__stat-label">Relationships</span>
        </div>
        <div className="kn-review__stat">
          <span className="kn-review__stat-value">{String(questions.length)}</span>
          <span className="kn-review__stat-label">Open Questions</span>
        </div>
      </div>

      <h3>Entities</h3>
      <ul className="kn-review__list">
        {entities.map((e) => (
          <li key={e.id ?? e.name}>
            <strong>{e.name}</strong>
            {e.description && <span className="kn-review__desc"> — {e.description}</span>}
          </li>
        ))}
      </ul>

      <h3>Relationships</h3>
      <ul className="kn-review__list">
        {relationships.map((r) => (
          <li key={r.id ?? `${r.from_entity}-${r.to_entity}`}>
            {r.from_entity} → {r.to_entity}
            {r.cardinality && <span className="kn-review__desc"> ({r.cardinality})</span>}
          </li>
        ))}
      </ul>
    </div>
  );
}

function QuestionsView() {
  const [priorityFilter, setPriorityFilter] = useState("");
  const fetchQuestions = useCallback((c: KnowNowClient) => c.getOpenQuestions(), []);
  const { data } = useApi<OpenQuestionsResponse>(fetchQuestions);

  const questions = data?.open_questions ?? [];

  const filtered = useMemo(() => {
    if (!priorityFilter) return questions;
    return questions.filter((q) => q.priority === priorityFilter);
  }, [questions, priorityFilter]);

  const priorities = useMemo(
    () => [...new Set(questions.map((q) => q.priority).filter(Boolean))],
    [questions],
  );

  return (
    <div className="kn-review__questions">
      <div className="kn-review__filter">
        <select
          aria-label="Filter by priority"
          value={priorityFilter}
          onChange={(e) => { setPriorityFilter(e.target.value); }}
          className="kn-domain-filter"
        >
          <option value="">All priorities</option>
          {priorities.map((p) => (
            <option key={p} value={p ?? ""}>{p}</option>
          ))}
        </select>
      </div>
      <table className="kn-review__table" aria-label="Open questions register">
        <thead>
          <tr>
            <th scope="col">Question</th>
            <th scope="col">Entity</th>
            <th scope="col">Priority</th>
            <th scope="col">Context</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((q) => (
            <tr key={q.id ?? q.question}>
              <td>{q.question}</td>
              <td>{q.entity ?? "—"}</td>
              <td>{q.priority ?? "—"}</td>
              <td>{q.context ?? "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ApprovalView() {
  const [items, setItems] = useState<Record<string, ReviewItemStatus>>({});
  const fetchEntities = useCallback((c: KnowNowClient) => c.getEntities(), []);
  const { data } = useApi<EntitiesResponse>(fetchEntities);

  const entities = data?.entities ?? [];

  const handleStatusChange = useCallback((entityId: string, status: ReviewItemStatus) => {
    setItems((prev) => ({ ...prev, [entityId]: status }));
  }, []);

  return (
    <div className="kn-review__approval">
      <p className="kn-health__muted">
        Set review status for each entity. State is stored locally.
      </p>
      <table className="kn-review__table" aria-label="Change approval">
        <thead>
          <tr>
            <th scope="col">Entity</th>
            <th scope="col">Status</th>
          </tr>
        </thead>
        <tbody>
          {entities.map((e) => {
            const id = e.id ?? e.name;
            return (
              <tr key={id}>
                <td>{e.name}</td>
                <td>
                  <select
                    value={items[id] ?? "draft"}
                    onChange={(ev) => { handleStatusChange(id, ev.target.value as ReviewItemStatus); }}
                    className="kn-review__status-select"
                    aria-label={`Status for ${e.name}`}
                  >
                    {STATUS_OPTIONS.map((s) => (
                      <option key={s} value={s}>{s}</option>
                    ))}
                  </select>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
