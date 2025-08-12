#!/bin/bash
# full-setup.sh

set -e  # Exit on any error

echo "🚀 Full deck-builder development environment setup"

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
sudo apt install -y build-essential curl git openssl pkg-config

# Install mold linker
if ! command -v mold &> /dev/null; then
    print_status "Installing mold linker..."
    sudo apt install -y mold
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

# Clone the repository if it doesn't exist
if [ ! -d "deck-builder" ]; then
    print_status "Cloning deck-builder repository..."
    gh repo clone scadoshi/deck-builder
    cd deck-builder
else
    print_status "Repository already exists, updating..."
    cd deck-builder
    git pull origin main
fi

# Setup PostgreSQL user and database
print_status "Setting up PostgreSQL user and database..."
sudo -u postgres psql -c "CREATE USER deck_builder_user WITH PASSWORD 'deck_builder_pass';" 2>/dev/null || print_warning "User might already exist"
sudo -u postgres psql -c "ALTER USER deck_builder_user CREATEDB;" 2>/dev/null || true
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE postgres TO deck_builder_user;" 2>/dev/null || true

# Create .env file
print_status "Creating .env configuration..."
cat > deck_builder_api/.env << EOF
DATABASE_URL=postgres://deck_builder_user:deck_builder_pass@localhost/deck_builder
ALLOWED_ORIGINS=http://localhost:3000
BIND_ADDRESS=0.0.0.0:8080
JWT_SECRET=$(openssl rand -base64 32)
EOF

# Setup database
cd deck_builder_api
print_status "Creating database..."
sqlx database create

print_status "Running migrations..."
sqlx migrate run

# Configure mold for Rust builds
print_status "Configuring mold linker for Rust..."
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF

# Build the project to ensure everything works
print_status "Building project with mold..."
cargo build

print_status "🎉 Setup complete!"
echo ""
echo "📂 Project location: $(pwd)"
echo "🗄️  Database: deck_builder"
echo "👤 DB User: deck_builder_user"
echo "🔑 DB Password: deck_builder_pass"
echo "🔗 Linker: mold (configured)"
echo "🐙 GitHub CLI: $(gh --version | head -1)"
echo ""
echo "🚀 To start the server:"
echo "   cd deck-builder/deck_builder_api"
echo "   cargo run"
echo ""
echo "🔍 To test the API:"
echo "   curl http://localhost:8080/health"