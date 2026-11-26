# Package a portable QuickNotes folder + zip from a Windows tauri build.
# Expects: src-tauri/target/release/QuickNotes.exe (after `npm run tauri build`).
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

# Evergreen WebView2 bootstrapper — portable builds cannot use MSI's downloadBootstrapper.
# App detects missing Runtime and can launch this installer (or the user runs it manually).
$bootstrapperName = "MicrosoftEdgeWebview2Setup.exe"
$bootstrapperDest = Join-Path $staging $bootstrapperName
$bootstrapperUrl = "https://go.microsoft.com/fwlink/p/?LinkId=2124703"
Write-Host "Downloading WebView2 Evergreen Bootstrapper..."
try {
  Invoke-WebRequest -Uri $bootstrapperUrl -OutFile $bootstrapperDest -UseBasicParsing
  if (-not (Test-Path $bootstrapperDest) -or ((Get-Item $bootstrapperDest).Length -lt 1000)) {
    throw "Bootstrapper download looked empty"
  }
  Write-Host "Bundled $bootstrapperName ($((Get-Item $bootstrapperDest).Length) bytes)"
} catch {
  Write-Warning "Could not download WebView2 bootstrapper: $_"
  Write-Warning "Portable zip will ship without it; app still detects Runtime and shows a MessageBox with the download link."
}

# Seed empty data/ so the folder is obviously writable after unzip (log lands here on first run).
New-Item -ItemType Directory -Force -Path (Join-Path $staging "data") | Out-Null
Set-Content -Path (Join-Path $staging "data\.gitkeep") -Value "" -NoNewline

$zipName = "QuickNotes-v$version-portable.zip"
$zipPath = Join-Path $root $zipName
if (Test-Path $zipPath) {
  Remove-Item -Force $zipPath
}

Compress-Archive -Path $staging -DestinationPath $zipPath -CompressionLevel Optimal

Write-Host "Created $zipPath"
Write-Host "Staging folder: $staging"
