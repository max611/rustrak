import type { Monitor } from '@rustrak/client';
import { Bell, History } from 'lucide-react';
import { useFormatter, useTranslations } from 'use-intl';
import { Button } from '@/shared/ui/components/shadcn/button';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/shared/ui/components/shadcn/card';
import { MonitorSchedule } from './monitor-schedule';
import { MonitorStatusBadge } from './monitor-status-badge';

export function MonitorCard({
  monitor,
  onManageChannels,
  onViewCheckIns,
}: {
  monitor: Monitor;
  onManageChannels: (monitor: Monitor) => void;
  onViewCheckIns: (monitor: Monitor) => void;
}) {
  const t = useTranslations('cronMonitors');
  const format = useFormatter();

  return (
    <Card size="sm">
      <CardHeader className="flex-row items-center justify-between space-y-0">
        <CardTitle className="flex items-center gap-2 text-base">
          <code>{monitor.slug}</code>
          <MonitorStatusBadge status={monitor.status} />
        </CardTitle>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onViewCheckIns(monitor)}
          >
            <History className="mr-1.5 size-3.5" />
            {t('card.checkIns')}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onManageChannels(monitor)}
          >
            <Bell className="mr-1.5 size-3.5" />
            {t('card.channels')}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        <Field label={t('card.schedule')}>
          <MonitorSchedule monitor={monitor} />
        </Field>
        <Field label={t('card.margin')}>
          {monitor.checkin_margin === null
            ? '—'
            : t('card.marginMinutes', { minutes: monitor.checkin_margin })}
        </Field>
        <Field label={t('card.lastCheckIn')}>
          {monitor.last_check_in_at
            ? format.relativeTime(new Date(monitor.last_check_in_at))
            : t('card.never')}
        </Field>
        <Field label={t('card.nextExpected')}>
          {monitor.next_expected_at
            ? format.relativeTime(new Date(monitor.next_expected_at))
            : '—'}
        </Field>
      </CardContent>
    </Card>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
        {label}
      </p>
      <p className="mt-1 text-sm">{children}</p>
    </div>
  );
}
