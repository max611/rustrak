import type { AlertIntegration, Monitor } from '@rustrak/client';
import { Loader2 } from 'lucide-react';
import { useEffect, useState, useTransition } from 'react';
import { toast } from 'sonner';
import { useTranslations } from 'use-intl';
import { putMonitorChannels } from '@/features/monitor/api/mutations';
import { getMonitorChannels } from '@/features/monitor/api/queries';
import { cn } from '@/shared/lib/utils';
import { ProviderIcon } from '@/shared/ui/components/provider-icon';
import { Button } from '@/shared/ui/components/shadcn/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/components/shadcn/dialog';
import { Switch } from '@/shared/ui/components/shadcn/switch';

/**
 * Which channels alert when this monitor misses a check-in or times out.
 *
 * Fetches the current assignment on open rather than trusting a prop: `PUT`
 * replaces the whole set, so showing anything other than what the server
 * actually has risks silently dropping an assignment nobody meant to remove.
 */
export function MonitorChannelsDialog({
  monitor,
  integrations,
  projectId,
  onOpenChange,
  onSaved,
}: {
  monitor: Monitor | null;
  integrations: AlertIntegration[];
  projectId: number;
  onOpenChange: (open: boolean) => void;
  onSaved: () => void;
}) {
  const t = useTranslations('cronMonitors');
  const [isPending, startTransition] = useTransition();
  const [isLoading, setIsLoading] = useState(false);
  const [selected, setSelected] = useState<Set<number>>(new Set());

  useEffect(() => {
    if (!monitor) return;
    let cancelled = false;
    setIsLoading(true);
    getMonitorChannels(projectId, monitor.slug).then((result) => {
      if (cancelled) return;
      setIsLoading(false);
      if (result.success) {
        setSelected(new Set(result.data.integration_ids));
      } else {
        toast.error(t('channels.loadFailed'), {
          description: result.error.message,
        });
      }
    });
    return () => {
      cancelled = true;
    };
  }, [monitor, projectId, t]);

  const toggle = (id: number, on: boolean) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (on) next.add(id);
      else next.delete(id);
      return next;
    });
  };

  const save = () => {
    if (!monitor) return;
    startTransition(async () => {
      const result = await putMonitorChannels(
        projectId,
        monitor.slug,
        Array.from(selected),
      );

      if (!result.success) {
        toast.error(t('channels.saveFailed'), {
          description: result.error.message,
        });
        return;
      }

      toast.success(t('channels.saved'));
      onSaved();
    });
  };

  return (
    <Dialog open={monitor !== null} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('channels.title')}</DialogTitle>
          <DialogDescription>
            {monitor ? t('channels.description', { slug: monitor.slug }) : null}
          </DialogDescription>
        </DialogHeader>

        <div className="py-2">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="size-5 animate-spin text-muted-foreground" />
            </div>
          ) : integrations.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t('channels.noIntegrations')}
            </p>
          ) : (
            <div className="space-y-2">
              {integrations.map((integration) => {
                const isSelected = selected.has(integration.id);
                return (
                  <div
                    key={integration.id}
                    className={cn(
                      'flex items-center justify-between rounded-lg border bg-card px-3 py-2.5 transition-colors',
                      isSelected ? 'border-primary/50' : 'border-border',
                    )}
                  >
                    <div className="flex items-center gap-2.5">
                      <div
                        className={cn(
                          'flex size-7 items-center justify-center rounded-md transition-colors',
                          isSelected
                            ? 'bg-primary/10 text-primary'
                            : 'bg-muted text-muted-foreground',
                        )}
                      >
                        <ProviderIcon
                          type={integration.provider_type}
                          className="size-3.5"
                        />
                      </div>
                      <div>
                        <p className="text-sm font-medium leading-none">
                          {integration.name}
                        </p>
                        <p className="text-[10px] text-muted-foreground mt-0.5 capitalize">
                          {integration.provider_type}
                        </p>
                      </div>
                    </div>
                    <Switch
                      checked={isSelected}
                      onCheckedChange={(on) => toggle(integration.id, on)}
                      disabled={isPending}
                      size="sm"
                    />
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isPending}
          >
            {t('channels.cancel')}
          </Button>
          <Button onClick={save} disabled={isPending || isLoading}>
            {isPending ? t('channels.saving') : t('channels.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
