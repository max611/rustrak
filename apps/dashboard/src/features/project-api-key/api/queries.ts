import type { ProjectApiKey, Result, RustrakError } from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

/**
 * List all API keys for a project (masked).
 * The full key is only returned by `create` or `getProjectApiKey`.
 */
export async function listProjectApiKeys(
  projectId: number,
): Promise<Result<ProjectApiKey[], RustrakError>> {
  const client = await createClient();
  return client.projectApiKeys.list(projectId);
}
