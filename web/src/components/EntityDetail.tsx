import type { Entity } from "../api/client";

interface EntityDetailProps {
  entity: Entity;
  onClose: () => void;
}

export function EntityDetail({ entity, onClose }: EntityDetailProps) {
  return (
    <article className="kn-entity-detail" aria-label={`Details for ${entity.name}`}>
      <header className="kn-entity-detail__header">
        <h2>{entity.name}</h2>
        <button
          className="kn-entity-detail__close"
          onClick={onClose}
          aria-label="Close detail view"
        >
          &times;
        </button>
      </header>

      {entity.description && (
        <p className="kn-entity-detail__desc">{entity.description}</p>
      )}

      <dl className="kn-entity-detail__meta">
        {entity.domain && (
          <>
            <dt>Domain</dt>
            <dd>{entity.domain}</dd>
          </>
        )}
        {entity.id && (
          <>
            <dt>ID</dt>
            <dd><code>{entity.id}</code></dd>
          </>
        )}
      </dl>

      {entity.attributes.length > 0 && (
        <section aria-label="Attributes">
          <h3>Attributes</h3>
          <table className="kn-entity-detail__attrs">
            <thead>
              <tr>
                <th scope="col">Name</th>
                <th scope="col">Type</th>
                <th scope="col">Description</th>
              </tr>
            </thead>
            <tbody>
              {entity.attributes.map((attr) => (
                <tr key={attr.id ?? attr.name}>
                  <td>{attr.name}</td>
                  <td><code>{attr.logical_type}</code></td>
                  <td>{attr.description ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}
    </article>
  );
}
