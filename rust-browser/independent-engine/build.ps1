$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = Join-Path (Split-Path (Split-Path $projectRoot -Parent) -Parent) 'work\independent-target'
cargo build --release --locked --target wasm32-unknown-unknown --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'Build fehlgeschlagen' }
Copy-Item (Join-Path $env:CARGO_TARGET_DIR 'wasm32-unknown-unknown\release\territory_independent.wasm') (Join-Path $projectRoot 'dist\independent\engine.wasm')
