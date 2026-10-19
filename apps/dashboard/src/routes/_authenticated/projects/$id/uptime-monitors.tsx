import { createFileRoute } from '@tanstack/react-router';
import { useTranslations } from 'use-intl';
import { listUptimeMonitors } from '@/features/uptime-monitor/api/queries';
import { UptimeMonitorsList } from '@/features/uptime-monitor/ui/components/monitors-list/uptime-monitors-list';
import { translator } from '@/shared/i18n/intl';
import { LoadFailure } from '@/shared/ui/components/load-failure';

export const Route = createFileRoute(
  '/_authenticated/projects/$id/uptime-monitors',
)({
  head: () => ({
    meta: [{ title: translator('uptimeMonitors')('meta.title') }],
  }),
  loader: ({ params }) => listUptimeMonitors(Number.parseInt(params.id, 10)),
  component: UptimeMonitorsPage,
});

function UptimeMonitorsPage() {
  const t = useTranslations('uptimeMonitors');
  const { id } = Route.useParams();
  const monitors = Route.useLoaderData();

  if (!monitors.success) {
    return <LoadFailure error={monitors.error} title={t('loadFailed')} />;
  }

  return (
    <div className="px-4 py-4 md:px-8 md:py-6">
      <div className="mb-6 md:mb-8">
        <h1 className="text-xl font-extrabold tracking-tight md:text-2xl">
          {t('title')}
        </h1>
        <p className="mt-1 text-muted-foreground">{t('subtitle')}</p>
      </div>

      <UptimeMonitorsList
        projectId={Number.parseInt(id, 10)}
        initialMonitors={monitors.data}
      />
    </div>
  );
}
