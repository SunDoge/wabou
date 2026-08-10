#!/usr/bin/env bash
set -euo pipefail

proxy_url="${WABOU_ACT_PROXY:-http://localhost:7890}"

# act uses the host network on Linux, so localhost inside a job container is
# the host proxy too. Export the variables both for act itself (action/image
# downloads) and for commands running inside each job container.
export HTTP_PROXY="$proxy_url"
export HTTPS_PROXY="$proxy_url"
export http_proxy="$proxy_url"
export https_proxy="$proxy_url"

exec act \
  --network host \
  --pull=false \
  --env "HTTP_PROXY=$proxy_url" \
  --env "HTTPS_PROXY=$proxy_url" \
  --env "http_proxy=$proxy_url" \
  --env "https_proxy=$proxy_url" \
  "$@"
