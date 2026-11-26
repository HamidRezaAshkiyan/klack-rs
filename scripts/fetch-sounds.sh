#!/usr/bin/env bash
set -euo pipefail

# Downloads all 55 MIT-licensed multi-sampled MP3 audio sprites from
# https://github.com/joshwcomeau/button-sounds into src-tauri/resources/sounds/

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${ROOT_DIR}/src-tauri/resources/sounds"
BASE_URL="https://raw.githubusercontent.com/joshwcomeau/button-sounds/main/package/assets"

SOUNDS=(
  "asus-kbd-hard"
  "asus-kbd-medium"
  "asus-kbd-soft"
  "asus-kbd-space"
  "asus-mouse"
  "camera-latch"
  "carson-microflip"
  "ergo-mx"
  "guitar-toggle-off"
  "guitar-toggle-on"
  "halo-hard"
  "halo-shift"
  "halo-soft"
  "halo-space"
  "heated-blanket"
  "hi75-hard"
  "hi75-knob"
  "hi75-medium"
  "hi75-soft"
  "hi75-space"
  "iidx"
  "kvm-switch"
  "led-blacklight"
  "lock-closed"
  "lock-open"
  "low-thumps"
  "mx-master-3"
  "popper-dice"
  "popper-empty"
  "ps5-clicky"
  "ps5-double"
  "ps5-dull"
  "ps5-paddle"
  "ps5-snappy"
  "raddy-high"
  "raddy-low"
  "raddy-medium"
  "rival-3"
  "rival-3-middle"
  "sd-clicky"
  "sd-distorted"
  "sd-metallic"
  "sd-resonant"
  "sd-squelchy"
  "sd-standard"
  "synth-off"
  "synth-on"
  "tonal-off"
  "tonal-on"
  "treadmill-flinty"
  "treadmill-pitched"
  "treadmill-sharp"
  "uhk-medium"
  "uhk-soft"
  "uhk-space"
)

mkdir -p "${OUT_DIR}"

echo "Fetching ${#SOUNDS[@]} audio sprites from joshwcomeau/button-sounds -> ${OUT_DIR}"

download_one() {
  local name="$1"
  local dest="${OUT_DIR}/${name}.mp3"
  if [[ -f "${dest}" && -s "${dest}" ]]; then
    return 0
  fi
  curl -fsSL "${BASE_URL}/${name}.mp3" -o "${dest}"
}

export -f download_one
export OUT_DIR BASE_URL

printf '%s\n' "${SOUNDS[@]}" | xargs -P 8 -I {} bash -c 'download_one "$@"' _ {}

count=$(find "${OUT_DIR}" -maxdepth 1 -name '*.mp3' -size +0c | wc -l | tr -d ' ')
echo "Done: ${count}/${#SOUNDS[@]} MP3 sprites ready in ${OUT_DIR}"
if [[ "${count}" -ne "${#SOUNDS[@]}" ]]; then
  echo "Error: expected ${#SOUNDS[@]} files, found ${count}" >&2
  exit 1
fi
