# Package a true no-install portable QuickNotes zip from a Windows tauri build.
# Expects: src-tauri/target/release/QuickNotes.exe (after `npm run tauri build`).
# Bundles Microsoft WebView2 Fixed Version Runtime next to the exe — no MSI/setup.
# Output: QuickNotes-vX.Y.Z-portable.zip in the repo root.

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$pkg = Get-Content -Raw (Join-Path $root "package.json") | ConvertFrom-Json
$version = $pkg.version
if (-not $version) { throw "Could not read version from package.json" }

$exe = Join-Path $root "src-tauri\target\release\QuickNotes.exe"
if (-not (Test-Path $exe)) {
  throw "Missing $exe — run a Windows tauri build first."
}

$stagingName = "QuickNotes-portable"
$staging = Join-Path $root $stagingName
if (Test-Path $staging) {
  Remove-Item -Recurse -Force $staging
}
New-Item -ItemType Directory -Force -Path $staging | Out-Null

Copy-Item $exe (Join-Path $staging "QuickNotes.exe")

# Marker enables beside-exe data/ paths (see src-tauri/src/models.rs).
Set-Content -Path (Join-Path $staging "PORTABLE") -Value "QuickNotes portable mode`n" -NoNewline

$portableMd = Join-Path $root "PORTABLE.md"
if (Test-Path $portableMd) {
  Copy-Item $portableMd (Join-Path $staging "PORTABLE.md")
}

# Copy any sibling DLLs from the release dir (WebView2 loader etc. if present).
$releaseDir = Split-Path $exe
Get-ChildItem $releaseDir -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
  Copy-Item $_.FullName (Join-Path $staging $_.Name)
}

# ---------------------------------------------------------------------------
# Bundle Fixed Version WebView2 Runtime (no installer / no admin / no MSI).
# App sets WEBVIEW2_BROWSER_EXECUTABLE_FOLDER to <exe>\WebView2 at startup.
# Zip grows by ~200–400 MB; that is intentional for corporate locked-down PCs.
# ---------------------------------------------------------------------------
function Resolve-FixedRuntimeUrl {
  # Prefer Microsoft's catalog API; fall back to a pinned CDN URL if API fails.
  $fallbackVersion = "158.0.4370.45"
  $fallbackUrl = "https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/4fe269a2-de5a-488b-b6a0-9d9a42d641b8/Microsoft.WebView2.FixedVersionRuntime.158.0.4370.45.x64.cab"

  try {
    $catalog = Invoke-RestMethod -Uri "https://developer.microsoft.com/microsoft-edge/api/webview2" -UseBasicParsing
    $entry = $catalog | Select-Object -First 1
    $build = $entry.builds | Where-Object { $_.architecture -eq "x64" } | Select-Object -First 1
    if ($build -and $build.url) {
      return @{ Version = [string]$entry.version; Url = [string]$build.url }
    }
  } catch {
    Write-Warning "WebView2 catalog API failed: $_ — using pinned fallback $fallbackVersion"
  }
  return @{ Version = $fallbackVersion; Url = $fallbackUrl }
}

$runtimeInfo = Resolve-FixedRuntimeUrl
$runtimeVersion = $runtimeInfo.Version
$runtimeUrl = $runtimeInfo.Url
$cabName = "Microsoft.WebView2.FixedVersionRuntime.$runtimeVersion.x64.cab"
$cacheDir = Join-Path $root ".webview2-fixed-runtime"
New-Item -ItemType Directory -Force -Path $cacheDir | Out-Null
$cabPath = Join-Path $cacheDir $cabName

Write-Host "WebView2 Fixed Runtime version: $runtimeVersion"
Write-Host "Downloading Fixed Runtime CAB (large, ~400 MB)..."
if (-not (Test-Path $cabPath) -or ((Get-Item $cabPath).Length -lt 10MB)) {
  Invoke-WebRequest -Uri $runtimeUrl -OutFile $cabPath -UseBasicParsing
}
$cabSize = (Get-Item $cabPath).Length
if ($cabSize -lt 10MB) {
  throw "Fixed Runtime CAB looks too small ($cabSize bytes): $cabPath"
}
Write-Host "CAB ready: $cabPath ($cabSize bytes)"

# Expand with Windows expand.exe — File Explorer / Expand-Archive break the layout.
$expandRoot = Join-Path $cacheDir "expanded-$runtimeVersion"
if (Test-Path $expandRoot) {
  Remove-Item -Recurse -Force $expandRoot
}
New-Item -ItemType Directory -Force -Path $expandRoot | Out-Null
Write-Host "Expanding CAB with expand.exe -F:* ..."
& expand.exe $cabPath -F:* $expandRoot | Out-Null
if ($LASTEXITCODE -ne 0) {
  throw "expand.exe failed with exit code $LASTEXITCODE"
}

# CAB usually expands to Microsoft.WebView2.FixedVersionRuntime.<ver>.x64\...
$extractedRuntime = Get-ChildItem $expandRoot -Directory -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -like "Microsoft.WebView2.FixedVersionRuntime*" } |
  Select-Object -First 1
if (-not $extractedRuntime) {
  # Some expands put files directly under expandRoot; look for msedgewebview2.exe.
  $exeHit = Get-ChildItem $expandRoot -Recurse -Filter "msedgewebview2.exe" -ErrorAction SilentlyContinue |
    Select-Object -First 1
  if ($exeHit) {
    $extractedRuntime = $exeHit.Directory
  }
}
if (-not $extractedRuntime -or -not (Test-Path (Join-Path $extractedRuntime.FullName "msedgewebview2.exe"))) {
  throw "Could not find msedgewebview2.exe after expanding Fixed Runtime CAB"
}

# Stable folder name next to QuickNotes.exe (startup.rs looks for WebView2\).
$webviewDest = Join-Path $staging "WebView2"
Write-Host "Copying Fixed Runtime -> $webviewDest"
Copy-Item -Path $extractedRuntime.FullName -Destination $webviewDest -Recurse -Force

# Record which version shipped (for support / PORTABLE docs).
Set-Content -Path (Join-Path $staging "WebView2.VERSION") -Value "$runtimeVersion`n" -NoNewline

$wvExe = Join-Path $webviewDest "msedgewebview2.exe"
if (-not (Test-Path $wvExe)) {
  throw "WebView2 folder missing msedgewebview2.exe after copy"
}
Write-Host "Bundled Fixed Runtime $runtimeVersion at WebView2\ ($((Get-ChildItem $webviewDest -Recurse -File | Measure-Object -Property Length -Sum).Sum) bytes)"

# Seed empty data/ so the folder is obviously writable after unzip (log lands here on first run).
New-Item -ItemType Directory -Force -Path (Join-Path $staging "data") | Out-Null
Set-Content -Path (Join-Path $staging "data\.gitkeep") -Value "" -NoNewline

$zipName = "QuickNotes-v$version-portable.zip"
$zipPath = Join-Path $root $zipName
if (Test-Path $zipPath) {
  Remove-Item -Force $zipPath
}

Write-Host "Compressing portable zip (may take a few minutes)..."
Compress-Archive -Path $staging -DestinationPath $zipPath -CompressionLevel Optimal

$zipSize = (Get-Item $zipPath).Length
Write-Host "Created $zipPath ($zipSize bytes)"
Write-Host "Staging folder: $staging"
Write-Host "Layout: QuickNotes.exe + WebView2\ (Fixed Runtime) + PORTABLE + data\ — no installers."
