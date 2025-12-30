#!/bin/bash
# macos_dev_setup.sh

set -e  # Exit on any error

echo "🚀 Full zwipe development environment setup for macOS"

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

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. Exiting."
    exit 1
fi

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    print_status "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH for Apple Silicon Macs
    if [[ $(uname -m) == "arm64" ]]; then
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
    
    print_status "Homebrew installed successfully"
else
    print_status "Homebrew already installed"
fi

# Update Homebrew
print_status "Updating Homebrew..."
brew update

# Install essential build tools
print_status "Installing essential build tools..."
brew install gcc cmake autoconf automake libtool pkg-config
brew install curl git openssl ripgrep

# Note: Using system linker (mold only supports Linux binaries on macOS)
print_status "Using system linker (optimized for macOS)"

# Install GitHub CLI
if ! command -v gh &> /dev/null; then
    print_status "Installing GitHub CLI..."
    brew install gh
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

# Remove any existing mold configuration from cargo config
if [ -f ~/.cargo/config.toml ]; then
    print_status "Removing mold configuration from cargo config..."
    # Create a backup and remove mold-related lines
    cp ~/.cargo/config.toml ~/.cargo/config.toml.backup 2>/dev/null || true
    grep -v "fuse-ld.*mold" ~/.cargo/config.toml > ~/.cargo/config.toml.tmp 2>/dev/null && mv ~/.cargo/config.toml.tmp ~/.cargo/config.toml || true
fi

# Install PostgreSQL if not present
if ! command -v psql &> /dev/null; then
    print_status "Installing PostgreSQL..."
    brew install postgresql@15
    
    # Start PostgreSQL service
    brew services start postgresql@15
    
    # Add PostgreSQL to PATH
    echo 'export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"' >> ~/.zprofile
    export PATH="/opt/homebrew/opt/postgresql@15/bin:$PATH"
    
    print_status "PostgreSQL installed and started"
else
    print_status "PostgreSQL already installed"
    # Make sure it's running
    if ! brew services list | grep postgresql | grep started &> /dev/null; then
        brew services start postgresql@15
    fi
fi

# Install cargo-binstall for efficient binary installation
if ! command -v cargo-binstall &> /dev/null; then
    print_status "Installing cargo-binstall..."
    # Try homebrew first (faster), then fallback to curl
    if command -v brew &> /dev/null; then
        brew install cargo-binstall
    else
        curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
        source $HOME/.cargo/env
    fi
    print_status "cargo-binstall installed successfully"
else
    print_status "cargo-binstall already installed"
fi

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    print_status "Installing sqlx-cli..."
    cargo binstall sqlx-cli --no-confirm
fi

# Install Dioxus CLI if not present
if ! command -v dx &> /dev/null; then
    print_status "Installing Dioxus CLI..."
    cargo binstall dioxus-cli --no-confirm
    print_status "Dioxus CLI installed successfully"
else
    print_status "Dioxus CLI already installed"
fi

# Check GitHub CLI authentication
# print_status "Checking GitHub authentication..."
# if ! gh auth status &> /dev/null; then
#     print_warning "Not authenticated with GitHub CLI"
#     echo "🔐 You need to authenticate with GitHub to clone the repository."
#     echo "Starting GitHub CLI authentication process..."
#     gh auth login
#
#     # Verify authentication worked
#     if ! gh auth status &> /dev/null; then
#         print_error "GitHub authentication failed. Please try again manually with 'gh auth login'"
#         exit 1
#     fi
#     print_status "GitHub authentication successful!"
# else
#     print_status "Already authenticated with GitHub CLI"
# fi

# Function to find project root reliably
find_project_root() {
    local current_dir="$(pwd)"
    local original_dir="$current_dir"

    # Walk up directory tree looking for project root indicators
    while [[ "$(basename "$current_dir")" != "/" ]]; do
        # Check if this looks like the project root
        if [[ -f "$current_dir/zerver/Cargo.toml" && -f "$current_dir/zwiper/Cargo.toml" ]]; then
            echo "$current_dir"
            return 0
        fi

        # Check if this is the zwipe directory itself
        if [[ "$(basename "$current_dir")" == "zwipe" ]]; then
            echo "$current_dir"
            return 0
        fi

        current_dir="$(dirname "$current_dir")"
    done

    # If we didn't find it, check if zwipe exists in original location
    if [[ -d "$original_dir/zwipe" ]]; then
        echo "$original_dir/zwipe"
        return 0
    fi

    # Default: assume we're in project root or will clone
    echo "$original_dir"
    return 1
}

# Determine project root and navigate there
PROJECT_ROOT=$(find_project_root)
PROJECT_ROOT_FOUND=$?

if [[ $PROJECT_ROOT_FOUND -eq 0 ]]; then
    print_status "Found project root at: $PROJECT_ROOT"
    cd "$PROJECT_ROOT"
else
    print_status "Project not found locally, will clone repository"
    print_status "Cloning zwipe repository..."
    gh repo clone scadoshi/zwipe
    PROJECT_ROOT="$(pwd)/zwipe"
    cd "$PROJECT_ROOT"
    print_status "Cloned to: $PROJECT_ROOT"
fi

# Setup PostgreSQL database
print_status "Setting up PostgreSQL database..."

# Get current username for database user
CURRENT_USER=$(whoami)

# Create a database user matching the current Unix user for peer authentication
createdb "$CURRENT_USER" 2>/dev/null || true

# Drop and recreate database with proper ownership
dropdb zerver 2>/dev/null || true
createdb zerver

# Create .env file
print_status "Creating .env configuration..."
cat > "$PROJECT_ROOT/zerver/.env" << EOF
# App State
JWT_SECRET=$(openssl rand -base64 32)
DATABASE_URL=postgres:///zerver?user=$CURRENT_USER
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

# Build the project to ensure everything works
print_status "Building backend project..."
cargo build

# Build and test Dioxus frontend
print_status "Setting up Dioxus frontend..."
cd ../zwiper

# Test Dioxus mobile build
print_status "Testing Dioxus mobile build..."
if dx build --platform mobile; then
    print_status "Dioxus mobile build successful"
else
    print_warning "Dioxus mobile build failed, but CLI is installed. You can debug with 'dx build --platform mobile' or try 'dx build --platform desktop' as fallback"
fi

print_status "🎉 Setup complete!"
echo ""
echo "📂 Project location: $(pwd)"
echo "🗄️  Database: zerver"
echo "👤 DB User: $CURRENT_USER"
echo "🔐 Auth: peer (no password needed)"
echo "🔗 Linker: system default (optimized for macOS)"
echo "🐙 GitHub CLI: $(gh --version | head -1)"
echo "📱 Dioxus CLI: $(dx --version)"
echo ""
echo "🚀 To start development:"
echo "   Backend:  cd zerver && cargo run"
echo "   Frontend: cd zwiper && dx serve --platform mobile"
echo ""
echo "💡 Mobile development options:"
echo "   - Primary: dx serve --platform mobile (native mobile experience)"
echo "   - Fallback: dx serve --platform desktop (if mobile doesn't work)"
echo "   - Web testing: dx serve --platform web"
echo "   - iOS testing: Perfect for your Mac setup with true iOS emulation"
