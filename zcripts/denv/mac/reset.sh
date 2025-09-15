#!/bin/bash
# reset.sh - Complete PostgreSQL reset and setup for macOS

set -e  # Exit on any error

echo "🔄 Complete PostgreSQL reset and zwipe setup for macOS"

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

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. Exiting."
    exit 1
fi

print_warning "This will completely remove PostgreSQL and all its data, then reinstall everything."
echo "This action cannot be undone!"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_status "Reset cancelled."
    exit 0
fi

# Step 1: Uninstall PostgreSQL
print_status "Step 1: Uninstalling PostgreSQL..."
bash "$SCRIPT_DIR/uninstall_pg.sh"

echo ""
print_status "PostgreSQL uninstall complete. Starting fresh setup..."
echo ""

# Step 2: Run setup
print_status "Step 2: Running complete setup..."
bash "$SCRIPT_DIR/setup.sh"

print_status "🎉 Reset complete!"
echo ""
echo "✨ PostgreSQL has been completely reset and zwipe environment is ready"
