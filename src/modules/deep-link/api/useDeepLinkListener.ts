import { useToast } from "@/components";
import type { DeepLinkBlockedPayload, DeepLinkInstallRequest } from "@/lib/tauri";
import { useTauriEvent } from "@/lib/useTauriEvent";
import { useDeepLinkStore } from "@/stores";

export function useDeepLinkListener() {
  const toast = useToast();

  useTauriEvent<DeepLinkInstallRequest>("deep-link-install", (payload) => {
    useDeepLinkStore.getState().setRequest(payload);
  });

  useTauriEvent<DeepLinkBlockedPayload>("deep-link-blocked", (payload) => {
    toast.error(
      "Download Blocked",
      `The domain "${payload.domain}" is not in your trusted providers list. You can add it in Settings.`,
    );
  });
}
