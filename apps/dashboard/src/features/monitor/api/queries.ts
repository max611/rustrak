import type {
  AssignMonitorChannels,
  CheckIn,
  ListCheckInsOptions,
  Monitor,
  OffsetPaginatedResponse,
  Result,
  RustrakError,
} from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

/**
 * List all monitors (Sentry Crons) for a project, most-recently-active first.
 */
export async function listMonitors(
  projectId: number,
): Promise<Result<Monitor[], RustrakError>> {
  const client = await createClient();
  return client.monitors.list(projectId);
}

/**
 * List check-ins for a single monitor with offset pagination (newest first).
 */
export async function listMonitorCheckIns(
  projectId: number,
  slug: string,
  options?: ListCheckInsOptions,
): Promise<Result<OffsetPaginatedResponse<CheckIn>, RustrakError>> {
  const client = await createClient();
  return client.monitors.listCheckIns(projectId, slug, options);
}

/**
 * Get the integration ids currently assigned to a monitor.
 */
export async function getMonitorChannels(
  projectId: number,
  slug: string,
): Promise<Result<AssignMonitorChannels, RustrakError>> {
  const client = await createClient();
  return client.monitors.getChannels(projectId, slug);
}
