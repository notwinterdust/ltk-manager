# Implementation Guide: Global Hotkeys & Reload-Before-Patcher

## Context

This guide documents how the Celestial Electron app implements two features — **global hotkeys** and **reload mods before patcher** — so they can be reproduced in another Electron-based patcher app. The patterns are portable to any Electron app that manages mods and runs a patcher process.

---

## Feature 1: Global Hotkeys

System-wide keyboard shortcuts that work even when the app is not focused, using Electron's `globalShortcut` module.

### 1.1 Type Definitions

Define the hotkey settings shape in your shared types:

```ts
interface CreatorHubSettings {
  reloadModsHotkey?: string | null; // Electron accelerator string e.g. "Ctrl+Shift+R"
  killLeagueHotkey?: string | null;
  killLeagueStopsPatcher?: boolean; // default: true
}
```

**Reference:** `shared/types/types.ts:27-43`

### 1.2 Persistent Storage

Store hotkey accelerator strings in your settings store (electron-store, JSON file, etc.):

```ts
// defaults
creatorHub: {
  reloadModsHotkey: null,       // disabled by default
  killLeagueHotkey: null,
  killLeagueStopsPatcher: true,
}
```

**Reference:** `electron/store.ts:122-129`

### 1.3 IPC Handlers (Main Process)

Create three handlers:

#### `get-<feature>-hotkey` — Return current accelerator

```ts
ipcMain.handle("get-reload-mods-hotkey", () => {
  return store.get("settings.creatorHub.reloadModsHotkey") || null;
});
```

#### `set-<feature>-hotkey` — Validate, register, persist

```ts
import { globalShortcut } from "electron";

ipcMain.handle("set-reload-mods-hotkey", async (_event, accelerator: string | null) => {
  const oldHotkey = store.get("settings.creatorHub.reloadModsHotkey");

  // 1. Unregister old hotkey if it exists
  if (oldHotkey) {
    globalShortcut.unregister(oldHotkey);
  }

  // 2. If disabling (null), just save and return
  if (!accelerator) {
    store.set("settings.creatorHub.reloadModsHotkey", null);
    return { success: true };
  }

  // 3. Validate by attempting registration
  const registered = globalShortcut.register(accelerator, async () => {
    // This callback fires when the hotkey is pressed globally
    await yourReloadModsFunction();
  });

  if (!registered) {
    return {
      success: false,
      error: `Hotkey "${accelerator}" is already in use by another application`,
    };
  }

  // 4. Persist
  store.set("settings.creatorHub.reloadModsHotkey", accelerator);
  return { success: true };
});
```

**Reference:** `electron/ipc/settingsHandlers.ts:187-236`

#### Bulk settings handler — Save all settings at once

Also handle hotkey re-registration inside your bulk `set-creator-hub-settings` handler, so the Save button in the UI works correctly.

**Reference:** `electron/ipc/settingsHandlers.ts:353-473`

### 1.4 Register Hotkeys on App Startup

In your app's startup/ready handler, re-register any persisted hotkeys:

```ts
app.whenReady().then(() => {
  const creatorHub = store.get("settings.creatorHub");

  const reloadHotkey = creatorHub?.reloadModsHotkey;
  if (reloadHotkey?.trim()) {
    globalShortcut.register(reloadHotkey.trim(), async () => {
      await yourReloadModsFunction();
    });
  }

  const killHotkey = creatorHub?.killLeagueHotkey;
  if (killHotkey?.trim()) {
    globalShortcut.register(killHotkey.trim(), async () => {
      await yourKillLeagueFunction();
    });
  }
});
```

**Reference:** `electron/startup.ts:262-309`

### 1.5 Clean Up on App Quit

```ts
app.on("will-quit", () => {
  globalShortcut.unregisterAll();
});
```

### 1.6 Preload Script (Renderer Bridge)

Expose the IPC calls to the renderer via `contextBridge`:

