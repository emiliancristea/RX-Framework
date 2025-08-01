#!/bin/bash

echo "Setting up RX Framework Git repository..."
echo

# Initialize git repository
git init
echo "Git repository initialized."

# Add the remote repository
git remote add origin https://github.com/emiliancristea/RX-Framework.git
echo "Remote repository added."

# Add all files
git add .
echo "All files added to staging."

# Create initial commit
git commit -m "Initial commit: RX Framework v0.1.0

- Complete cross-platform GUI framework for Rust
- Native backends for Windows (Win32), Linux (X11), and macOS (Cocoa)
- Widget system with Button, Label, TextInput, Container
- Layout management with FlexLayout, GridLayout, AbsoluteLayout
- 2D drawing canvas with shapes, text, and colors
- Comprehensive event system for mouse, keyboard, and window events
- Zero-warning compilation with production-ready code quality
- Full documentation and examples
- CI/CD pipeline with GitHub Actions
- Cross-platform testing and automated releases"

echo "Initial commit created."

# Push to GitHub
git branch -M main
git push -u origin main
echo "Repository pushed to GitHub!"

echo
echo "========================================"
echo "RX Framework successfully uploaded!"
echo "Repository: https://github.com/emiliancristea/RX-Framework"
echo "========================================"
echo