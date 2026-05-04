/** Generated from docs/dev/api/openapi.yaml — do not edit manually. */

import type {
  DomainsResponse,
  EntitiesResponse,
  GraphResponse,
  HealthResponse,
  ModulesResponse,
  OpenQuestionsResponse,
  ProjectResponse,
  RelationshipsResponse,
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

  private async get<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      credentials: "include",
    });

    if (!response.ok) {
      throw new ApiError(response.status, path);
    }

    return response.json() as Promise<T>;
  }
}

export class ApiError extends Error {
  readonly status: number;
  readonly path: string;

  constructor(status: number, path: string) {
    super(`API request failed: ${path} returned ${String(status)}`);
    this.name = "ApiError";
    this.status = status;
    this.path = path;
  }
}
