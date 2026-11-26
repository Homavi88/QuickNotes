# QuickNotes portable (no installer)

Corporate / locked-down PCs: unzip anywhere (USB stick, user folder) and run **`QuickNotes.exe` only**. No MSI, no NSIS setup, no `MicrosoftEdgeWebview2Setup.exe`, no admin rights, no Program Files.

This package **bundles the Microsoft WebView2 Fixed Version Runtime** next to the exe, so the UI works without installing WebView2 Runtime on the machine.

## Quick start

1. Download `QuickNotes-vX.Y.Z-portable.zip` from [Releases](https://github.com/Homavi88/QuickNotes/releases).
2. Unzip to a **writable local folder** (example: `%USERPROFILE%\Apps\QuickNotes`).  
   Avoid read-only network shares and **UNC paths** — Fixed Version WebView2 cannot run from a network location.
3. Run `QuickNotes.exe`. Nothing else to install.
4. The app lives in the **system tray** (notification area). Global hotkey (default `Ctrl+Alt+N`) opens quick note. Use the tray menu for Notes / Settings.
5. Optional: open **Settings → Автозапуск с Windows** to start with Windows.

Windows may show SmartScreen for unsigned builds: **More info → Run anyway**. Antivirus may also warn on an unsigned exe — that is not an installer prompt.

## Requirements

- Windows 10/11 (x64)
- **No separate WebView2 install** — the zip includes `WebView2\` (Fixed Version Runtime)
- Writable local disk folder (not UNC)

## If nothing appears (tray / windows)

1. Confirm `WebView2\msedgewebview2.exe` exists next to `QuickNotes.exe`.
2. Open the log: `<unzipped folder>\data\quicknotes.log` (created on launch).
3. Confirm `PORTABLE` is next to `QuickNotes.exe` (enables `data/` paths).
4. Check the tray overflow chevron (`^`) — icons are often hidden on Windows 11.
5. SmartScreen / antivirus may have blocked the first run — allow the exe and retry.
6. On Windows 10, first launch may adjust ACLs on `WebView2\` (required for Fixed Runtime ≥120). If the log shows `icacls` failures and windows fail, copy the folder to a user-writable path and relaunch.

## Data locations (portable package)

The zip includes a `PORTABLE` marker file next to `QuickNotes.exe`. While that marker is present:

| What | Path |
|------|------|
| Config | `<folder>\data\config.json` |
| Notes (default) | `<folder>\data\notes\<Project>\notes.md` |
| Startup log | `<folder>\data\quicknotes.log` |
| WebView2 profile/cache | `<folder>\data\webview2\` |

Everything stays under the unzipped folder. Safe for USB and user-writable directories; nothing is written under Program Files.

You can still change the notes folder in Settings to e.g. `%LOCALAPPDATA%\QuickNotes` if you prefer profile-backed storage while keeping the portable exe.

### Installed (MSI / NSIS) layout

Without the `PORTABLE` marker (normal installer builds):

| What | Path |
|------|------|
| Config | `%APPDATA%\QuickNotes\config.json` |
| Notes (default) | `%USERPROFILE%\QuickNotes\` |
| Startup log | `%APPDATA%\QuickNotes\quicknotes.log` (or beside exe) |

Installer builds use the system Evergreen WebView2 Runtime (bootstrapper at install time). The **portable** zip does not.

## Autostart without an installer

Settings → **Автозапуск с Windows** uses `tauri-plugin-autostart`, which registers the **absolute path of the running `QuickNotes.exe`** in the current-user Run key (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). No MSI or admin elevation is required.

If you move or rename the portable folder, open Settings and toggle autostart off/on so the path is updated.

## Folder layout in the zip

```
QuickNotes-portable/
  QuickNotes.exe          # run this — only step needed
  PORTABLE                # marker → use data/ next to the exe
  PORTABLE.md             # this file
  WebView2.VERSION        # Fixed Runtime version string
  WebView2/               # Microsoft Fixed Version Runtime (local, no install)
    msedgewebview2.exe
    ...                   # remaining Fixed Runtime binaries (~hundreds of MB)
  data/                   # config + notes + quicknotes.log + webview2 profile
```

Zip size is large (~200–400+ MB) because the Fixed Runtime is included on purpose.

## Building the zip locally (Windows)

```bash
npm ci
npm run tauri build -- --bundles msi,nsis
pwsh -File scripts/package-portable.ps1
```

The packaging script downloads the Fixed Version Runtime CAB from Microsoft, expands it with `expand.exe`, and copies it to `WebView2\` inside the staging folder.

Output: `QuickNotes-vX.Y.Z-portable.zip` in the repo root.
