import type { ProjectApiKey } from '@rustrak/client';
import { KeySquare, Plus } from 'lucide-react';
import { useState, useTransition } from 'react';
import { toast } from 'sonner';
import { useTranslations } from 'use-intl';
import {
  createProjectApiKey,
  deleteProjectApiKey,
  getProjectApiKey,
} from '@/features/project-api-key/api/mutations';
import { copyToClipboard } from '@/shared/lib/clipboard';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/shared/ui/components/shadcn/alert-dialog';
import { Button } from '@/shared/ui/components/shadcn/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/components/shadcn/card';
import { useRouter } from '@/shared/ui/hooks/use-router';
import { ApiKeyCards, ApiKeyTable } from './api-key-rows';
import { CreateApiKeyDialog } from './create-api-key-dialog';

interface ApiKeysListProps {
  projectId: number;
  initialKeys: ProjectApiKey[];
}

/**
 * Project API keys, mirroring `TokensList`'s shape: a per-project secret
 * instead of a per-user one, same create-once/reveal-again/revoke lifecycle.
 */
export function ApiKeysList({ projectId, initialKeys }: ApiKeysListProps) {
  const t = useTranslations('projectApiKeys');
  const commonT = useTranslations('common');
  const router = useRouter();
  const [isPending, startTransition] = useTransition();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);

  const [busyId, setBusyId] = useState<number | null>(null);
  const [pendingDelete, setPendingDelete] = useState<ProjectApiKey | null>(
    null,
  );

  const closeCreate = () => {
    setIsCreateOpen(false);
    setNewKey(null);
    router.refresh();
  };

  const create = (description: string) => {
    startTransition(async () => {
      const result = await createProjectApiKey(projectId, {
        description: description || undefined,
      });

      if (!result.success) {
        toast.error(t('toasts.createFailed'), {
          description: result.error.message,
        });
        return;
      }

      setNewKey(result.data.key);
      toast.success(t('toasts.created'), {
        description: t('toasts.createdHint'),
      });
    });
  };

  const confirmDelete = () => {
    if (!pendingDelete) return;
    const id = pendingDelete.id;
    setBusyId(id);
    setPendingDelete(null);

    startTransition(async () => {
      const result = await deleteProjectApiKey(projectId, id);

      if (result.success) {
        toast.success(t('toasts.deleted'));
        router.refresh();
      } else {
        toast.error(t('toasts.deleteFailed'), {
          description: result.error.message,
        });
      }

      setBusyId(null);
    });
  };

  const copy = (key: ProjectApiKey) => {
    setBusyId(key.id);
    startTransition(async () => {
      const result = await getProjectApiKey(projectId, key.id);

      if (!result.success) {
        toast.error(t('toasts.getFailed'), {
          description: result.error.message,
        });
        setBusyId(null);
        return;
      }

      if (await copyToClipboard(result.data.key)) {
        toast.success(t('toasts.copied'));
      } else {
        toast.info(t('toasts.copyTitle'), { description: result.data.key });
      }
      setBusyId(null);
    });
  };

  const copyNewKey = async () => {
    if (!newKey) return false;
    if (await copyToClipboard(newKey)) return true;

    toast.info(commonT('clipboardUnavailable'), {
      description: commonT('clipboardUnavailableHint'),
    });
    return false;
  };

  const rowProps = {
    keys: initialKeys,
    isBusy: (key: ProjectApiKey) => isPending || busyId === key.id,
    onCopy: copy,
    onDelete: setPendingDelete,
  };

  return (
    <div className="space-y-6">
      <CreateApiKeyDialog
        open={isCreateOpen}
        onOpenChange={(open) => (open ? setIsCreateOpen(true) : closeCreate())}
        newKey={newKey}
        isPending={isPending}
        onCreate={create}
        onDone={closeCreate}
        onCopyKey={copyNewKey}
      />

      {initialKeys.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12">
            <KeySquare className="size-12 text-muted-foreground/50 mb-4" />
            <p className="text-muted-foreground mb-4">{t('empty.title')}</p>
            <Button variant="outline" onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 size-4" />
              {t('empty.createFirst')}
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>{t('list.title')}</CardTitle>
            <CardDescription>{t('list.description')}</CardDescription>
          </CardHeader>
          <CardContent>
            <ApiKeyCards {...rowProps} />
            <ApiKeyTable {...rowProps} />
          </CardContent>
        </Card>
      )}

      <AlertDialog
        open={pendingDelete !== null}
        onOpenChange={(open) => !open && setPendingDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('deleteDialog.title')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('deleteDialog.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isPending}>
              {t('deleteDialog.cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmDelete}
              disabled={isPending}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isPending
                ? t('deleteDialog.deleting')
                : t('deleteDialog.confirm')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
