import { useTranslations } from 'use-intl';
import { cn } from '@/shared/lib/utils';
import { Badge } from '@/shared/ui/components/shadcn/badge';

/** A monitor's derived status: active/ok/error/missed/timeout/disabled. */
export function MonitorStatusBadge({ status }: { status: string }) {
  const t = useTranslations('cronMonitors');
  const isOk = status === 'ok' || status === 'active';
  const isBad = status === 'missed' || status === 'timeout' || status === 'error';

  return (
    <Badge
      variant={isOk ? 'default' : 'secondary'}
      className={cn(
        'text-[10px] font-bold uppercase tracking-wider',
        isOk &&
          'bg-green-500/10 text-green-500 border-green-500/20 hover:bg-green-500/20',
        isBad &&
          'bg-red-500/10 text-red-500 border-red-500/20 hover:bg-red-500/20',
      )}
    >
      {t(`status.${status}`)}
    </Badge>
  );
}
