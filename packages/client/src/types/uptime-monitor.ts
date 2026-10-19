import type { z } from 'zod';
import type {
  assignUptimeMonitorChannelsSchema,
  createUptimeMonitorSchema,
  updateUptimeMonitorSchema,
  uptimeCheckResultSchema,
  uptimeCheckTypeSchema,
  uptimeMonitorSchema,
  uptimeMonitorStateSchema,
} from '../schemas/uptime-monitor.js';

/**
 * Uptime monitor check type
 */
export type UptimeCheckType = z.infer<typeof uptimeCheckTypeSchema>;

/**
 * Uptime monitor resource
 */
export type UptimeMonitor = z.infer<typeof uptimeMonitorSchema>;

/**
 * Request payload for creating an uptime monitor
 */
export type CreateUptimeMonitor = z.infer<typeof createUptimeMonitorSchema>;

/**
 * Request payload for updating an uptime monitor
 */
export type UpdateUptimeMonitor = z.infer<typeof updateUptimeMonitorSchema>;

/**
 * Uptime monitor state machine row
 */
export type UptimeMonitorState = z.infer<typeof uptimeMonitorStateSchema>;

/**
 * Result of a manually triggered check
 */
export type UptimeCheckResult = z.infer<typeof uptimeCheckResultSchema>;

/**
 * Request payload for replacing a monitor's alert-integration assignments
 */
export type AssignUptimeMonitorChannels = z.infer<
  typeof assignUptimeMonitorChannelsSchema
>;
