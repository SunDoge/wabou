#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 && $# != 4 && $# != 6 )); then
  echo "usage: $0 <app> <out.png> [width height [click-x click-y]]" >&2
  exit 2
fi

app=$1
out=$2
width=${3:-1440}
height=${4:-900}
bundle="dist/${app}/resources/bundle.js"

mise exec -- bun run wabou build --app-dir "apps/${app}"

args=(
  --js "$bundle"
  --mode png
  --out "$out"
  --width "$width"
  --height "$height"
)
if (( $# == 6 )); then
  args+=(--click "$5" "$6")
fi

cargo run -q -p wabou-quick -- "${args[@]}"
echo "$out"
