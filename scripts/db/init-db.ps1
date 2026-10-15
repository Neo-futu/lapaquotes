#Requires -Version 5.1
<#
.SYNOPSIS
  Create PostgreSQL database `lapa` and apply project schema.
.EXAMPLE
  $env:PGPASSWORD = "secret"
  .\scripts\db\init-db.ps1
#>
param(
    [string]$PgHost = "127.0.0.1",
    [int]$Port = 5432,
    [string]$User = "postgres",
    [string]$Database = "lapa",
    [string]$PsqlPath = ""
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

function Find-Psql {
    param([string]$Hint)
    if ($Hint -and (Test-Path $Hint)) { return (Resolve-Path $Hint).Path }

    $candidates = @(
        "D:\Program Files\PostgreSQL\18\bin\psql.exe",
        "C:\Program Files\PostgreSQL\18\bin\psql.exe",
        "C:\Program Files\PostgreSQL\17\bin\psql.exe",
        "C:\Program Files\PostgreSQL\16\bin\psql.exe"
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }

    $cmd = Get-Command psql -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }

    $reg = Get-ItemProperty "HKLM:\SOFTWARE\PostgreSQL\Installations\*" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($reg -and $reg.'Base Directory') {
        $p = Join-Path $reg.'Base Directory' "bin\psql.exe"
        if (Test-Path $p) { return $p }
    }

    throw "psql.exe not found. Install PostgreSQL client tools or pass -PsqlPath."
}

$psql = Find-Psql -Hint $PsqlPath
Write-Host "Using psql: $psql"

if (-not $env:PGPASSWORD) {
    $secure = Read-Host "PostgreSQL password for user [$User]" -AsSecureString
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try {
        $env:PGPASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
    } finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) | Out-Null
    }
}

$common = @("-v", "ON_ERROR_STOP=1", "-U", $User, "-h", $PgHost, "-p", "$Port")

Write-Host ">> Create database '$Database' if missing..."
$existsRaw = & $psql @common -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = '$Database'"
if ($LASTEXITCODE -ne 0) { throw "Failed to check database existence (exit $LASTEXITCODE)" }
$exists = if ($null -eq $existsRaw) { "" } else { "$existsRaw".Trim() }
if ($exists -ne "1") {
    & $psql @common -d postgres -c "CREATE DATABASE `"$Database`" ENCODING 'UTF8';"
    if ($LASTEXITCODE -ne 0) { throw "Failed to create database (exit $LASTEXITCODE)" }
    Write-Host "Created database $Database"
} else {
    Write-Host "Database $Database already exists, skip create"
}

Write-Host ">> Apply schema.sql to '$Database'..."
& $psql @common -d $Database -f (Join-Path $Root "schema.sql")
if ($LASTEXITCODE -ne 0) { throw "Failed to apply schema (exit $LASTEXITCODE)" }

Write-Host ">> Tables:"
& $psql @common -d $Database -c "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;"
if ($LASTEXITCODE -ne 0) { throw "Failed to list tables (exit $LASTEXITCODE)" }

$pgUrl = "postgresql://${User}:$($env:PGPASSWORD)@${PgHost}:${Port}/${Database}"
$envFile = Join-Path $Root ".env"
$pgLine = "PG__URL=$pgUrl"

if (Test-Path $envFile) {
    $lines = Get-Content $envFile
    $found = $false
    $newLines = foreach ($line in $lines) {
        if ($line -match '^PG__URL=') {
            $found = $true
            $pgLine
        } else {
            $line
        }
    }
    if (-not $found) {
        $newLines += $pgLine
    }
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllLines($envFile, $newLines, $utf8NoBom)
    Write-Host "Updated PG__URL in .env"
} else {
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($envFile, ($pgLine + "`n"), $utf8NoBom)
    Write-Host "Created .env with PG__URL"
}

Write-Host ""
Write-Host "Done. Next steps:"
Write-Host "  cargo cli settings-init"
Write-Host "  cargo cli user-add <username> <password>"
Write-Host "PG__URL written to .env (do not commit)."
