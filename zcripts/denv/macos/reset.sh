#!/bin/bash
set -e

echo "resetting zwipe database for macos..."

# check if running on macos
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "error: this script is for macos only"
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
dropdb zerver 2>/dev/null || true

echo "creating database..."
CURRENT_USER=$(whoami)
createdb "$CURRENT_USER" 2>/dev/null || true
createdb zerver

# recreate .env file
echo "recreating .env file..."
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

# run migrations
echo "running migrations..."
cd zerver
sqlx migrate run
cd ..

echo ""
echo "reset complete"
