import { LuDownload } from "react-icons/lu";

import { Button, SectionCard } from "@/components";

interface MigrationSectionProps {
  onImport: () => void;
}

export function MigrationSection({ onImport }: MigrationSectionProps) {
  return (
    <SectionCard title="Import from cslol-manager">
      <div className="space-y-3">
        <p className="text-sm text-surface-400">
          If you previously used cslol-manager, you can import your installed mods into LTK Manager.
        </p>
        <Button variant="outline" size="sm" onClick={onImport}>
          <span className="flex items-center gap-2">
            <LuDownload className="h-4 w-4" />
            Import Mods...
          </span>
        </Button>
      </div>
    </SectionCard>
  );
}
