import type { ProjectApiKey } from '@rustrak/client';
import { useMemo } from 'react';
import { useFormatter, useTranslations } from 'use-intl';
import { DataTable } from '@/shared/ui/components/data-table/data-table';
import {
  createAppColumnHelper,
  useAppTable,
} from '@/shared/ui/components/data-table/use-app-table';
import { ApiKeyActions } from './api-key-actions';

type Formatter = ReturnType<typeof useFormatter>;

function lastUsed(
  key: ProjectApiKey,
  t: (key: string) => string,
  format: Formatter,
): string {
  return key.last_used_at
    ? format.relativeTime(new Date(key.last_used_at))
    : t('never');
}

const created = (key: ProjectApiKey, format: Formatter) =>
  format.dateTime(new Date(key.created_at), 'date');

export interface ApiKeyListProps {
  keys: ProjectApiKey[];
  isBusy: (key: ProjectApiKey) => boolean;
  onCopy: (key: ProjectApiKey) => void;
  onDelete: (key: ProjectApiKey) => void;
}

/** The phone layout: one card per key, facts stacked rather than columned. */
export function ApiKeyCards({ keys, isBusy, onCopy, onDelete }: ApiKeyListProps) {
  const t = useTranslations('projectApiKeys');
  const format = useFormatter();
  return (
    <div className="md:hidden space-y-3">
      {keys.map((key) => (
        <div
          key={key.id}
          className="flex items-start justify-between gap-3 rounded-lg border p-3"
        >
          <div className="min-w-0 space-y-1">
            <code className="text-xs font-mono bg-muted px-2 py-1 rounded block w-fit">
              {key.key_prefix}
            </code>
            {key.description && (
              <p className="text-sm truncate">{key.description}</p>
            )}
            <p className="text-xs text-muted-foreground">
              {t('createdOn', { date: created(key, format) })}
              {' · '}
              {key.last_used_at
                ? t('usedOn', { time: lastUsed(key, t, format) })
                : t('neverUsed')}
            </p>
          </div>
          <ApiKeyActions
            apiKey={key}
            busy={isBusy(key)}
            onCopy={onCopy}
            onDelete={onDelete}
          />
        </div>
      ))}
    </div>
  );
}

const helper = createAppColumnHelper<ProjectApiKey>();

/** The desktop layout: the same keys as a table. */
export function ApiKeyTable({ keys, isBusy, onCopy, onDelete }: ApiKeyListProps) {
  const t = useTranslations('projectApiKeys');
  const format = useFormatter();
  const columns = useMemo(
    () =>
      helper.columns([
        helper.accessor('key_prefix', {
          header: t('columns.key'),
          size: 160,
          minSize: 120,
          cell: ({ getValue }) => (
            <code className="rounded bg-muted px-2 py-1 font-mono text-sm">
              {getValue()}
            </code>
          ),
        }),
        helper.accessor('description', {
          header: t('columns.description'),
          minSize: 200,
          meta: { grow: true },
          cell: ({ getValue }) =>
            getValue() || <span className="text-muted-foreground">-</span>,
        }),
        helper.accessor('created_at', {
          id: 'created',
          header: t('columns.created'),
          size: 150,
          minSize: 120,
          cell: ({ row }) => (
            <span className="text-sm whitespace-nowrap">
              {created(row.original, format)}
            </span>
          ),
        }),
        helper.accessor('last_used_at', {
          id: 'last_used',
          header: t('columns.lastUsed'),
          size: 160,
          minSize: 120,
          cell: ({ row }) => (
            <span
              className={
                row.original.last_used_at
                  ? 'text-sm whitespace-nowrap'
                  : 'text-sm whitespace-nowrap text-muted-foreground'
              }
            >
              {lastUsed(row.original, t, format)}
            </span>
          ),
        }),
        helper.display({
          id: 'actions',
          size: 96,
          minSize: 96,
          maxSize: 96,
          header: () => <span className="sr-only">{t('actionsLabel')}</span>,
          cell: ({ row }) => (
            <ApiKeyActions
              apiKey={row.original}
              busy={isBusy(row.original)}
              onCopy={onCopy}
              onDelete={onDelete}
            />
          ),
        }),
      ]),
    [format, isBusy, onCopy, onDelete, t],
  );

  const table = useAppTable({
    data: keys,
    columns,
    getRowId: (key) => String(key.id),
    rowCount: keys.length,
  });

  return <DataTable table={table} className="hidden md:block" />;
}
