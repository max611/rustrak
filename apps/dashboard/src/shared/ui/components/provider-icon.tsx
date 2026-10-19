import { Bell, Braces, Hash, Mail, Webhook } from 'lucide-react';

/**
 * The mark for an alert provider.
 *
 * `Bell` is the fallback rather than a missing glyph: the server may name a
 * provider this build has never heard of, and a row with no icon reads as a
 * broken row instead of an unfamiliar one.
 *
 * Lives in `shared/` rather than `features/alert/` because more than one
 * slice needs it: `features/monitor`'s channel-assignment dialog shows the
 * same provider marks as the alert feature's own integration pickers, and a
 * feature slice may not import another slice's `ui/components` directly.
 */
export function ProviderIcon({
  type,
  className,
}: {
  type: string;
  className?: string;
}) {
  switch (type) {
    case 'slack':
      return <Hash className={className} />;
    case 'email':
      return <Mail className={className} />;
    case 'webhook':
      return <Webhook className={className} />;
    case 'custom_webhook':
      return <Braces className={className} />;
    default:
      return <Bell className={className} />;
  }
}
