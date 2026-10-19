import type {
  Result,
  RustrakError,
  UptimeMonitor,
  UptimeMonitorState,
} from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

/**
 * List all uptime monitors for a project.
 */
export async function listUptimeMonitors(
  projectId: number,
): Promise<Result<UptimeMonitor[], RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.list(projectId);
}

/**
 * Get a monitor's current state-machine row (`null` until its first tick).
 */
export async function getUptimeMonitorState(
  projectId: number,
  id: string,
): Promise<Result<UptimeMonitorState | null, RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.getState(projectId, id);
}
