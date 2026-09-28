#!/usr/bin/env bash
# Deploy a release build to the connected iPhone: optimized, what the store
# will run.
#
# Usage: zcripts/ios/deploy_release.sh [--device NAME]
set -euo pipefail
exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/deploy.sh" --release "$@"
