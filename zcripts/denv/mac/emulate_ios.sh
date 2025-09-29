#!/bin/bash
# Start iPhone 16 simulator for Dioxus development

set -e

# Check macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This script requires macOS"
    exit 1
fi

# Check Xcode
if ! command -v xcrun &> /dev/null; then
    echo "Error: Xcode command line tools not found. Run 'xcode-select --install'"
    exit 1
fi

# Use iPhone 16 directly
SIMULATOR_NAME="iPhone 16"

echo "Starting iPhone 16 simulator..."

# Boot simulator if not already running
if ! xcrun simctl list devices | grep -q "$SIMULATOR_NAME.*Booted"; then
    xcrun simctl boot "$SIMULATOR_NAME"
    echo "Waiting for simulator to boot..."
    sleep 3
fi

# Open Simulator app
open -a Simulator

echo "iPhone 16 simulator ready"
