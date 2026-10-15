# Start site with cargo-leptos (fixes missing cargo PATH in conda/Cursor shells).
$ErrorActionPreference = "Stop"
$CargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path (Join-Path $CargoBin "cargo.exe")) {
    if ($env:Path -notlike "*$CargoBin*") {
        $env:Path = "$CargoBin;" + $env:Path
    }
} else {
    throw "cargo.exe not found at $CargoBin. Install Rust from https://rustup.rs"
}

Set-Location (Resolve-Path (Join-Path $PSScriptRoot ".."))
if (-not $env:RUST_LOG) { $env:RUST_LOG = "info,site=debug" }
cargo leptos watch -p site
