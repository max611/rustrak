import { createFileRoute } from '@tanstack/react-router';
import { useTranslations } from 'use-intl';
import { listIntegrations } from '@/features/alert/api/queries';
import { listMonitors } from '@/features/monitor/api/queries';
import { MonitorsList } from '@/features/monitor/ui/components/monitors-list/monitors-list';
import { translator } from '@/shared/i18n/intl';
import { loadAll } from '@/shared/lib/results';
import { LoadFailure } from '@/shared/ui/components/load-failure';

export const Route = createFileRoute('/_authenticated/projects/$id/monitors')({
  head: () => ({
    meta: [{ title: translator('cronMonitors')('meta.title') }],
  }),
  loader: ({ params }) => {
    const projectId = Number.parseInt(params.id, 10);
    return loadAll([listMonitors(projectId), listIntegrations()]);
  },
  component: MonitorsPage,
});

function MonitorsPage() {
  const t = useTranslations('cronMonitors');
  const { id } = Route.useParams();
  const data = Route.useLoaderData();

  if (!data.success) {
    return <LoadFailure error={data.error} title={t('loadFailed')} />;
  }

  const [monitors, integrations] = data.data;

  return (
    <div className="px-4 py-4 md:px-8 md:py-6">
      <div className="mb-6 md:mb-8">
        <h1 className="text-xl font-extrabold tracking-tight md:text-2xl">
          {t('title')}
        </h1>
        <p className="mt-1 text-muted-foreground">{t('subtitle')}</p>
      </div>

      <MonitorsList
        projectId={Number.parseInt(id, 10)}
        monitors={monitors}
        integrations={integrations}
      />
    </div>
  );
}
