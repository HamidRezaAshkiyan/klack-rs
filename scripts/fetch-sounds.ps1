#Requires -Version 7.0
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

# Downloads all 55 MIT-licensed multi-sampled MP3 audio sprites from
# https://github.com/joshwcomeau/button-sounds into src-tauri/resources/sounds/

$RootDir = Split-Path -Parent $PSScriptRoot
$OutDir = Join-Path $RootDir 'src-tauri' 'resources' 'sounds'
$BaseUrl = 'https://raw.githubusercontent.com/joshwcomeau/button-sounds/main/package/assets'

$Sounds = @(
    'asus-kbd-hard', 'asus-kbd-medium', 'asus-kbd-soft', 'asus-kbd-space', 'asus-mouse',
    'camera-latch', 'carson-microflip', 'ergo-mx', 'guitar-toggle-off', 'guitar-toggle-on',
    'halo-hard', 'halo-shift', 'halo-soft', 'halo-space', 'heated-blanket',
    'hi75-hard', 'hi75-knob', 'hi75-medium', 'hi75-soft', 'hi75-space',
    'iidx', 'kvm-switch', 'led-blacklight', 'lock-closed', 'lock-open',
    'low-thumps', 'mx-master-3', 'popper-dice', 'popper-empty',
    'ps5-clicky', 'ps5-double', 'ps5-dull', 'ps5-paddle', 'ps5-snappy',
    'raddy-high', 'raddy-low', 'raddy-medium', 'rival-3', 'rival-3-middle',
    'sd-clicky', 'sd-distorted', 'sd-metallic', 'sd-resonant', 'sd-squelchy', 'sd-standard',
    'synth-off', 'synth-on', 'tonal-off', 'tonal-on',
    'treadmill-flinty', 'treadmill-pitched', 'treadmill-sharp',
    'uhk-medium', 'uhk-soft', 'uhk-space'
)

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "Fetching $($Sounds.Count) audio sprites from joshwcomeau/button-sounds -> $OutDir"

$Sounds | ForEach-Object -Parallel {
    $name = $_
    $dest = Join-Path $using:OutDir "$name.mp3"
    if ((Test-Path $dest) -and ((Get-Item $dest).Length -gt 0)) {
        return
    }
    Invoke-WebRequest -Uri "$using:BaseUrl/$name.mp3" -OutFile $dest
} -ThrottleLimit 8

$ready = (Get-ChildItem -Path $OutDir -Filter '*.mp3' | Where-Object { $_.Length -gt 0 }).Count
Write-Host "Done: $ready/$($Sounds.Count) MP3 sprites ready in $OutDir"
if ($ready -ne $Sounds.Count) {
    throw "Expected $($Sounds.Count) files, found $ready"
}
