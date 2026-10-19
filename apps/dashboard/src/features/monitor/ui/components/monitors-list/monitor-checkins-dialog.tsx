import type { CheckIn, Monitor } from '@rustrak/client';
import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { useFormatter, useTranslations } from 'use-intl';
import { listMonitorCheckIns } from '@/features/monitor/api/queries';
import { Button } from '@/shared/ui/components/shadcn/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/components/shadcn/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/shared/ui/components/shadcn/table';
import { MonitorStatusBadge } from './monitor-status-badge';

const PER_PAGE = 20;

export function MonitorCheckInsDialog({
  monitor,
  projectId,
  onOpenChange,
}: {
  monitor: Monitor | null;
  projectId: number;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useTranslations('cronMonitors');
  const format = useFormatter();
  const [page, setPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [checkIns, setCheckIns] = useState<CheckIn[]>([]);
  const [hasMore, setHasMore] = useState(false);

  useEffect(() => {
    setPage(1);
  }, [monitor]);

  useEffect(() => {
    if (!monitor) return;
    let cancelled = false;
    setIsLoading(true);
    listMonitorCheckIns(projectId, monitor.slug, {
      page,
      per_page: PER_PAGE,
    }).then((result) => {
      if (cancelled) return;
      setIsLoading(false);
      if (result.success) {
        setCheckIns(result.data.items);
        setHasMore(page * PER_PAGE < result.data.total_count);
      } else {
        toast.error(t('checkIns.loadFailed'), {
          description: result.error.message,
        });
      }
    });
    return () => {
      cancelled = true;
    };
  }, [monitor, projectId, page, t]);

  return (
    <Dialog open={monitor !== null} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t('checkIns.title')}</DialogTitle>
          <DialogDescription>
            {monitor ? t('checkIns.description', { slug: monitor.slug }) : null}
          </DialogDescription>
        </DialogHeader>

        <div className="py-2">
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="size-5 animate-spin text-muted-foreground" />
            </div>
          ) : checkIns.length === 0 ? (
            <p className="py-8 text-center text-sm text-muted-foreground">
              {t('checkIns.empty')}
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('checkIns.columns.status')}</TableHead>
                  <TableHead>{t('checkIns.columns.duration')}</TableHead>
                  <TableHead>{t('checkIns.columns.environment')}</TableHead>
                  <TableHead>{t('checkIns.columns.timestamp')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {checkIns.map((checkIn) => (
                  <TableRow key={checkIn.id}>
                    <TableCell>
                      <MonitorStatusBadge status={checkIn.status} />
                    </TableCell>
                    <TableCell className="text-sm tabular-nums">
                      {checkIn.duration === null
                        ? '—'
                        : t('checkIns.durationSeconds', {
                            seconds: checkIn.duration.toFixed(1),
                          })}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {checkIn.environment ?? '—'}
                    </TableCell>
                    <TableCell className="text-sm whitespace-nowrap">
                      {format.dateTime(new Date(checkIn.timestamp), 'precise')}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </div>

        <div className="flex items-center justify-between">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page <= 1 || isLoading}
          >
            <ChevronLeft className="mr-1 size-4" />
            {t('checkIns.prev')}
          </Button>
          <span className="text-xs text-muted-foreground">
            {t('checkIns.page', { page })}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => p + 1)}
            disabled={!hasMore || isLoading}
          >
            {t('checkIns.next')}
            <ChevronRight className="ml-1 size-4" />
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
