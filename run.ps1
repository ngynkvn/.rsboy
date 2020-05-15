param (
    [string]$log = "info"
)
Write-Host "Running gboy"

$env:RUST_LOG = $log
cargo run ./roms/Tetris.gb