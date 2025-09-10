#!/bin/bash
# uninstall_pg.sh - macOS PostgreSQL uninstaller

set -e  # Exit on any error

echo "🧹 PostgreSQL uninstaller for macOS"

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

# Get current username
CURRENT_USER=$(whoami)

# Stop PostgreSQL service if running
print_status "Stopping PostgreSQL service..."
brew services stop postgresql@15 2>/dev/null || true

# Drop database if it exists
print_status "Dropping zerver database..."
dropdb zerver 2>/dev/null || true

# Remove PostgreSQL packages
print_status "Removing PostgreSQL packages..."
brew uninstall postgresql@15 2>/dev/null || true

# Remove PostgreSQL data directory
print_status "Removing PostgreSQL data directory..."
rm -rf /opt/homebrew/var/postgresql@15/ 2>/dev/null || true
rm -rf /usr/local/var/postgresql@15/ 2>/dev/null || true

# Remove PostgreSQL from PATH in shell profiles
print_status "Cleaning up shell profile configurations..."
# Remove PostgreSQL PATH entries from common shell profiles
for profile in ~/.zprofile ~/.zshrc ~/.bash_profile ~/.bashrc; do
    if [ -f "$profile" ]; then
        # Remove lines containing postgresql@15 from PATH
        sed -i '' '/postgresql@15/d' "$profile" 2>/dev/null || true
    fi
done

print_status "🎉 PostgreSQL completely uninstalled!"
echo ""
echo "📂 All PostgreSQL data has been removed"
echo "🗄️  Database 'zerver' deleted"
echo "👤 User '$CURRENT_USER' removed from PostgreSQL"
echo "🧹 PostgreSQL packages and data directories cleaned"
echo "🔧 Shell profile configurations updated"
