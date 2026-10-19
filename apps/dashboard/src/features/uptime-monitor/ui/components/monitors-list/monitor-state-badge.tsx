import type { UptimeMonitorState } from '@rustrak/client';
import { useEffect, useState } from 'react';
import { useTranslations } from 'use-intl';
import { getUptimeMonitorState } from '@/features/uptime-monitor/api/queries';
import { cn } from '@/shared/lib/utils';
import { Badge } from '@/shared/ui/components/shadcn/badge';

/**
 * A monitor's live up/down state, fetched on its own — separate from the
 * list response because the state machine row can lag the monitor by one
 * scheduler tick (or not exist yet for a monitor created seconds ago).
 */
export function MonitorStateBadge({
  projectId,
  monitorId,
}: {
  projectId: number;
  monitorId: string;
}) {
  const t = useTranslations('uptimeMonitors');
  const [state, setState] = useState<UptimeMonitorState | null | undefined>(
    undefined,
  );

  useEffect(() => {
    let cancelled = false;
    setState(undefined);
    getUptimeMonitorState(projectId, monitorId).then((result) => {
      if (cancelled) return;
      setState(result.success ? result.data : null);
    });
    return () => {
      cancelled = true;
    };
  }, [projectId, monitorId]);

  if (state === undefined) {
    return (
      <Badge variant="secondary" className="text-[10px] font-bold uppercase">
        {t('state.loading')}
      </Badge>
    );
  }

  if (state === null) {
    return (
      <Badge variant="secondary" className="text-[10px] font-bold uppercase">
        {t('state.pending')}
      </Badge>
    );
  }

  const isUp = state.state === 'up';
  const isDown = state.state === 'down';

  return (
    <Badge
      variant={isUp ? 'default' : 'secondary'}
      className={cn(
        'text-[10px] font-bold uppercase tracking-wider',
        isUp &&
          'bg-green-500/10 text-green-500 border-green-500/20 hover:bg-green-500/20',
        isDown &&
          'bg-red-500/10 text-red-500 border-red-500/20 hover:bg-red-500/20',
        (state.state === 'pending_up' || state.state === 'pending_down') &&
          'bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20',
      )}
    >
      {t(`state.${state.state}`)}
    </Badge>
  );
}
