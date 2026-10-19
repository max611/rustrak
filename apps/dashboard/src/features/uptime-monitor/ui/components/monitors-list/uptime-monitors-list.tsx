import type { UptimeMonitor } from '@rustrak/client';
import { Plus, Radio } from 'lucide-react';
import { useState, useTransition } from 'react';
import { toast } from 'sonner';
import { useTranslations } from 'use-intl';
import {
  createUptimeMonitor,
  deleteUptimeMonitor,
  triggerUptimeMonitorCheck,
  updateUptimeMonitor,
} from '@/features/uptime-monitor/api/mutations';
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
import { MonitorFormDialog } from './monitor-form-dialog';
import { MonitorCards, MonitorTable } from './monitor-rows';

interface UptimeMonitorsListProps {
  projectId: number;
  initialMonitors: UptimeMonitor[];
}

export function UptimeMonitorsList({
  projectId,
  initialMonitors,
}: UptimeMonitorsListProps) {
  const t = useTranslations('uptimeMonitors');
  const router = useRouter();
  const [isPending, startTransition] = useTransition();
  const [busyId, setBusyId] = useState<string | null>(null);

  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingMonitor, setEditingMonitor] = useState<UptimeMonitor | null>(
    null,
  );
  const [formError, setFormError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<UptimeMonitor | null>(
    null,
  );

  const openCreate = () => {
    setEditingMonitor(null);
    setFormError(null);
    setIsFormOpen(true);
  };

  const openEdit = (monitor: UptimeMonitor) => {
    setEditingMonitor(monitor);
    setFormError(null);
    setIsFormOpen(true);
  };

  const closeForm = () => {
    setIsFormOpen(false);
    setEditingMonitor(null);
    setFormError(null);
  };

  const submitForm = (values: {
    name: string;
    check_type: 'http' | 'tcp';
    url: string;
    interval_secs?: number;
    timeout_secs?: number;
    expected_status?: number;
    fail_threshold?: number;
    recovery_threshold?: number;
    repeat_interval_secs?: number;
    enabled?: boolean;
  }) => {
    startTransition(async () => {
      const result = editingMonitor
        ? await updateUptimeMonitor(projectId, editingMonitor.id, values)
        : await createUptimeMonitor(projectId, values);

      if (!result.success) {
        setFormError(result.error.message);
        return;
      }

      toast.success(
        editingMonitor ? t('toasts.updated') : t('toasts.created'),
      );
      closeForm();
      router.refresh();
    });
  };

  const checkNow = (monitor: UptimeMonitor) => {
    setBusyId(monitor.id);
    startTransition(async () => {
      const result = await triggerUptimeMonitorCheck(projectId, monitor.id);
      setBusyId(null);

      if (!result.success) {
        toast.error(t('toasts.checkFailed'), {
          description: result.error.message,
        });
        return;
      }

      if (result.data.ok) {
        toast.success(t('toasts.checkOk', { ms: result.data.latency_ms }));
      } else {
        toast.error(t('toasts.checkFailedResult'), {
          description: result.data.error ?? undefined,
        });
      }
    });
  };

  const confirmDelete = () => {
    if (!pendingDelete) return;
    const monitor = pendingDelete;
    setBusyId(monitor.id);
    setPendingDelete(null);

    startTransition(async () => {
      const result = await deleteUptimeMonitor(projectId, monitor.id);

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

  const rowProps = {
    projectId,
    monitors: initialMonitors,
    isBusy: (monitor: UptimeMonitor) => isPending && busyId === monitor.id,
    onCheckNow: checkNow,
    onEdit: openEdit,
    onDelete: setPendingDelete,
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-end">
        <Button
          className="font-bold uppercase tracking-wider"
          onClick={openCreate}
        >
          <Plus className="mr-2 size-4" />
          {t('create.trigger')}
        </Button>
      </div>

      <MonitorFormDialog
        key={editingMonitor?.id ?? 'new'}
        open={isFormOpen}
        onOpenChange={(open) => (open ? setIsFormOpen(true) : closeForm())}
        existingMonitor={editingMonitor}
        isPending={isPending}
        error={formError}
        onSubmit={submitForm}
      />

      {initialMonitors.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Radio className="size-12 text-muted-foreground/50 mb-4" />
            <p className="text-muted-foreground mb-4">{t('empty.title')}</p>
            <Button variant="outline" onClick={openCreate}>
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
            <MonitorCards {...rowProps} />
            <MonitorTable {...rowProps} />
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
              {t('deleteDialog.description', {
                name: pendingDelete?.name ?? '',
              })}
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
