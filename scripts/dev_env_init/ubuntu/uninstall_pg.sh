#!/bin/bash
# uninstall_pg.sh - Ubuntu PostgreSQL uninstaller

set -e  # Exit on any error

echo "🧹 PostgreSQL uninstaller for Ubuntu"

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

# Check if running on Ubuntu
if ! grep -q "Ubuntu" /etc/os-release; then
    print_error "This script is designed for Ubuntu/WSL. Exiting."
    exit 1
fi

# Get current username
CURRENT_USER=$(whoami)

# Stop PostgreSQL service if running
print_status "Stopping PostgreSQL service..."
sudo systemctl stop postgresql 2>/dev/null || true
sudo systemctl disable postgresql 2>/dev/null || true

# Drop database if it exists
print_status "Dropping zerver database..."
sudo -u postgres psql -c "DROP DATABASE IF EXISTS zerver;" 2>/dev/null || true

# Drop user if it exists
print_status "Dropping database user: $CURRENT_USER..."
sudo -u postgres psql -c "DROP USER IF EXISTS \"$CURRENT_USER\";" 2>/dev/null || true

# Remove PostgreSQL packages
print_status "Removing PostgreSQL packages..."
sudo apt remove --purge -y postgresql postgresql-contrib postgresql-client-common postgresql-common

# Remove PostgreSQL data directory
print_status "Removing PostgreSQL data directory..."
sudo rm -rf /var/lib/postgresql/

# Remove PostgreSQL configuration files
print_status "Removing PostgreSQL configuration..."
sudo rm -rf /etc/postgresql/

# Clean up package cache
print_status "Cleaning package cache..."
sudo apt autoremove -y
sudo apt autoclean

print_status "🎉 PostgreSQL completely uninstalled!"
echo ""
echo "📂 All PostgreSQL data has been removed"
echo "🗄️  Database 'zerver' deleted"
echo "👤 User '$CURRENT_USER' removed from PostgreSQL"
echo "🧹 PostgreSQL packages and data directories cleaned"
