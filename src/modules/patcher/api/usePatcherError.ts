import { useToast } from "@/components";
import type { AppError } from "@/lib/tauri";
import { useTauriEvent } from "@/lib/useTauriEvent";

export function usePatcherError() {
  const toast = useToast();

  useTauriEvent<AppError>("patcher-error", (payload) => {
    toast.error("Patcher Error", payload.message);
  });
}
