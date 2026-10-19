import type { UptimeCheckType, UptimeMonitor } from '@rustrak/client';
import { useState } from 'react';
import { useTranslations } from 'use-intl';
import { Button } from '@/shared/ui/components/shadcn/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/components/shadcn/dialog';
import { Input } from '@/shared/ui/components/shadcn/input';
import { Label } from '@/shared/ui/components/shadcn/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/components/shadcn/select';
import { Switch } from '@/shared/ui/components/shadcn/switch';

export interface MonitorFormValues {
  name: string;
  check_type: UptimeCheckType;
  url: string;
  interval_secs: string;
  timeout_secs: string;
  expected_status: string;
  fail_threshold: string;
  recovery_threshold: string;
  repeat_interval_secs: string;
  enabled: boolean;
}

const DEFAULTS: MonitorFormValues = {
  name: '',
  check_type: 'http',
  url: '',
  interval_secs: '60',
  timeout_secs: '10',
  expected_status: '',
  fail_threshold: '2',
  recovery_threshold: '2',
  repeat_interval_secs: '3600',
  enabled: true,
};

function fromMonitor(monitor: UptimeMonitor): MonitorFormValues {
  return {
    name: monitor.name,
    check_type: monitor.check_type as UptimeCheckType,
    url: monitor.url,
    interval_secs: String(monitor.interval_secs),
    timeout_secs: String(monitor.timeout_secs),
    expected_status:
      monitor.expected_status === null ? '' : String(monitor.expected_status),
    fail_threshold: String(monitor.fail_threshold),
    recovery_threshold: String(monitor.recovery_threshold),
    repeat_interval_secs: String(monitor.repeat_interval_secs),
    enabled: monitor.enabled,
  };
}

/** Parses the form's string fields into the shape the API expects. */
function parseInt10(value: string): number | undefined {
  if (value.trim() === '') return undefined;
  const n = Number.parseInt(value, 10);
  return Number.isFinite(n) ? n : undefined;
}

export interface MonitorSubmitValues {
  name: string;
  check_type: UptimeCheckType;
  url: string;
  interval_secs?: number;
  timeout_secs?: number;
  expected_status?: number;
  fail_threshold?: number;
  recovery_threshold?: number;
  repeat_interval_secs?: number;
  enabled?: boolean;
}

export function parseMonitorForm(values: MonitorFormValues): MonitorSubmitValues {
  return {
    name: values.name.trim(),
    check_type: values.check_type,
    url: values.url.trim(),
    interval_secs: parseInt10(values.interval_secs),
    timeout_secs: parseInt10(values.timeout_secs),
    expected_status: parseInt10(values.expected_status),
    fail_threshold: parseInt10(values.fail_threshold),
    recovery_threshold: parseInt10(values.recovery_threshold),
    repeat_interval_secs: parseInt10(values.repeat_interval_secs),
    enabled: values.enabled,
  };
}

/**
 * One dialog for create and edit, same shape `AlertRuleFormDialog` uses for
 * its own two modes — a `key` on the mount point resets the form when the
 * target monitor changes.
 */
export function MonitorFormDialog({
  open,
  onOpenChange,
  existingMonitor,
  isPending,
  error,
  onSubmit,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  existingMonitor: UptimeMonitor | null;
  isPending: boolean;
  error: string | null;
  onSubmit: (values: MonitorSubmitValues) => void;
}) {
  const t = useTranslations('uptimeMonitors');
  const [values, setValues] = useState<MonitorFormValues>(
    existingMonitor ? fromMonitor(existingMonitor) : DEFAULTS,
  );

  const isEdit = existingMonitor !== null;
  const isValid = values.name.trim() !== '' && values.url.trim() !== '';

  const set = <K extends keyof MonitorFormValues>(
    key: K,
    value: MonitorFormValues[K],
  ) => setValues((prev) => ({ ...prev, [key]: value }));

  const submit = () => {
    if (!isValid) return;
    onSubmit(parseMonitorForm(values));
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isEdit ? t('form.editTitle') : t('form.createTitle')}
          </DialogTitle>
          <DialogDescription>{t('form.description')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-2">
            <Label htmlFor="monitor-name">{t('form.name')}</Label>
            <Input
              id="monitor-name"
              value={values.name}
              onChange={(e) => set('name', e.target.value)}
              placeholder={t('form.namePlaceholder')}
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-2">
              <Label htmlFor="monitor-type">{t('form.checkType')}</Label>
              <Select
                value={values.check_type}
                onValueChange={(v) => set('check_type', v as UptimeCheckType)}
                disabled={isEdit}
              >
                <SelectTrigger id="monitor-type" className="w-full">
                  <SelectValue>
                    {() => t(`form.checkType${values.check_type === 'http' ? 'Http' : 'Tcp'}`)}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="http">{t('form.checkTypeHttp')}</SelectItem>
                  <SelectItem value="tcp">{t('form.checkTypeTcp')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="monitor-url">{t('form.url')}</Label>
              <Input
                id="monitor-url"
                value={values.url}
                onChange={(e) => set('url', e.target.value)}
                placeholder={
                  values.check_type === 'http'
                    ? 'https://example.com/health'
                    : 'example.com:443'
                }
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-2">
              <Label htmlFor="monitor-interval">{t('form.intervalSecs')}</Label>
              <Input
                id="monitor-interval"
                inputMode="numeric"
                value={values.interval_secs}
                onChange={(e) => set('interval_secs', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="monitor-timeout">{t('form.timeoutSecs')}</Label>
              <Input
                id="monitor-timeout"
                inputMode="numeric"
                value={values.timeout_secs}
                onChange={(e) => set('timeout_secs', e.target.value)}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-2">
              <Label htmlFor="monitor-fail-threshold">
                {t('form.failThreshold')}
              </Label>
              <Input
                id="monitor-fail-threshold"
                inputMode="numeric"
                value={values.fail_threshold}
                onChange={(e) => set('fail_threshold', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="monitor-recovery-threshold">
                {t('form.recoveryThreshold')}
              </Label>
              <Input
                id="monitor-recovery-threshold"
                inputMode="numeric"
                value={values.recovery_threshold}
                onChange={(e) => set('recovery_threshold', e.target.value)}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            {values.check_type === 'http' && (
              <div className="space-y-2">
                <Label htmlFor="monitor-expected-status">
                  {t('form.expectedStatus')}
                </Label>
                <Input
                  id="monitor-expected-status"
                  inputMode="numeric"
                  placeholder={t('form.expectedStatusPlaceholder')}
                  value={values.expected_status}
                  onChange={(e) => set('expected_status', e.target.value)}
                />
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="monitor-repeat-interval">
                {t('form.repeatIntervalSecs')}
              </Label>
              <Input
                id="monitor-repeat-interval"
                inputMode="numeric"
                value={values.repeat_interval_secs}
                onChange={(e) => set('repeat_interval_secs', e.target.value)}
              />
            </div>
          </div>

          {isEdit && (
            <div className="flex items-center justify-between rounded-lg border p-3">
              <div className="space-y-0.5">
                <Label className="text-sm font-medium">
                  {t('form.enabled')}
                </Label>
                <p className="text-xs text-muted-foreground">
                  {t('form.enabledDescription')}
                </p>
              </div>
              <Switch
                checked={values.enabled}
                onCheckedChange={(v) => set('enabled', v)}
              />
            </div>
          )}

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isPending}
          >
            {t('form.cancel')}
          </Button>
          <Button onClick={submit} disabled={isPending || !isValid}>
            {isPending
              ? t('form.saving')
              : isEdit
                ? t('form.save')
                : t('form.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
