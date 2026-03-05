import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export function useTauriEvent<T>(eventName: string | null, callback: (payload: T) => void): void {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    if (!eventName) return;

    let mounted = true;
    const promise = listen<T>(eventName, (event) => {
      callbackRef.current(event.payload);
    });

    promise.then((unlisten) => {
      if (!mounted) unlisten();
    });

    return () => {
      mounted = false;
      promise.then((unlisten) => unlisten());
    };
  }, [eventName]);
}
