#!/bin/bash
set -e

echo "setting up zwipe development environment for ubuntu..."

# check if running on ubuntu
if ! grep -q "^ID=ubuntu" /etc/os-release; then
    echo "error: this script is for ubuntu only"
    exit 1
fi

# prod (18.6) and CI run postgres 18; pin it so older ubuntu releases match
PG_VERSION="18"

# update package lists
echo "updating package lists..."
sudo apt-get update

# install system dependencies
echo "installing system dependencies..."
sudo apt-get install -y build-essential curl wget file git libssl-dev pkg-config

# install dioxus dependencies
echo "installing dioxus dependencies..."
sudo apt-get install -y libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev libgtk-3-dev

# install postgresql from the postgresql.org apt repo (pgdg), pinned to PG_VERSION.
# falls back to ubuntu's default postgresql when pgdg has no build for this release yet.
echo "installing postgresql $PG_VERSION..."
if ! dpkg -s "postgresql-$PG_VERSION" &> /dev/null; then
    sudo apt-get install -y postgresql-common
    sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y || true
    if ! sudo apt-get install -y "postgresql-$PG_VERSION"; then
        echo "warning: postgresql-$PG_VERSION unavailable for this release, installing ubuntu's default (prod runs $PG_VERSION)"
        sudo apt-get install -y postgresql
    fi
fi

# start postgresql
sudo systemctl enable --now postgresql

# install sqlx-cli
echo "installing sqlx-cli..."
if ! command -v sqlx &> /dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres
fi

# install dioxus cli, pinned to match the `dioxus` crate in zwiper/Cargo.toml.
# keep DX_VERSION in lockstep with that dependency and with the other platforms.
# --locked keeps cargo off git2 0.21 (breaks auth-git2 0.5.8); --force replaces a
# dx of a different version (this guard only runs on a mismatch).
DX_VERSION="0.7.10"
echo "installing dioxus cli ($DX_VERSION)..."
if ! dx --version 2>/dev/null | grep -q "$DX_VERSION"; then
    cargo install dioxus-cli --version "$DX_VERSION" --locked --force
fi

# setup database
echo "setting up database..."
CURRENT_USER=$(whoami)

# create database user
sudo -u postgres createuser --createdb --no-createrole --no-superuser "$CURRENT_USER" 2>/dev/null || true

# create zwipe database
sudo -u postgres psql -c "DROP DATABASE IF EXISTS zerver;"
sudo -u postgres psql -c "CREATE DATABASE zerver OWNER $CURRENT_USER;"

# create .env files
echo "creating .env files..."
cat > zerver/.env << ENV
# app state
JWT_SECRET=$(openssl rand -hex 32)
DATABASE_URL=postgres:///zerver?user=$CURRENT_USER
BIND_ADDRESS=127.0.0.1:3000
# cors configuration
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
# rust
# per-target directives parsed by tracing_subscriber::EnvFilter, see zerver/.env.example
RUST_LOG=info,sqlx=warn,zwipe=debug,zerver=debug
RUST_BACKTRACE=0
# log directory (zerver defaults to /var/log/zwipe in prod; /tmp is the dev-safe path)
LOG_DIR=/tmp/zwipe-logs
# email config (placeholder: dev doesn't send mail; swap in a real Resend key to test verify/reset flows)
RESEND_API_KEY=changeme
RESEND_EMAIL_FROM=support@zwipe.net
# user-facing support email shown in transactional emails (optional; default: support@zwipe.net)
SUPPORT_EMAIL_ADDRESS=support@zwipe.net
# public web base url for email verify/reset links + outbound User-Agent (optional; default: https://zwipe.net)
WEB_BASE_URL=https://zwipe.net
ENV

cat > zwiper/.env << ENV
# app state
BACKEND_URL=http://127.0.0.1:3000
# rust
RUST_LOG=info,zwiper=debug
RUST_BACKTRACE=0
ENV

# run migrations
echo "running database migrations..."
cd zerver
sqlx migrate run
cd ..

# scoped zervice role: dev parity with prod (throwaway local password).
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
echo "  backend:  cd zerver && cargo run --bin zerver"
echo "  frontend: cd zwiper && dx serve"
echo "  service:  cargo run --bin zervice"
