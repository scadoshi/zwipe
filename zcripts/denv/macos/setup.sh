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

# install dioxus cli
echo "installing dioxus cli..."
if ! command -v dx &> /dev/null; then
    cargo install dioxus-cli
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
