import { MonitorDown } from "lucide-react";

import { SectionCard, Switch } from "@/components";
import type { Settings } from "@/lib/tauri";

interface MinimizeToTraySectionProps {
  settings: Settings;
  onSave: (settings: Settings) => void;
}

export function MinimizeToTraySection({ settings, onSave }: MinimizeToTraySectionProps) {
  return (
    <SectionCard title="System Tray" icon={<MonitorDown className="h-5 w-5" />}>
      <div className="space-y-3">
        <label className="flex items-center justify-between gap-4">
          <div>
            <span className="block text-sm font-medium text-surface-200">
              Minimize to system tray
            </span>
            <span className="block text-sm text-surface-400">
              When enabled, clicking the minimize button will hide the application to the system
              tray instead of the taskbar. Double-click the tray icon to restore.
            </span>
          </div>
          <Switch
            checked={settings.minimizeToTray}
            onCheckedChange={(checked) => onSave({ ...settings, minimizeToTray: checked })}
          />
        </label>
        <label className="flex items-center justify-between gap-4">
          <div>
            <span className="block text-sm font-medium text-surface-200">
              Start minimized to tray
            </span>
            <span className="block text-sm text-surface-400">
              When enabled, the application will start hidden in the system tray. Double-click the
              tray icon to open.
            </span>
          </div>
          <Switch
            checked={settings.startInTray}
            onCheckedChange={(checked) => onSave({ ...settings, startInTray: checked })}
          />
        </label>
      </div>
    </SectionCard>
  );
}
