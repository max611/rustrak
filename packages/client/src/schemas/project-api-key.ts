import { z } from 'zod';
import { dateTimeSchema } from './common.js';

/**
 * Project API key response schema from list endpoint (masked)
 */
export const projectApiKeySchema = z.object({
  id: z.number().int(),
  project_id: z.number().int(),
  key_prefix: z.string(),
  description: z.string().nullable(),
  created_at: dateTimeSchema,
  last_used_at: dateTimeSchema.nullable(),
});

/**
 * Project API key created/revealed response schema (full key value)
 */
export const projectApiKeyCreatedSchema = z.object({
  id: z.number().int(),
  project_id: z.number().int(),
  key: z.string(),
  description: z.string().nullable(),
  created_at: dateTimeSchema,
});

/**
 * Create project API key request schema
 */
export const createProjectApiKeySchema = z.object({
  description: z.string().optional(),
});
