#!/bin/bash
#
# Setup script to configure git hooks for the project
#

echo "Setting up git hooks for the project..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: This is not a git repository."
    exit 1
fi

# Configure git to use the .githooks directory
git config core.hooksPath .githooks

# Make sure the hooks are executable
chmod +x .githooks/*

echo "Git hooks configured successfully!"
echo "Now format checks will run automatically on every commit."
echo ""
echo "To bypass checks (not recommended), use: git commit --no-verify"
echo "To run checks manually: .githooks/pre-commit"
