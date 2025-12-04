$ErrorActionPreference = 'Stop'

# 1. Determine Architecture
# Note: The JSON only has x86_64 for Windows currently. 
# If arm64 is added later, we can uncomment the logic.
$arch = "x86_64" 
# if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { $arch = "arm64" }

$assetName = "bob-windows-$arch.zip"
$apiUrl = "https://api.github.com/repos/MordechaiHadad/bob/releases/latest"
$installDir = "$env:LOCALAPPDATA\bob"
$zipPath = "$env:TEMP\bob.zip"

Write-Host "Fetching latest release info..."
try {
    $json = Invoke-RestMethod -Uri $apiUrl -UseBasicParsing
} catch {
    Write-Error "Failed to fetch release info from GitHub."
    exit 1
}

# Find the correct asset URL
$asset = $json.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1

if (-not $asset) {
    Write-Error "Could not find release asset: $assetName"
    exit 1
}

Write-Host "Downloading $($asset.name)..."
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath

# Install
Write-Host "Extracting to $installDir..."
if (Test-Path $installDir) { Remove-Item -Recurse -Force $installDir }
Expand-Archive -Path $zipPath -DestinationPath $installDir -Force
Remove-Item $zipPath

Write-Host "✅ Bob installed successfully!"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "✅ Added $installDir to your User PATH."
    Write-Host "   You may need to restart your terminal for changes to take effect."
}
