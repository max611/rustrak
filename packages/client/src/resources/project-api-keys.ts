import type { RustrakError } from '../errors.js';
import type { Result } from '../result.js';
import {
  createProjectApiKeySchema,
  projectApiKeyCreatedSchema,
  projectApiKeySchema,
} from '../schemas/project-api-key.js';
import type {
  CreateProjectApiKey,
  ProjectApiKey,
  ProjectApiKeyCreated,
} from '../types/project-api-key.js';
import { BaseResource } from './base.js';

/**
 * Project API Keys resource — a secret scoped to exactly one project,
 * authenticating the /api/v1/* surface (feature parity with BugWatch's
 * per-project `api_key`).
 */
export class ProjectApiKeysResource extends BaseResource {
  /**
   * List all API keys for a project (masked)
   */
  async list(
    projectId: number,
  ): Promise<Result<ProjectApiKey[], RustrakError>> {
    return this.request(
      () => this.http.get(`api/projects/${projectId}/api-keys`),
      projectApiKeySchema.array(),
    );
  }

  /**
   * Get a single API key by ID (full key value, revealed again on demand).
   */
  async get(
    projectId: number,
    id: number,
  ): Promise<Result<ProjectApiKeyCreated, RustrakError>> {
    return this.request(
      () => this.http.get(`api/projects/${projectId}/api-keys/${id}`),
      projectApiKeyCreatedSchema,
    );
  }

  /**
   * Create a new API key for a project.
   * Note: The full key is only returned once during creation (and via `get`).
   */
  async create(
    projectId: number,
    input: CreateProjectApiKey,
  ): Promise<Result<ProjectApiKeyCreated, RustrakError>> {
    const validatedInput = this.validateInput(
      input,
      createProjectApiKeySchema,
    );
    if (!validatedInput.success) {
      return validatedInput;
    }

    return this.request(
      () =>
        this.http.post(`api/projects/${projectId}/api-keys`, {
          json: validatedInput.data,
        }),
      projectApiKeyCreatedSchema,
    );
  }

  /**
   * Revoke an API key.
   */
  async delete(
    projectId: number,
    id: number,
  ): Promise<Result<void, RustrakError>> {
    return this.requestVoid(() =>
      this.http.delete(`api/projects/${projectId}/api-keys/${id}`),
    );
  }
}
