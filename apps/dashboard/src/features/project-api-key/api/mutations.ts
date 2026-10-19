import type {
  CreateProjectApiKey,
  ProjectApiKeyCreated,
  Result,
  RustrakError,
} from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

/**
 * Create a new project API key.
 * The full key is only returned once during creation - save it immediately.
 */
export async function createProjectApiKey(
  projectId: number,
  input: CreateProjectApiKey,
): Promise<Result<ProjectApiKeyCreated, RustrakError>> {
  const client = await createClient();
  return client.projectApiKeys.create(projectId, input);
}

/**
 * Revoke a project API key.
 */
export async function deleteProjectApiKey(
  projectId: number,
  id: number,
): Promise<Result<void, RustrakError>> {
  const client = await createClient();
  return client.projectApiKeys.delete(projectId, id);
}

/**
 * Get the full key value by ID (reveals it again, same as the auth-token
 * convention).
 */
export async function getProjectApiKey(
  projectId: number,
  id: number,
): Promise<Result<ProjectApiKeyCreated, RustrakError>> {
  const client = await createClient();
  return client.projectApiKeys.get(projectId, id);
}
