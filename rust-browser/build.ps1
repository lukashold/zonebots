$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
  $env:CARGO_TARGET_DIR = Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) 'work\rust-target'
  cargo build --release --target wasm32-unknown-unknown --lib --locked
  if ($LASTEXITCODE -ne 0) { throw 'Rust/Wasm build failed' }
  Copy-Item (Join-Path $env:CARGO_TARGET_DIR 'wasm32-unknown-unknown\release\zonebots_rust.wasm') 'dist\engine.wasm'
  cargo build --release --bin z-sim --locked
  if ($LASTEXITCODE -ne 0) { throw 'Native Rust build failed' }
} finally { Pop-Location }
