# QuickNotes portable (no installer)

Corporate / locked-down PCs: unzip anywhere (USB stick, user folder, network share copy) and run. No MSI, no Program Files, no admin rights required for the app itself.

## Quick start

1. Download `QuickNotes-vX.Y.Z-portable.zip` from [Releases](https://github.com/Homavi88/QuickNotes/releases).
2. Unzip to any writable folder (example: `%USERPROFILE%\Apps\QuickNotes` or a USB path).
3. Run `QuickNotes.exe`.
4. The app lives in the system tray. Global hotkey (default `Ctrl+Alt+N`) opens quick note.
5. Optional: open **Settings → Автозапуск с Windows** to start with Windows.

Windows may show SmartScreen for unsigned builds: **More info → Run anyway**.

## Requirements

- Windows 10/11
- [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (usually already installed with Edge on corporate images)

## Data locations (portable package)

The zip includes a `PORTABLE` marker file next to `QuickNotes.exe`. While that marker is present:

| What | Path |
|------|------|
| Config | `<folder>\data\config.json` |
| Notes (default) | `<folder>\data\notes\<Project>\notes.md` |

Everything stays under the unzipped folder (plus WebView2/OS caches). Safe for USB and user-writable directories; nothing is written under Program Files.

You can still change the notes folder in Settings to e.g. `%LOCALAPPDATA%\QuickNotes` if you prefer profile-backed storage while keeping the portable exe.

### Installed (MSI / NSIS) layout

Without the `PORTABLE` marker (normal installer builds):

| What | Path |
|------|------|
| Config | `%APPDATA%\QuickNotes\config.json` |
| Notes (default) | `%USERPROFILE%\QuickNotes\` |

## Autostart without an installer

Settings → **Автозапуск с Windows** uses `tauri-plugin-autostart`, which registers the **absolute path of the running `QuickNotes.exe`** in the current-user Run key (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). No MSI or admin elevation is required.

If you move or rename the portable folder, open Settings and toggle autostart off/on so the path is updated.

## Folder layout in the zip

```
QuickNotes-portable/
  QuickNotes.exe   # run this
  PORTABLE         # marker → use data/ next to the exe
  PORTABLE.md      # this file
  data/            # created on first run (config + notes)
```

## Building the zip locally (Windows)

```bash
npm ci
npm run tauri build -- --bundles msi,nsis
pwsh -File scripts/package-portable.ps1
```

Output: `QuickNotes-vX.Y.Z-portable.zip` in the repo root.
