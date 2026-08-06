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
args=(
  render
  --app-dir "apps/${app}"
  --out "$out"
  --width "$width"
  --height "$height"
)
if (( $# == 6 )); then
  args+=(--click "$5" "$6")
fi

mise exec -- bun run wabou -- "${args[@]}"
echo "$out"
