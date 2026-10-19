import type { Result, RustrakError } from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

/**
 * Replaces all alert-integration assignments for a monitor.
 */
export async function putMonitorChannels(
  projectId: number,
  slug: string,
  integrationIds: number[],
): Promise<Result<void, RustrakError>> {
  const client = await createClient();
  return client.monitors.putChannels(projectId, slug, {
    integration_ids: integrationIds,
  });
}
