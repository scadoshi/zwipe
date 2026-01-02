#!/bin/bash
set -e

echo "resetting zwipe database for fedora..."

# check if running on fedora
if ! grep -q "Fedora" /etc/os-release; then
    echo "error: this script is for fedora only"
    exit 1
fi

echo ""
echo "warning: this will drop and recreate the zerver database"
read -p "continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "cancelled"
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
cat > zerver/.env << EOF
# app state
JWT_SECRET=$(openssl rand -base64 32)
DATABASE_URL=postgres:///zerver?user=$CURRENT_USER
BIND_ADDRESS=127.0.0.1:3000
# cors configuration
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
# rust
RUST_LOG=info
RUST_BACKTRACE=0
EOF

cat > zwiper/.env << EOF
# app state
BACKEND_URL=http://127.0.0.1:3000
# rust
RUST_LOG=info
RUST_BACKTRACE=0
EOF

# run migrations
echo "running migrations..."
cd zerver
sqlx migrate run
cd ..

echo ""
echo "reset complete"
