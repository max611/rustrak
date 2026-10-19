import type {
  CreateUptimeMonitor,
  Result,
  RustrakError,
  UpdateUptimeMonitor,
  UptimeCheckResult,
  UptimeMonitor,
} from '@rustrak/client';
import { createClient } from '@/shared/api/rustrak';

export async function createUptimeMonitor(
  projectId: number,
  input: CreateUptimeMonitor,
): Promise<Result<UptimeMonitor, RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.create(projectId, input);
}

export async function updateUptimeMonitor(
  projectId: number,
  id: string,
  input: UpdateUptimeMonitor,
): Promise<Result<UptimeMonitor, RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.update(projectId, id, input);
}

export async function deleteUptimeMonitor(
  projectId: number,
  id: string,
): Promise<Result<void, RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.delete(projectId, id);
}

export async function triggerUptimeMonitorCheck(
  projectId: number,
  id: string,
): Promise<Result<UptimeCheckResult, RustrakError>> {
  const client = await createClient();
  return client.uptimeMonitors.triggerCheck(projectId, id);
}
