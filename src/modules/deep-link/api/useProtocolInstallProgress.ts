import type { ProtocolInstallProgress } from "@/lib/tauri";
import { useTauriProgress } from "@/lib/useTauriProgress";

export function useProtocolInstallProgress() {
  return useTauriProgress<ProtocolInstallProgress>("protocol-install-progress");
}
