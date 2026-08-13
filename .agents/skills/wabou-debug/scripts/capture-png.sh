#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 && $# != 4 && $# != 6 )); then
  echo "usage: $0 <app> <out.png> [width height [click-x click-y]]" >&2
  exit 2
fi

app=$1
if [[ -f "${app}/Cargo.toml" ]]; then
  app_path=$app
else
  app_path="apps/${app}"
fi
out=$2
width=${3:-1440}
height=${4:-900}
args=(
  render
  "${app_path}"
  --out "$out"
  --width "$width"
  --height "$height"
  --window-id "${WABOU_CAPTURE_WINDOW_ID:-1}"
  --scale-factor "${WABOU_CAPTURE_SCALE_FACTOR:-1}"
)
if [[ -n "${WABOU_CAPTURE_MODE:-}" ]]; then
  args+=(--mode "$WABOU_CAPTURE_MODE")
fi
if (( $# == 6 )); then
  args+=(--click "$5" "$6")
fi

mise exec -- bun run wabou -- "${args[@]}"
echo "$out"
