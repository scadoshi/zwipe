#!/bin/bash
set -e

echo "resetting zwipe database for ubuntu..."

# check if running on ubuntu
if ! grep -q "^ID=ubuntu" /etc/os-release; then
    echo "error: this script is for ubuntu only"
    exit 1
fi

echo ""
echo "warning: this will drop and recreate the zerver database"
read -p "continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "canceled"
    exit 0
fi

# drop and recreate database
echo "dropping database..."
CURRENT_USER=$(whoami)
sudo -u postgres psql -c "DROP DATABASE IF EXISTS zerver;"

echo "creating database..."
sudo -u postgres createuser --createdb --no-createrole --no-superuser "$CURRENT_USER" 2>/dev/null || true
sudo -u postgres psql -c "CREATE DATABASE zerver OWNER $CURRENT_USER;"

# recreate .env files
echo "recreating .env files..."
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
SUPPORT_EMAIL_ADDRESS=support@zwipe.net
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
echo "running migrations..."
cd zerver
sqlx migrate run
cd ..

# scoped zervice role: dev parity with prod (throwaway local password).
echo "provisioning zervice role..."
psql -d zerver -f zcripts/server/sql/zervice_role.sql
psql -d zerver -c "ALTER ROLE zervice PASSWORD 'zervice'"

echo ""
echo "reset complete"
