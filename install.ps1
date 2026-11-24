[CmdletBinding()]
param(
    [string]$Version = $env:AI_SESSION_VERSION,
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'ai-session-export'),
    [string]$ReleaseDir = $env:AI_SESSION_RELEASE_DIR,
    [switch]$NoPath
)
$ErrorActionPreference = 'Stop'
$repo = 'https://github.com/lilyco-42/ai-session-export'
try { $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() }
catch { $arch = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE } }
switch ($arch.ToLowerInvariant()) {
    { $_ -in 'x64', 'amd64' } { $target = 'x86_64-pc-windows-msvc'; break }
    'arm64' { $target = 'aarch64-pc-windows-msvc'; break }
    default { throw "Unsupported Windows architecture: $arch" }
}
$name = "ai-session-export-$target"
$asset = "$name.zip"
$InstallRoot = [IO.Path]::GetFullPath($InstallRoot)
$app = Join-Path $InstallRoot 'app'
$exe = Join-Path $app 'ai-session-export-server.exe'
New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null
if (Test-Path -LiteralPath $app) {
    if ((Get-Item -LiteralPath $app).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Refusing to replace a reparse point: $app" }
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw "Unrecognized install directory: $app" }
}
$tmp = Join-Path $InstallRoot ('.install-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null
$backup = Join-Path $tmp 'previous'
try {
    if ($ReleaseDir) {
        Copy-Item -LiteralPath (Join-Path $ReleaseDir $asset) -Destination $tmp
        Copy-Item -LiteralPath (Join-Path $ReleaseDir 'SHA256SUMS') -Destination $tmp
    } else {
        if (-not $Version) {
            $release = Invoke-RestMethod -Uri 'https://api.github.com/repos/lilyco-42/ai-session-export/releases/latest' -Headers @{ 'User-Agent' = 'ai-session-export-installer' }
            $Version = $release.tag_name
        }
        if ($Version -notmatch '^v[0-9][A-Za-z0-9._-]*$') { throw "Invalid release tag: $Version" }
        foreach ($file in @($asset, 'SHA256SUMS')) {
            Invoke-WebRequest -UseBasicParsing -Uri "$repo/releases/download/$Version/$file" -OutFile (Join-Path $tmp $file)
        }
    }
    $pattern = '^([0-9a-fA-F]{64})\s+\*?' + [regex]::Escape($asset) + '$'
    $hashes = @(Get-Content -LiteralPath (Join-Path $tmp 'SHA256SUMS') | ForEach-Object { if ($_ -match $pattern) { $Matches[1] } })
    if ($hashes.Count -ne 1) { throw "Missing or invalid checksum for $asset" }
    $zip = Join-Path $tmp $asset
    if ((Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash -ne $hashes[0]) { throw 'SHA256 mismatch' }
    Expand-Archive -LiteralPath $zip -DestinationPath $tmp
    $stage = Join-Path $tmp $name
    if (-not (Test-Path -LiteralPath (Join-Path $stage 'ai-session-export-server.exe') -PathType Leaf) -or
        -not (Test-Path -LiteralPath (Join-Path $stage 'web/dist/index.html') -PathType Leaf)) { throw 'Incomplete release archive' }
    if (Test-Path -LiteralPath $app) { Move-Item -LiteralPath $app -Destination $backup }
    try { Move-Item -LiteralPath $stage -Destination $app }
    catch {
        if (Test-Path -LiteralPath $backup) { Move-Item -LiteralPath $backup -Destination $app }
        throw
    }
    if (-not $NoPath) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (($userPath -split ';') -notcontains $app) {
            $newPath = if ($userPath) { $userPath.TrimEnd(';') + ';' + $app } else { $app }
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        }
        if (($env:Path -split ';') -notcontains $app) { $env:Path += ";$app" }
    }
    Write-Host "Installed: $exe"
    Write-Host 'Run ai-session-export-server and open http://127.0.0.1:8788'
} finally {
    if ((Test-Path -LiteralPath $backup) -and -not (Test-Path -LiteralPath $app)) {
        Write-Warning "Previous installation preserved at $backup"
    } else { Remove-Item -LiteralPath $tmp -Recurse -Force }
}