```ts
contextBridge.exposeInMainWorld("api", {
  getReloadModsHotkey: () => ipcRenderer.invoke("get-reload-mods-hotkey"),
  setReloadModsHotkey: (hotkey: string | null) =>
    ipcRenderer.invoke("set-reload-mods-hotkey", hotkey),
  getKillLeagueHotkey: () => ipcRenderer.invoke("get-kill-league-hotkey"),
  setKillLeagueHotkey: (hotkey: string | null) =>
    ipcRenderer.invoke("set-kill-league-hotkey", hotkey),
  getCreatorHubSettings: () => ipcRenderer.invoke("get-creator-hub-settings"),
  setCreatorHubSettings: (s: any) => ipcRenderer.invoke("set-creator-hub-settings", s),
});
```

**Reference:** `electron/preload.ts:550-573`

### 1.7 Renderer UI — Hotkey Capture

The UI component uses a `<div>` with `tabIndex` and `onKeyDown` to capture key combos:

```tsx
const [isCapturing, setIsCapturing] = useState(false);

const handleKeyDown = (e: React.KeyboardEvent) => {
  if (!isCapturing) return;
  e.preventDefault();

  const keys: string[] = [];
  if (e.ctrlKey) keys.push("Ctrl");
  if (e.altKey) keys.push("Alt");
  if (e.shiftKey) keys.push("Shift");
  if (e.metaKey) keys.push("Meta");

  const mainKey = e.key;
  if (!["Control", "Alt", "Shift", "Meta"].includes(mainKey)) {
    let keyName = mainKey.length === 1 ? mainKey.toUpperCase() : mainKey;
    keys.push(keyName);

    const accelerator = keys.join("+"); // e.g. "Ctrl+Shift+R"
    setIsCapturing(false);

    // Send to main process for validation + registration
    const result = await window.api.setReloadModsHotkey(accelerator);
    if (result.success) {
      setHotkey(accelerator);
      toast.success(`Hotkey set to: ${accelerator}`);
    } else {
      toast.error(result.error);
    }
  }
};
```

**Reference:** `src/components/dialogs/CreatorHubSettingsDialog.tsx:101-157`

### 1.8 Hotkey Action: Reload All Mods

What happens when the reload hotkey fires:

```ts
async function reloadAllCreatorHubMods() {
  // 1. Stop the patcher
  await patcherService.stop();

  // 2. Kill League of Legends
  spawn("taskkill", ["/F", "/IM", "League of Legends.exe"]);

  // 3. Re-import each enabled Creator Hub mod from its source folder
  const mods = modManager
    .getAllMods()
    .filter((m) => m.byCreatorHub && m.isEnabled && m.type === "imported");
  for (const mod of mods) {
    await modService.reloadMod(mod.id); // re-reads source folder, re-packages
  }

  // 4. Save state & clear cache
  await modManager.forceSave();
  store.delete("activeModsInfo");

  // 5. Run patcher with fresh mods
  await patcherService.createCombinedOverlay();

  // 6. Reconnect to League client (optional)
  await lcuApi.triggerReconnect();
}
```

**Reference:** `electron/ipc/skinHandlers.ts:958-1050`

### 1.9 Hotkey Action: Kill League

```ts
async function killLeague() {
  const shouldStopPatcher = store.get("settings.creatorHub.killLeagueStopsPatcher") ?? true;

  if (shouldStopPatcher) {
    await patcherService.stop();
  }

  const proc = spawn("taskkill", ["/F", "/IM", "League of Legends.exe"]);
  // Wait with 3-second timeout
  await new Promise((resolve) => {
    const timeout = setTimeout(resolve, 3000);
    proc.on("close", () => {
      clearTimeout(timeout);
      resolve(undefined);
    });
  });
}
```

**Reference:** `electron/ipc/serviceHandlers.ts:87-138`

---

## Feature 2: Reload Mods Before Patcher

A boolean toggle that, when enabled, automatically reloads all Creator Hub mods right before the patcher runs.

### 2.1 The Setting

- **Store key:** `settings.creatorHub.reloadModsBeforePatcher`
- **Default:** `true`
- **UI:** Checkbox in Settings → Preferences tab

