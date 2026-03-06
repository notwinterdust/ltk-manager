# Protocol Schema Contract: `ltk://` URI

## Protocol Format

```
ltk://install?url=<download_url>&name=<mod_name>&author=<author_name>&source=<site_name>
```

**Scheme**: `ltk`
**Action**: `install` (only supported action in v1)

## Parameters

| Parameter  | Required | Type   | Description                                                                              |
| ---------- | -------- | ------ | ---------------------------------------------------------------------------------------- |
| `url`      | Yes      | URL    | HTTPS download URL for a `.modpkg` or `.fantome` file                                    |
| `name`     | No       | String | URL-encoded display name for the mod (max 256 chars)                                     |
| `author`   | No       | String | URL-encoded author/creator name (max 256 chars)                                          |
| `source`   | No       | String | URL-encoded name of the originating site (max 256 chars)                                 |
| `checksum` | Reserved | String | Accepted but ignored. Format: `sha256:<hex>`. Reserved for future checksum verification. |
| `api`      | Reserved | String | Accepted but ignored. Reserved for future API endpoint support.                          |

## Validation Rules

1. `url` parameter MUST be present
2. `url` MUST use `https` scheme (`http` is rejected)
3. `url` MUST be a well-formed URL parseable by standard URL parsers
4. `url` host MUST NOT resolve to loopback or private IP ranges
5. `name` and `source` are display-only; no validation beyond length limit and URL decoding
6. Unknown parameters MUST be silently ignored (forward compatibility)

## Examples

### Minimal (URL only)

```
ltk://install?url=https://cdn.runeforge.dev/mods/dark-star-aatrox.modpkg
```

### With display metadata

```
ltk://install?url=https://cdn.runeforge.dev/mods/dark-star-aatrox.modpkg&name=Dark%20Star%20Aatrox&author=SkinCreator42&source=Runeforge
```

### With reserved parameters (accepted, ignored)

```
ltk://install?url=https://cdn.runeforge.dev/mods/dark-star-aatrox.modpkg&name=Dark%20Star%20Aatrox&checksum=sha256:9f86d081884c7d659a2feaa0c55ad015&api=https://api.runeforge.dev/v1/mods/dark-star-aatrox
```

## Website Integration

### Simple HTML link

```html
<a
  href="ltk://install?url=https://example.com/mods/cool-skin.modpkg&name=Cool%20Skin&author=SkinMaker&source=MySite"
>
  Install in LTK Manager
</a>
```

### With fallback (app not installed)

```html
<a href="ltk://install?url=..." id="install-btn">Install in LTK Manager</a>
<script>
  document.getElementById("install-btn").addEventListener("click", (e) => {
    setTimeout(() => {
      window.location.href = "https://leaguetoolkit.dev/download";
    }, 2000);
  });
</script>
```

## Tauri IPC Commands

### `deep_link_install_mod`

Triggered when user confirms the install in the confirmation dialog.

**Input**:

```typescript
{
  url: string;      // HTTPS download URL
  name?: string;    // Display name (from protocol link)
  author?: string;  // Author/creator name (from protocol link)
  source?: string;  // Source site name (from protocol link)
}
```

**Output** (success):

```typescript
{
  ok: true;
  value: InstalledMod; // Same type as existing mod install
}
```

**Output** (error):

```typescript
{
  ok: false;
  error: {
    code: "INVALID_PATH" | "IO" | "MODPKG" | "VALIDATION_FAILED";
    message: string;
    context?: object;
  }
}
```

### Frontend Event: `deep-link-install`

Emitted from backend to frontend when a valid `ltk://install` URL is received.

**Payload**:

```typescript
{
  url: string;
  name: string | null;
  author: string | null;
  source: string | null;
}
```

### Frontend Event: `protocol-install-progress`

Emitted during download/install phases.

**Payload**:

```typescript
{
  stage: "downloading" | "validating" | "installing" | "complete" | "error";
  bytes_downloaded: number;
  total_bytes: number | null;
  error: string | null;
}
```
