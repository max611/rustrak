import type { RustrakError } from '../errors.js';
import type { Result } from '../result.js';
import {
  assignUptimeMonitorChannelsSchema,
  createUptimeMonitorSchema,
  updateUptimeMonitorSchema,
  uptimeCheckResultSchema,
  uptimeMonitorSchema,
  uptimeMonitorStateSchema,
} from '../schemas/uptime-monitor.js';
import type {
  AssignUptimeMonitorChannels,
  CreateUptimeMonitor,
  UpdateUptimeMonitor,
  UptimeCheckResult,
  UptimeMonitor,
  UptimeMonitorState,
} from '../types/uptime-monitor.js';
import { BaseResource } from './base.js';

/**
 * Uptime Monitors resource (active HTTP/TCP probing — distinct from
 * `MonitorsResource`'s Sentry Crons check-ins).
 */
export class UptimeMonitorsResource extends BaseResource {
  /**
   * List all uptime monitors for a project.
   */
  async list(
    projectId: number,
  ): Promise<Result<UptimeMonitor[], RustrakError>> {
    return this.request(
      () => this.http.get(`api/projects/${projectId}/uptime-monitors`),
      uptimeMonitorSchema.array(),
    );
  }

  /**
   * Get a single uptime monitor by ID.
   */
  async get(
    projectId: number,
    id: string,
  ): Promise<Result<UptimeMonitor, RustrakError>> {
    return this.request(
      () =>
        this.http.get(`api/projects/${projectId}/uptime-monitors/${id}`),
      uptimeMonitorSchema,
    );
  }

  /**
   * Get a monitor's current state-machine row (`null` until the scheduler's
   * first tick after creation).
   */
  async getState(
    projectId: number,
    id: string,
  ): Promise<Result<UptimeMonitorState | null, RustrakError>> {
    return this.request(
      () =>
        this.http.get(
          `api/projects/${projectId}/uptime-monitors/${id}/state`,
        ),
      uptimeMonitorStateSchema.nullable(),
    );
  }

  /**
   * Create a new uptime monitor.
   */
  async create(
    projectId: number,
    input: CreateUptimeMonitor,
  ): Promise<Result<UptimeMonitor, RustrakError>> {
    const validatedInput = this.validateInput(
      input,
      createUptimeMonitorSchema,
    );
    if (!validatedInput.success) {
      return validatedInput;
    }

    return this.request(
      () =>
        this.http.post(`api/projects/${projectId}/uptime-monitors`, {
          json: validatedInput.data,
        }),
      uptimeMonitorSchema,
    );
  }

  /**
   * Update an uptime monitor (name, url, thresholds, enabled, ...).
   */
  async update(
    projectId: number,
    id: string,
    input: UpdateUptimeMonitor,
  ): Promise<Result<UptimeMonitor, RustrakError>> {
    const validatedInput = this.validateInput(
      input,
      updateUptimeMonitorSchema,
    );
    if (!validatedInput.success) {
      return validatedInput;
    }

    return this.request(
      () =>
        this.http.patch(`api/projects/${projectId}/uptime-monitors/${id}`, {
          json: validatedInput.data,
        }),
      uptimeMonitorSchema,
    );
  }

  /**
   * Delete an uptime monitor.
   */
  async delete(
    projectId: number,
    id: string,
  ): Promise<Result<void, RustrakError>> {
    return this.requestVoid(() =>
      this.http.delete(`api/projects/${projectId}/uptime-monitors/${id}`),
    );
  }

  /**
   * Manually trigger a probe ("Check now") outside the scheduler's interval.
   */
  async triggerCheck(
    projectId: number,
    id: string,
  ): Promise<Result<UptimeCheckResult, RustrakError>> {
    return this.request(
      () =>
        this.http.post(
          `api/projects/${projectId}/uptime-monitors/${id}/check`,
        ),
      uptimeCheckResultSchema,
    );
  }

  /**
   * Replace all alert-integration assignments for a monitor.
   */
  async putChannels(
    projectId: number,
    id: string,
    input: AssignUptimeMonitorChannels,
  ): Promise<Result<void, RustrakError>> {
    const validatedInput = this.validateInput(
      input,
      assignUptimeMonitorChannelsSchema,
    );
    if (!validatedInput.success) {
      return validatedInput;
    }

    return this.requestVoid(() =>
      this.http.put(`api/projects/${projectId}/uptime-monitors/${id}/channels`, {
        json: validatedInput.data,
      }),
    );
  }
}
