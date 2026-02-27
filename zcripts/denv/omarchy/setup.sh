#!/bin/bash
set -e

echo "setting up zwipe development environment for omarchy (arch linux)..."

# check if running on arch linux
if ! grep -q "ID=arch" /etc/os-release; then
    echo "error: this script is for arch linux (omarchy) only"
    exit 1
fi

# update package database
echo "updating package database..."
sudo pacman -Syu --noconfirm

# install system dependencies
echo "installing system dependencies..."
sudo pacman -S --needed --noconfirm base-devel curl git openssl pkgconf

# install dioxus dependencies
echo "installing dioxus dependencies..."
sudo pacman -S --needed --noconfirm webkit2gtk-4.1 xdotool libappindicator-gtk3
sudo pacman -S --needed --noconfirm gtk3 glib2 librsvg

# install postgresql
echo "installing postgresql..."
if ! command -v psql &> /dev/null; then
    sudo pacman -S --needed --noconfirm postgresql

    # initialize database
    if [ ! -f /var/lib/postgres/data/postgresql.conf ]; then
        sudo -u postgres initdb -D /var/lib/postgres/data
    fi

    sudo systemctl enable postgresql
fi

# start postgresql
sudo systemctl start postgresql

# install sqlx-cli
echo "installing sqlx-cli..."
if ! command -v sqlx &> /dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres
fi

# install dioxus cli
echo "installing dioxus cli..."
if ! command -v dx &> /dev/null; then
    cargo install dioxus-cli
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
echo "running database migrations..."
cd zerver
sqlx migrate run
cd ..

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
