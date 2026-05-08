import { useCallback, useMemo, useState } from "react";
import type { Domain, DomainsResponse, EntitiesResponse, Entity } from "../api/client";
import { KnowNowClient } from "../api/client";
import { useApi } from "../hooks/useApi";
import { SearchInput } from "./SearchInput";

interface EntityListProps {
  onSelect: (entity: Entity) => void;
  selectedId: string | null;
}

export function EntityList({ onSelect, selectedId }: EntityListProps) {
  const [search, setSearch] = useState("");
  const [domainFilter, setDomainFilter] = useState<string>("");

  const fetchEntities = useCallback((c: KnowNowClient) => c.getEntities(), []);
  const fetchDomains = useCallback((c: KnowNowClient) => c.getDomains(), []);

  const { data: entitiesResp, loading, error } = useApi<EntitiesResponse>(fetchEntities);
  const { data: domainsResp } = useApi<DomainsResponse>(fetchDomains);

  const entities: Entity[] = entitiesResp?.entities ?? [];
  const domains: Domain[] = domainsResp?.domains ?? [];

  const domainLabel = useMemo(() => {
    const map = new Map<string, string>();
    for (const d of domains) {
      if (d.id) map.set(d.id, d.name);
      map.set(d.name, d.name);
    }
    return (key: string) => map.get(key) ?? key;
  }, [domains]);

  const filtered = useMemo(() => {
    const term = search.toLowerCase();
    return entities.filter((e) => {
      if (domainFilter && e.domain !== domainFilter) return false;
      if (!term) return true;
      return (
        e.name.toLowerCase().includes(term) ||
        (e.description?.toLowerCase().includes(term) ?? false)
      );
    });
  }, [entities, search, domainFilter]);

  if (loading) return <div className="kn-loading" aria-live="polite">Loading entities...</div>;
  if (error) return <div className="kn-error" role="alert">Failed to load entities: {error.message}</div>;

  return (
    <section className="kn-entity-list" aria-label="Entity list">
      <div className="kn-entity-list__controls">
        <SearchInput
          value={search}
          onChange={setSearch}
          placeholder="Search entities..."
        />
        <DomainFilter
          domains={domains}
          value={domainFilter}
          onChange={setDomainFilter}
        />
      </div>
      <div className="kn-entity-list__count" aria-live="polite">
        {filtered.length} of {entities.length} entities
      </div>
      <ul className="kn-entity-list__items" role="listbox" aria-label="Entities">
        {filtered.map((entity) => (
          <li
            key={entity.id ?? entity.name}
            role="option"
            aria-selected={selectedId === (entity.id ?? entity.name)}
            className={`kn-entity-list__item${selectedId === (entity.id ?? entity.name) ? " kn-entity-list__item--selected" : ""}`}
            onClick={() => { onSelect(entity); }}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onSelect(entity);
              }
            }}
            tabIndex={0}
          >
            <span className="kn-entity-list__name">{entity.name}</span>
            {entity.domain && (
              <span className="kn-entity-list__domain">{domainLabel(entity.domain)}</span>
            )}
            {entity.description && (
              <span className="kn-entity-list__desc">{entity.description}</span>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

function DomainFilter({
  domains,
  value,
  onChange,
}: {
  domains: Domain[];
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <select
      className="kn-domain-filter"
      aria-label="Filter by domain"
      value={value}
      onChange={(e) => { onChange(e.target.value); }}
    >
      <option value="">All domains</option>
      {domains.map((d) => (
        <option key={d.id ?? d.name} value={d.id ?? d.name}>
          {d.name}
        </option>
      ))}
    </select>
  );
}
