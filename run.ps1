param (
    [string]$log = "info"
)
Write-Host "Running gboy"

$env:RUST_LOG = $log
$env:RUST_BACKTRACE = 1
cargo run ./roms/Tetris.gb