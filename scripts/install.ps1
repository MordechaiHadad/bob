$ErrorActionPreference = 'Stop'

$arch = "x86_64" 

$assetName = "bob-windows-$arch.zip"
$apiUrl = "https://api.github.com/repos/MordechaiHadad/bob/releases/latest"
$installDir = "$env:LOCALAPPDATA\bob_bin"
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

Write-Host "Extracting..."
$tempExtract = "$env:TEMP\bob_extract_tmp"
if (Test-Path $tempExtract) { Remove-Item -Recurse -Force $tempExtract }
Expand-Archive -Path $zipPath -DestinationPath $tempExtract -Force

if (Test-Path $installDir) { Remove-Item -Recurse -Force $installDir }
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$bobExe = Get-ChildItem -Path $tempExtract -Filter "bob.exe" -Recurse | Select-Object -First 1
if ($bobExe) {
    Move-Item -Path "$($bobExe.Directory.FullName)\*" -Destination $installDir -Force
} else {
    Write-Error "Error: bob.exe not found in the downloaded archive."
    exit 1
}

Remove-Item $zipPath
Remove-Item -Recurse -Force $tempExtract

Write-Host "✅ Bob installed successfully!"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "✅ Added $installDir to your User PATH."
    Write-Host "   You may need to restart your terminal for changes to take effect."
}
