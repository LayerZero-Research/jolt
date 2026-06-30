param(
    [Parameter(Position = 0, Mandatory = $true)]
    [ValidateSet("check", "table-v", "recursive", "all")]
    [string]$Command
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

$Image = if ($env:IMAGE) { $env:IMAGE } else { "jolt-artifact" }
$Results = if ($env:RESULTS) { $env:RESULTS } else { "results" }
$DoryCache = if ($env:DORY_CACHE) { $env:DORY_CACHE } else { "jolt-dory" }
$Runs = if ($env:RUNS) { $env:RUNS } else { "" }

docker info *> $null
if ($LASTEXITCODE -ne 0) {
    throw "Docker is not available to this shell. Start Docker Desktop, then retry."
}

New-Item -ItemType Directory -Force $Results | Out-Null
docker volume create $DoryCache | Out-Null
docker build -f paper_artifacts_scripts/Dockerfile -t $Image .
docker run --rm `
    -e "RUNS=$Runs" `
    -v "${PWD}\${Results}:/tmp/jolt-paper-experiments" `
    -v "${DoryCache}:/root/.cache/dory" `
    $Image $Command

if ($LASTEXITCODE -ne 0) {
    throw "Docker artifact command failed. If the error mentions Dory setup or a .urs file, run: docker volume rm $DoryCache"
}
