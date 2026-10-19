import type { UptimeMonitor } from '@rustrak/client';
import { useMemo } from 'react';
import { useTranslations } from 'use-intl';
import { DataTable } from '@/shared/ui/components/data-table/data-table';
import {
  createAppColumnHelper,
  useAppTable,
} from '@/shared/ui/components/data-table/use-app-table';
import { Badge } from '@/shared/ui/components/shadcn/badge';
import { MonitorActions } from './monitor-actions';
import { MonitorStateBadge } from './monitor-state-badge';

export interface MonitorListProps {
  projectId: number;
  monitors: UptimeMonitor[];
  isBusy: (monitor: UptimeMonitor) => boolean;
  onCheckNow: (monitor: UptimeMonitor) => void;
  onEdit: (monitor: UptimeMonitor) => void;
  onDelete: (monitor: UptimeMonitor) => void;
}

/** The phone layout: one card per monitor, facts stacked rather than columned. */
export function MonitorCards({
  projectId,
  monitors,
  isBusy,
  onCheckNow,
  onEdit,
  onDelete,
}: MonitorListProps) {
  const t = useTranslations('uptimeMonitors');
  return (
    <div className="md:hidden space-y-3">
      {monitors.map((monitor) => (
        <div
          key={monitor.id}
          className="flex items-start justify-between gap-3 rounded-lg border p-3"
        >
          <div className="min-w-0 space-y-1">
            <div className="flex items-center gap-2">
              <p className="text-sm font-medium truncate">{monitor.name}</p>
              <MonitorStateBadge projectId={projectId} monitorId={monitor.id} />
              {!monitor.enabled && (
                <Badge variant="secondary" className="text-[10px] uppercase">
                  {t('disabled')}
                </Badge>
              )}
            </div>
            <code className="block truncate text-xs text-muted-foreground">
              {monitor.url}
            </code>
            <p className="text-xs text-muted-foreground">
              {t('columns.interval')}: {monitor.interval_secs}s
            </p>
          </div>
          <MonitorActions
            monitor={monitor}
            busy={isBusy(monitor)}
            onCheckNow={onCheckNow}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        </div>
      ))}
    </div>
  );
}

const helper = createAppColumnHelper<UptimeMonitor>();

/** The desktop layout: the same monitors as a table. */
export function MonitorTable({
  projectId,
  monitors,
  isBusy,
  onCheckNow,
  onEdit,
  onDelete,
}: MonitorListProps) {
  const t = useTranslations('uptimeMonitors');
  const columns = useMemo(
    () =>
      helper.columns([
        helper.accessor('name', {
          header: t('columns.name'),
          minSize: 140,
          meta: { grow: true },
          cell: ({ row }) => (
            <div className="flex items-center gap-2">
              <span className="font-medium">{row.original.name}</span>
              {!row.original.enabled && (
                <Badge variant="secondary" className="text-[10px] uppercase">
                  {t('disabled')}
                </Badge>
              )}
            </div>
          ),
        }),
        helper.accessor('url', {
          header: t('columns.url'),
          minSize: 200,
          meta: { grow: true },
          cell: ({ getValue }) => (
            <code className="truncate text-xs text-muted-foreground">
              {getValue()}
            </code>
          ),
        }),
        helper.display({
          id: 'state',
          header: t('columns.state'),
          size: 120,
          minSize: 100,
          cell: ({ row }) => (
            <MonitorStateBadge
              projectId={projectId}
              monitorId={row.original.id}
            />
          ),
        }),
        helper.accessor('interval_secs', {
          id: 'interval',
          header: t('columns.interval'),
          size: 100,
          minSize: 80,
          cell: ({ getValue }) => (
            <span className="text-sm whitespace-nowrap">{getValue()}s</span>
          ),
        }),
        helper.display({
          id: 'actions',
          size: 128,
          minSize: 128,
          maxSize: 128,
          header: () => <span className="sr-only">{t('actionsLabel')}</span>,
          cell: ({ row }) => (
            <MonitorActions
              monitor={row.original}
              busy={isBusy(row.original)}
              onCheckNow={onCheckNow}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ),
        }),
      ]),
    [projectId, isBusy, onCheckNow, onEdit, onDelete, t],
  );

  const table = useAppTable({
    data: monitors,
    columns,
    getRowId: (monitor) => monitor.id,
    rowCount: monitors.length,
  });

  return <DataTable table={table} className="hidden md:block" />;
}
