import { createFileRoute } from '@tanstack/react-router';
import { useTranslations } from 'use-intl';
import { listProjectApiKeys } from '@/features/project-api-key/api/queries';
import { ApiKeysList } from '@/features/project-api-key/ui/components/api-keys-list/api-keys-list';
import { translator } from '@/shared/i18n/intl';
import { LoadFailure } from '@/shared/ui/components/load-failure';

export const Route = createFileRoute(
  '/_authenticated/projects/$id/settings/api-keys',
)({
  head: () => ({
    meta: [{ title: translator('projectApiKeys')('meta.title') }],
  }),
  loader: ({ params }) =>
    listProjectApiKeys(Number.parseInt(params.id, 10)),
  component: ApiKeysPage,
});

function ApiKeysPage() {
  const t = useTranslations('projectApiKeys');
  const { id } = Route.useParams();
  const keys = Route.useLoaderData();

  if (!keys.success) {
    return <LoadFailure error={keys.error} title={t('loadFailed')} />;
  }

  return (
    <>
      <div className="mb-6 md:mb-8">
        <h1 className="text-xl font-extrabold tracking-tight md:text-2xl">
          {t('title')}
        </h1>
        <p className="mt-1 text-muted-foreground">{t('subtitle')}</p>
      </div>

      <ApiKeysList
        projectId={Number.parseInt(id, 10)}
        initialKeys={keys.data}
      />
    </>
  );
}
