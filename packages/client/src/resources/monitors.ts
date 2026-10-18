import {
  checkInSchema,
  monitorsListResponseSchema,
  offsetPaginatedResponseSchema,
} from '../schemas/index.js';
import type {
  CheckIn,
  ListCheckInsOptions,
  Monitor,
  OffsetPaginatedResponse,
} from '../types/index.js';
import { mapResult, type Result } from '../result.js';
import type { RustrakError } from '../errors.js';
import { BaseResource } from './base.js';

/**
 * Monitors API resource (Sentry Crons — scheduled job monitoring)
 */
export class MonitorsResource extends BaseResource {
  /**
   * List all monitors for a project, most-recently-active first.
   */
  async list(projectId: number): Promise<Result<Monitor[], RustrakError>> {
    const data = await this.http
      .get(`api/projects/${projectId}/monitors`)
      .json();

    return mapResult(
      this.validate(data, monitorsListResponseSchema),
      (parsed) => parsed.monitors,
    );
  }

  /**
   * List check-ins for a single monitor (by slug) with offset-based
   * pagination (newest first).
   */
  async listCheckIns(
    projectId: number,
    slug: string,
    options?: ListCheckInsOptions,
  ): Promise<Result<OffsetPaginatedResponse<CheckIn>, RustrakError>> {
    const searchParams: Record<string, string> = {};

    if (options?.page) {
      searchParams.page = String(options.page);
    }
    if (options?.per_page) {
      searchParams.per_page = String(options.per_page);
    }

    const data = await this.http
      .get(`api/projects/${projectId}/monitors/${slug}/checkins`, {
        searchParams,
      })
      .json();

    return this.validate(data, offsetPaginatedResponseSchema(checkInSchema));
  }
}
