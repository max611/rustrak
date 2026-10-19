import type { Monitor } from '@rustrak/client';
import { useTranslations } from 'use-intl';

/** Renders a monitor's schedule as "every 10 minutes" or a raw crontab string. */
export function MonitorSchedule({ monitor }: { monitor: Monitor }) {
  const t = useTranslations('cronMonitors');

  if (!monitor.schedule_type) {
    return <span className="text-muted-foreground">{t('schedule.unknown')}</span>;
  }

  if (monitor.schedule_type === 'crontab') {
    return <code className="text-xs">{monitor.schedule_value}</code>;
  }

  return (
    <span>
      {t('schedule.interval', {
        value: monitor.schedule_value ?? '?',
        unit: monitor.schedule_unit ?? '',
      })}
    </span>
  );
}
