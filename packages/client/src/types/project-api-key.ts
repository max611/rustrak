import type { z } from 'zod';
import type {
  createProjectApiKeySchema,
  projectApiKeyCreatedSchema,
  projectApiKeySchema,
} from '../schemas/project-api-key.js';

/**
 * Project API key resource from list endpoint (masked)
 */
export type ProjectApiKey = z.infer<typeof projectApiKeySchema>;

/**
 * Project API key resource when created or revealed (full key value)
 */
export type ProjectApiKeyCreated = z.infer<typeof projectApiKeyCreatedSchema>;

/**
 * Request payload for creating a project API key
 */
export type CreateProjectApiKey = z.infer<typeof createProjectApiKeySchema>;
