#!/bin/bash
set -e

echo "setting up zwipe development environment for macos..."

# check if running on macos
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "error: this script is for macos only"
    exit 1
fi

# check for xcode command line tools
if ! xcode-select -p &> /dev/null; then
    echo "error: xcode command line tools required for ios development"
    echo "install with: xcode-select --install"
    exit 1
fi

# command line tools alone are not enough: ios simulator builds
# (aarch64-apple-ios-sim) need the full xcode app + iphonesimulator sdk.
# a clt-only machine passes the check above but fails later in cc-rs with
# 'xcrun: error: SDK "iphonesimulator" cannot be located'.
if ! xcrun --show-sdk-path --sdk iphonesimulator &> /dev/null; then
    echo "error: iphonesimulator sdk not found — full xcode is required for ios builds"
    echo "  the active toolchain is: $(xcode-select -p)"
    echo "  (command line tools alone cannot build for the ios simulator)"
    echo ""
    echo "to fix:"
    echo "  1. install xcode from the app store"
    echo "  2. sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
    echo "  3. sudo xcodebuild -license accept"
    echo "  4. xcodebuild -downloadPlatform iOS   # installs the simulator runtime"
    echo "  5. re-run this script"
    exit 1
fi

# ensure an ios simulator runtime + device exist. the sdk check above lets you
# BUILD for the simulator; BOOTING one needs a runtime (a separate ~7gb download)
# plus a created device. without these, `dx serve --ios` fails with
# "No iOS sdks installed" / "No devices are booted".
if ! xcrun simctl list runtimes 2>/dev/null | grep -q "iOS "; then
    echo "warning: no ios simulator runtime installed — you can build but not launch a sim"
    echo "  install it with:  xcodebuild -downloadPlatform iOS"
    echo "  then re-run this script to provision a simulator device"
else
    # 6.5\" reference device. dx serve --ios launches into whatever sim is booted;
    # runtime is omitted so simctl picks the newest installed one automatically.
    SIM_NAME="iPhone 11 Pro Max"
    if xcrun simctl list devices | grep -q "^[[:space:]]*$SIM_NAME ("; then
        echo "simulator '$SIM_NAME' already present"
    else
        echo "creating '$SIM_NAME' simulator..."
        xcrun simctl create "$SIM_NAME" "$SIM_NAME" > /dev/null
    fi
fi

# install/verify homebrew
if ! command -v brew &> /dev/null; then
    echo "installing homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

    # add to PATH for apple silicon
    if [[ $(uname -m) == "arm64" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
fi

# update homebrew
echo "updating homebrew..."
brew update

# install system dependencies
echo "installing system dependencies..."
brew install openssl pkg-config

# install postgresql
echo "installing postgresql..."
if ! command -v psql &> /dev/null; then
    brew install postgresql@15
    echo 'export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"' >> ~/.zprofile
    export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"
fi

# start postgresql
brew services start postgresql@15

# install sqlx-cli
echo "installing sqlx-cli..."
if ! command -v sqlx &> /dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres
fi

# install dioxus cli — pinned to match the `dioxus` crate in zwiper/Cargo.toml.
# an unpinned `cargo install dioxus-cli` grabs the newest published version
# (including prereleases), which trips dx's "versions are incompatible" warning
# at serve time. keep DX_VERSION in lockstep with that Cargo.toml dependency.
# --locked uses dx's shipped Cargo.lock: without it cargo re-resolves transitive
# deps to newest-compatible and pulls git2 0.21, which fails to build auth-git2
# 0.5.8 (uses Cred::credential_helper, removed in git2 0.21).
# --force overwrites an existing dx of a different version (this guard only runs
# on a mismatch); without it cargo aborts with "binary `dx` already exists".
DX_VERSION="0.7.10"
echo "installing dioxus cli ($DX_VERSION)..."
if ! dx --version 2>/dev/null | grep -q "$DX_VERSION"; then
    cargo install dioxus-cli --version "$DX_VERSION" --locked --force
fi

# setup database
echo "setting up database..."
CURRENT_USER=$(whoami)

# create user database for peer auth
createdb "$CURRENT_USER" 2>/dev/null || true

# create zwipe database
dropdb zerver 2>/dev/null || true
createdb zerver

# create .env files
echo "creating .env files..."
cat > zerver/.env << EOF
# app state
JWT_SECRET=$(openssl rand -hex 32)
DATABASE_URL=postgres:///zerver?user=$CURRENT_USER
BIND_ADDRESS=127.0.0.1:3000
# cors configuration
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
# rust
# per-target directives parsed by tracing_subscriber::EnvFilter — see zerver/.env.example
RUST_LOG=info,sqlx=warn,zwipe=debug,zerver=debug
RUST_BACKTRACE=0
# log directory (zerver defaults to /var/log/zwipe in prod; /tmp is the dev-safe path)
LOG_DIR=/tmp/zwipe-logs
# email config (placeholder — dev doesn't send mail; swap in a real Resend key to test verify/reset flows)
RESEND_API_KEY=changeme
RESEND_EMAIL_FROM=support@zwipe.net
# user-facing support email shown in transactional emails (optional; default: support@zwipe.net)
SUPPORT_EMAIL_ADDRESS=support@zwipe.net
# client min-version gate (0.0.0 = open / allow everyone; flip to force updates)
MIN_CLIENT_VERSION=0.0.0
# public web base url for email verify/reset links + outbound User-Agent (optional; default: https://zwipe.net)
WEB_BASE_URL=https://zwipe.net
EOF

cat > zwiper/.env << EOF
# app state
BACKEND_URL=http://127.0.0.1:3000
# rust
RUST_LOG=info,zwiper=debug
RUST_BACKTRACE=0
EOF

# run migrations
echo "running database migrations..."
cd zerver
sqlx migrate run
cd ..

# scoped zervice role — dev parity with prod (throwaway local password).
# Lets a local `zervice` run prove the grants in zcripts/server/sql/ before
# a forgotten one can fail a nightly prod run.
echo "provisioning zervice role..."
psql -d zerver -f zcripts/server/sql/zervice_role.sql
psql -d zerver -c "ALTER ROLE zervice PASSWORD 'zervice'"

echo ""
echo "setup complete"
echo ""
echo "database: zerver"
echo "user: $CURRENT_USER"
echo "auth: peer (no password)"
echo ""
echo "to start development:"
echo "  backend:        cd zerver && cargo run --bin zerver"
echo "  frontend (web): cd zwiper && dx serve"
echo "  frontend (ios): open -a Simulator && (cd zwiper && dx serve --ios)"
echo "  service:        cargo run --bin zervice"
