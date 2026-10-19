import type { UptimeMonitor } from '@rustrak/client';
import { Pencil, Trash2, Zap } from 'lucide-react';
import { useTranslations } from 'use-intl';
import { Button } from '@/shared/ui/components/shadcn/button';

export function MonitorActions({
  monitor,
  busy,
  onCheckNow,
  onEdit,
  onDelete,
}: {
  monitor: UptimeMonitor;
  busy: boolean;
  onCheckNow: (monitor: UptimeMonitor) => void;
  onEdit: (monitor: UptimeMonitor) => void;
  onDelete: (monitor: UptimeMonitor) => void;
}) {
  const t = useTranslations('uptimeMonitors');

  return (
    <div className="flex shrink-0 items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        onClick={() => onCheckNow(monitor)}
        disabled={busy}
        aria-label={t('checkNowLabel', { name: monitor.name })}
      >
        <Zap className="size-4" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        onClick={() => onEdit(monitor)}
        disabled={busy}
        aria-label={t('editLabel', { name: monitor.name })}
      >
        <Pencil className="size-4" />
      </Button>
      <Button
        variant="destructive"
        size="icon"
        onClick={() => onDelete(monitor)}
        disabled={busy}
        aria-label={t('deleteLabel', { name: monitor.name })}
      >
        <Trash2 className="size-4" />
      </Button>
    </div>
  );
}
