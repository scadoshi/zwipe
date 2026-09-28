#!/usr/bin/env bash
# Deploy a debug build to the connected iPhone. The daily one.
#
# Usage: zcripts/ios/deploy_debug.sh [--device NAME]
set -euo pipefail
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/deploy.sh" "$@"
