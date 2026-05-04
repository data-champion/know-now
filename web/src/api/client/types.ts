/** Generated from docs/dev/api/openapi.yaml — do not edit manually. */

export interface HealthResponse {
  status: string;
}

export interface VersionResponse {
  engine_version: string;
  api_contract_version: string;
  compatibility: string;
}

export interface StatusResponse {
  server: string;
  version: string;
  write_mode: boolean;
}

export interface Project {
  name: string;
  description: string | null;
  owner: string | null;
  tags: string[];
}

export interface ProjectResponse {
  project: Project | null;
  version: string | null;
}

export interface Domain {
  id: string | null;
  name: string;
  description: string | null;
}

export interface DomainsResponse {
  domains: Domain[];
}

export interface Module {
  id: string | null;
  name: string;
  description: string | null;
}

export interface ModulesResponse {
  modules: Module[];
}

export interface Attribute {
  id: string | null;
  name: string;
  logical_type: string;
  description: string | null;
}

export interface Entity {
  id: string | null;
  name: string;
  domain: string | null;
  description: string | null;
  attributes: Attribute[];
}

export interface EntitiesResponse {
  entities: Entity[];
}

export interface Relationship {
  id: string | null;
  from_entity: string;
  to_entity: string;
  cardinality: string | null;
  description: string | null;
}

export interface RelationshipsResponse {
  relationships: Relationship[];
}

export interface GraphNode {
  id: string;
  name: string;
  domain: string | null;
}

export interface GraphEdge {
  id: string;
  from: string;
  to: string;
}

export interface GraphResponse {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface OpenQuestion {
  id: string | null;
  question: string;
  context: string | null;
  entity: string | null;
  priority: string | null;
}

export interface OpenQuestionsResponse {
  open_questions: OpenQuestion[];
}
