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

export interface GenerationStatusResponse {
  has_generation: boolean;
  engine_version: string;
  artifact_count: number;
  warnings: string[];
  last_generated_epoch: number | null;
  generated_dir_exists: boolean;
}

export interface ManifestArtifact {
  path: string;
  kind: string;
  artifact_id: string;
  generator: string;
  generator_version: string;
  hash: string;
  metadata_object_ids: string[];
}

export interface Manifest {
  engine_version: string;
  metadata_schema_version: string;
  generator_contract_version: string;
  project_id: string;
  input_hash: string;
  lockfile_hash: string;
  target_database: {
    kind: string;
    version: string;
    compatibility_floor: string;
  };
  policy: {
    pack: string;
    version: string;
    hash: string;
  };
  template_renderers: Array<{
    profile: string;
    engine: string;
    profile_version: string;
  }>;
  artifacts: ManifestArtifact[];
  warnings: string[];
}

export interface ManifestResponse {
  manifest: Manifest | null;
}

export interface DocFile {
  path: string;
  kind: string;
}

export interface DocsListResponse {
  docs: DocFile[];
}

export interface DocsContentResponse {
  path: string;
  content: string;
}
