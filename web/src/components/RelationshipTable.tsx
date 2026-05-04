import type { Relationship } from "../api/client";

interface RelationshipTableProps {
  relationships: Relationship[];
}

export function RelationshipTable({ relationships }: RelationshipTableProps) {
  if (relationships.length === 0) {
    return <p className="kn-empty">No relationships defined.</p>;
  }

  return (
    <table className="kn-rel-table" aria-label="Relationships table">
      <thead>
        <tr>
          <th scope="col">From</th>
          <th scope="col">To</th>
          <th scope="col">Cardinality</th>
          <th scope="col">Description</th>
        </tr>
      </thead>
      <tbody>
        {relationships.map((rel) => (
          <tr key={rel.id ?? `${rel.from_entity}-${rel.to_entity}`}>
            <td>{rel.from_entity}</td>
            <td>{rel.to_entity}</td>
            <td>{rel.cardinality ?? "—"}</td>
            <td>{rel.description ?? "—"}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