### 2.2 Implementation

Insert a check at the top of your "run patcher" handler, **before** the actual patching begins:

```ts
ipcMain.handle("run-active-skins", async () => {
  const creatorHub = store.get("settings.creatorHub");

  // --- Reload-before-patcher gate ---
  if (creatorHub?.reloadModsBeforePatcher === true) {
    const creatorHubMods = modManager
      .getAllMods()
      .filter((m) => m.byCreatorHub === true && m.isEnabled === true && m.type === "imported");

    for (const mod of creatorHubMods) {
      await rawModService.reloadMod(mod.id);
    }

    if (creatorHubMods.length > 0) {
      await modManager.forceSave();
      await new Promise((r) => setTimeout(r, 500)); // small delay for FS sync
      store.delete("activeModsInfo"); // bust cache
    }
  }
  // --- End gate ---

  // Proceed with normal patching...
  await patcherService.createCombinedOverlay();
});
```

**Reference:** `electron/ipc/skinHandlers.ts:833-870`

### 2.3 What `reloadMod` Does Under the Hood

```ts
async reloadMod(modId: string) {
  const mod = await modManager.getMod(modId)

  // Must be an imported mod with a known source path
  if (!mod || mod.type !== 'imported' || !mod.sourcePath) {
    return { success: false, error: 'Not a reloadable mod' }
  }

  // Verify source folder still exists
  await fs.access(mod.sourcePath)

  // Re-import from source (overwrite mode)
  return await this.importRawModFolder(
    mod.sourcePath,  // original folder on disk
    mod.name,
    mod.champion,
    true             // overwrite = true
  )
}
```

**Reference:** `electron/services/rawModService.ts:1050-1091`

---

## Architecture Summary

```
Renderer (React)
  ├── Settings UI captures accelerator strings
  ├── Sends via window.api.setReloadModsHotkey(accelerator)
  └── Sends via window.api.setCreatorHubSettings({...})

Preload (contextBridge)
  └── Maps window.api methods to ipcRenderer.invoke()

Main Process
  ├── settingsHandlers.ts — validate & register hotkeys, persist settings
  ├── startup.ts — re-register hotkeys from store on app launch
  ├── skinHandlers.ts — reload-before-patcher gate + reload-all-mods action
  ├── serviceHandlers.ts — kill league action
  └── store.ts — encrypted persistent storage
```

---

## Key Files to Reference in Celestial Electron

| File                                                          | What to Copy                       |
| ------------------------------------------------------------- | ---------------------------------- |
| `shared/types/types.ts:27-43`                                 | Setting type definitions           |
| `electron/store.ts:122-129`                                   | Store defaults                     |
| `electron/preload.ts:550-573`                                 | IPC bridge                         |
| `electron/ipc/settingsHandlers.ts:175-473`                    | Get/set hotkey + settings handlers |
| `electron/startup.ts:262-309`                                 | Hotkey registration on app start   |
| `electron/ipc/skinHandlers.ts:833-870`                        | Reload-before-patcher check        |
| `electron/ipc/skinHandlers.ts:958-1050`                       | Reload all mods handler            |
| `electron/ipc/serviceHandlers.ts:87-138`                      | Kill league handler                |
| `electron/services/rawModService.ts:1050-1091`                | Single mod reload logic            |
| `src/components/dialogs/CreatorHubSettingsDialog.tsx:101-157` | Hotkey capture UI                  |

---

## Verification

1. **Hotkey registration:** Set a hotkey in settings, close the app, reopen — hotkey should still be registered
2. **Hotkey conflict:** Try registering a hotkey already used by another app — should return an error
3. **Hotkey disable:** Click "Disable" — hotkey should be unregistered and setting set to null
4. **Reload before patcher:** Enable the toggle, modify a mod's source files, run patcher — mod should pick up changes
5. **Kill League:** Set kill hotkey, launch League, press hotkey — League process should terminate
6. **Kill + stop patcher:** With "Stop Patcher" checked, pressing kill hotkey should also stop the patcher
