import type { AlertIntegration, Monitor } from '@rustrak/client';
import { Radio } from 'lucide-react';
import { useState } from 'react';
import { useTranslations } from 'use-intl';
import {
  Card,
  CardContent,
} from '@/shared/ui/components/shadcn/card';
import { MonitorCard } from './monitor-card';
import { MonitorChannelsDialog } from './monitor-channels-dialog';
import { MonitorCheckInsDialog } from './monitor-checkins-dialog';

interface MonitorsListProps {
  projectId: number;
  monitors: Monitor[];
  integrations: AlertIntegration[];
}

export function MonitorsList({
  projectId,
  monitors,
  integrations,
}: MonitorsListProps) {
  const t = useTranslations('cronMonitors');
  const [channelsMonitor, setChannelsMonitor] = useState<Monitor | null>(null);
  const [checkInsMonitor, setCheckInsMonitor] = useState<Monitor | null>(null);

  if (monitors.length === 0) {
    return (
      <Card className="border-dashed">
        <CardContent className="flex flex-col items-center justify-center py-12">
          <Radio className="size-12 text-muted-foreground/50 mb-4" />
          <p className="text-muted-foreground">{t('empty.title')}</p>
          <p className="mt-1 max-w-md text-center text-xs text-muted-foreground">
            {t('empty.description')}
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {monitors.map((monitor) => (
        <MonitorCard
          key={monitor.id}
          monitor={monitor}
          onManageChannels={setChannelsMonitor}
          onViewCheckIns={setCheckInsMonitor}
        />
      ))}

      <MonitorChannelsDialog
        monitor={channelsMonitor}
        integrations={integrations}
        projectId={projectId}
        onOpenChange={(open) => !open && setChannelsMonitor(null)}
        onSaved={() => setChannelsMonitor(null)}
      />

      <MonitorCheckInsDialog
        monitor={checkInsMonitor}
        projectId={projectId}
        onOpenChange={(open) => !open && setCheckInsMonitor(null)}
      />
    </div>
  );
}
