import { LuPalette } from "react-icons/lu";

import { SectionCard } from "@/components";
import type { Settings } from "@/lib/tauri";

import { AccentColorPicker } from "./AccentColorPicker";
import { BackdropImagePicker } from "./BackdropImagePicker";
import { ThemePicker } from "./ThemePicker";

interface AppearanceSectionProps {
  settings: Settings;
  onSave: (settings: Settings) => void;
}

export function AppearanceSection({ settings, onSave }: AppearanceSectionProps) {
  return (
    <SectionCard title="Appearance" icon={<LuPalette className="h-5 w-5" />}>
      <ThemePicker settings={settings} onSave={onSave} />
      <AccentColorPicker settings={settings} onSave={onSave} />
      <BackdropImagePicker settings={settings} onSave={onSave} />
    </SectionCard>
  );
}
