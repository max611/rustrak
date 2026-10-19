import type { ProjectApiKey } from '@rustrak/client';
import { Copy, Trash2 } from 'lucide-react';
import { useTranslations } from 'use-intl';
import { Button } from '@/shared/ui/components/shadcn/button';

/** Copy and delete for one key, shared by the phone card and the desktop row. */
export function ApiKeyActions({
  apiKey,
  busy,
  onCopy,
  onDelete,
}: {
  apiKey: ProjectApiKey;
  busy: boolean;
  onCopy: (key: ProjectApiKey) => void;
  onDelete: (key: ProjectApiKey) => void;
}) {
  const t = useTranslations('projectApiKeys');
  const label = apiKey.description || apiKey.key_prefix;

  return (
    <div className="flex shrink-0 items-center gap-1">
      <Button
        variant="ghost"
        size="icon"
        onClick={() => onCopy(apiKey)}
        disabled={busy}
        aria-label={t('copyKeyLabel', { label })}
      >
        <Copy className="size-4" />
      </Button>
      <Button
        variant="destructive"
        size="icon"
        onClick={() => onDelete(apiKey)}
        disabled={busy}
        aria-label={t('deleteKeyLabel', { label })}
      >
        <Trash2 className="size-4" />
      </Button>
    </div>
  );
}
