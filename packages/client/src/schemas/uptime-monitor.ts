import { z } from 'zod';
import { dateTimeSchema, uuidSchema } from './common.js';

/**
 * Uptime monitor check type (active HTTP/TCP probing — distinct from
 * Sentry Crons' `Monitor`/`CheckIn`, which are heartbeat-style).
 */
export const uptimeCheckTypeSchema = z.enum(['http', 'tcp']);

/**
 * Uptime monitor resource
 */
export const uptimeMonitorSchema = z.object({
  id: uuidSchema,
  project_id: z.number().int(),
  name: z.string(),
  check_type: z.string(),
  url: z.string(),
  interval_secs: z.number().int(),
  timeout_secs: z.number().int(),
  expected_status: z.number().int().nullable(),
  fail_threshold: z.number().int(),
  recovery_threshold: z.number().int(),
  repeat_interval_secs: z.number().int(),
  enabled: z.boolean(),
  created_at: dateTimeSchema,
  updated_at: dateTimeSchema,
});

/**
 * Create uptime monitor request schema
 */
export const createUptimeMonitorSchema = z.object({
  name: z.string(),
  check_type: uptimeCheckTypeSchema,
  url: z.string(),
  interval_secs: z.number().int().optional(),
  timeout_secs: z.number().int().optional(),
  expected_status: z.number().int().optional(),
  fail_threshold: z.number().int().optional(),
  recovery_threshold: z.number().int().optional(),
  repeat_interval_secs: z.number().int().optional(),
});

/**
 * Update uptime monitor request schema
 */
export const updateUptimeMonitorSchema = z.object({
  name: z.string().optional(),
  url: z.string().optional(),
  interval_secs: z.number().int().optional(),
  timeout_secs: z.number().int().optional(),
  expected_status: z.number().int().optional(),
  fail_threshold: z.number().int().optional(),
  recovery_threshold: z.number().int().optional(),
  repeat_interval_secs: z.number().int().optional(),
  enabled: z.boolean().optional(),
});

/**
 * Uptime monitor state machine row — absent until the scheduler's first
 * tick, so the resource method may resolve this schema or `null`.
 */
export const uptimeMonitorStateSchema = z.object({
  monitor_id: uuidSchema,
  state: z.enum(['up', 'pending_down', 'down', 'pending_up']),
  fail_counter: z.number().int(),
  recovery_counter: z.number().int(),
  last_check_at: dateTimeSchema.nullable(),
  next_check_at: dateTimeSchema,
  alerted_down_at: dateTimeSchema.nullable(),
  last_alerted_at: dateTimeSchema.nullable(),
  alert_count: z.number().int(),
  incident_id: uuidSchema.nullable(),
});

/**
 * Result of a manually triggered check ("Check now")
 */
export const uptimeCheckResultSchema = z.object({
  ok: z.boolean(),
  latency_ms: z.number().int(),
  error: z.string().nullable(),
});

/**
 * Request schema for replacing a monitor's alert-integration assignments
 */
export const assignUptimeMonitorChannelsSchema = z.object({
  integration_ids: z.array(z.number().int()),
});
