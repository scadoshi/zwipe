#!/bin/bash
# setup.sh

set -e  # Exit on any error

echo "🚀 Full zwipe development environment setup"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if running on WSL/Ubuntu
if ! grep -q "Ubuntu" /etc/os-release; then
    print_error "This script is designed for Ubuntu/WSL. Exiting."
    exit 1
fi

# Update package list
print_status "Updating package list..."
sudo apt update

# Install essential build tools
print_status "Installing essential build tools..."
sudo apt install -y build-essential curl git openssl pkg-config ripgrep

# Install mold linker
if ! command -v mold &> /dev/null; then
    print_status "Installing mold linker..."
    cargo install mold
    print_status "Mold linker installed successfully"
else
    print_status "Mold linker already installed"
fi

# Install GitHub CLI
if ! command -v gh &> /dev/null; then
    print_status "Installing GitHub CLI..."
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
    sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
    sudo apt update
    sudo apt install -y gh
else
    print_status "GitHub CLI already installed"
fi

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    print_status "Rust installed successfully"
else
    print_status "Rust already installed"
fi

# Install PostgreSQL if not present
if ! command -v psql &> /dev/null; then
    print_status "Installing PostgreSQL..."
    sudo apt install -y postgresql postgresql-contrib
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
    print_status "PostgreSQL installed and started"
else
    print_status "PostgreSQL already installed"
    # Make sure it's running
    if ! systemctl is-active --quiet postgresql; then
        sudo systemctl start postgresql
    fi
fi

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    print_status "Installing sqlx-cli..."
    cargo install sqlx-cli
fi

# Check GitHub CLI authentication
print_status "Checking GitHub authentication..."
if ! gh auth status &> /dev/null; then
    print_warning "Not authenticated with GitHub CLI"
    echo "🔐 You need to authenticate with GitHub to clone the repository."
    echo "Starting GitHub CLI authentication process..."
    gh auth login
    
    # Verify authentication worked
    if ! gh auth status &> /dev/null; then
        print_error "GitHub authentication failed. Please try again manually with 'gh auth login'"
        exit 1
    fi
    print_status "GitHub authentication successful!"
else
    print_status "Already authenticated with GitHub CLI"
fi

# Determine if we need to clone and where we are
CURRENT_DIR=$(basename "$(pwd)")
PARENT_DIR=$(basename "$(dirname "$(pwd)")")

if [[ "$CURRENT_DIR" == "zwipe" || "$CURRENT_DIR" == "zerver" || "$PARENT_DIR" == "zwipe" ]]; then
    print_status "Already inside zwipe project, no cloning needed"
    # Navigate to project root if we're in a subdirectory
    if [[ "$CURRENT_DIR" == "zerver" ]]; then
        cd ..
    elif [[ "$PARENT_DIR" == "zwipe" ]]; then
        cd ..
    fi
elif [ -d "zwipe" ]; then
    print_status "Repository already exists, updating..."
    cd zwipe
    git pull origin main
else
    print_status "Cloning zwipe repository..."
    gh repo clone scadoshi/zwipe
    cd zwipe
fi

# Setup PostgreSQL user and database
print_status "Setting up PostgreSQL user and database..."
sudo -u postgres psql -c "CREATE USER zwiper WITH PASSWORD 'pazzword';" 2>/dev/null || print_warning "User might already exist"
sudo -u postgres psql -c "ALTER USER zwiper CREATEDB;" 2>/dev/null || true

# Create the database and grant proper permissions
sudo -u postgres psql -c "DROP DATABASE IF EXISTS zerver;"
sudo -u postgres psql -c "CREATE DATABASE zerver OWNER zwiper;"
sudo -u postgres psql -d zerver -c "GRANT ALL PRIVILEGES ON SCHEMA public TO zwiper;"
sudo -u postgres psql -d zerver -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO zwiper;"
sudo -u postgres psql -d zerver -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO zwiper;"
sudo -u postgres psql -d zerver -c "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO zwiper;"
sudo -u postgres psql -d zerver -c "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO zwiper;"

# Create .env file
print_status "Creating .env configuration..."
cat > zerver/.env << EOF
# App State
JWT_SECRET=$(openssl rand -base64 32)
DATABASE_URL=postgres://zwiper:pazzword@localhost/zerver
BIND_ADDRESS=127.0.0.1:3000
# CORS Configuration
ALLOWED_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
# Rust
RUST_LOG=info
RUST_BACKTRACE=0
EOF

# Setup database
cd zerver

print_status "Running migrations..."
sqlx migrate run

# Mold configuration already exists in project

# Build the project to ensure everything works
print_status "Building project with mold..."
cargo build

print_status "🎉 Setup complete!"
echo ""
echo "📂 Project location: $(pwd)"
echo "🗄️  Database: zerver"
echo "👤 DB User: zwiper"
echo "🔑 DB Password: pazzword"
echo "🔗 Linker: mold (configured)"
echo "🐙 GitHub CLI: $(gh --version | head -1)"
echo ""
echo "🚀 Opening Cursor in zwipe directory..."
cursor ../