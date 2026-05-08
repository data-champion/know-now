/** Generated from docs/dev/api/openapi.yaml — do not edit manually. */

import type {
  DocsContentResponse,
  DocsListResponse,
  DomainsResponse,
  EntitiesResponse,
  GenerationStatusResponse,
  GraphResponse,
  HealthResponse,
  ManifestResponse,
  ModulesResponse,
  OpenQuestionsResponse,
  ProjectResponse,
  RelationshipsResponse,
  ReviewStateResponse,
  StatusResponse,
  VersionResponse,
} from "./types";

const GENERATED_AGAINST_API_VERSION = "1";

export class KnowNowClient {
  private readonly baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
  }

  async checkCompatibility(): Promise<void> {
    const version = await this.getVersion();
    if (version.api_contract_version !== GENERATED_AGAINST_API_VERSION) {
      console.warn(
        `[know-now] API version mismatch: client expects v${GENERATED_AGAINST_API_VERSION}, ` +
          `server reports v${version.api_contract_version}. Some features may not work correctly.`,
      );
    }
  }

  async getHealth(): Promise<HealthResponse> {
    return this.get<HealthResponse>("/__health");
  }

  async getVersion(): Promise<VersionResponse> {
    return this.get<VersionResponse>("/api/v1/version");
  }

  async getStatus(): Promise<StatusResponse> {
    return this.get<StatusResponse>("/api/v1/status");
  }

  async getProject(): Promise<ProjectResponse> {
    return this.get<ProjectResponse>("/api/v1/project");
  }

  async getDomains(): Promise<DomainsResponse> {
    return this.get<DomainsResponse>("/api/v1/domains");
  }

  async getModules(): Promise<ModulesResponse> {
    return this.get<ModulesResponse>("/api/v1/modules");
  }

  async getEntities(): Promise<EntitiesResponse> {
    return this.get<EntitiesResponse>("/api/v1/entities");
  }

  async getRelationships(): Promise<RelationshipsResponse> {
    return this.get<RelationshipsResponse>("/api/v1/relationships");
  }

  async getGraph(): Promise<GraphResponse> {
    return this.get<GraphResponse>("/api/v1/graph");
  }

  async getOpenQuestions(): Promise<OpenQuestionsResponse> {
    return this.get<OpenQuestionsResponse>("/api/v1/open-questions");
  }

  async getGenerationStatus(): Promise<GenerationStatusResponse> {
    return this.get<GenerationStatusResponse>("/api/v1/generation-status");
  }

  async getManifest(): Promise<ManifestResponse> {
    return this.get<ManifestResponse>("/api/v1/manifest");
  }

  async getDocsList(): Promise<DocsListResponse> {
    return this.get<DocsListResponse>("/api/v1/docs");
  }

  async getDocsContent(path: string): Promise<DocsContentResponse> {
    return this.get<DocsContentResponse>(
      `/api/v1/docs/content?path=${encodeURIComponent(path)}`,
    );
  }

  async getReviewState(): Promise<ReviewStateResponse> {
    return this.get<ReviewStateResponse>("/api/v1/review-state");
  }

  private async get<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      credentials: "include",
    });

    if (!response.ok) {
      let metadataError: MetadataErrorResponse | undefined;
      if (response.status === 422) {
        try {
          const body = (await response.json()) as { kind?: string } & Partial<MetadataErrorResponse>;
          if (body.kind === "metadata_error") {
            metadataError = body as MetadataErrorResponse;
          }
        } catch {
          // body wasn't JSON or didn't match the shape; fall through
        }
      }
      throw new ApiError(response.status, path, metadataError);
    }

    return response.json() as Promise<T>;
  }
}

export interface MetadataErrorEntry {
  file: string;
  line?: number;
  column?: number;
  code: string;
  message: string;
}

export interface MetadataErrorResponse {
  kind: "metadata_error";
  summary: string;
  errors: MetadataErrorEntry[];
}

export class ApiError extends Error {
  readonly status: number;
  readonly path: string;
  readonly metadataError: MetadataErrorResponse | undefined;

  constructor(
    status: number,
    path: string,
    metadataError?: MetadataErrorResponse,
  ) {
    super(`API request failed: ${path} returned ${String(status)}`);
    this.name = "ApiError";
    this.status = status;
    this.path = path;
    this.metadataError = metadataError;
  }
}
