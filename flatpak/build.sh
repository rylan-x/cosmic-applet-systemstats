#!/bin/bash
# Helper script to build the Flatpak package

set -e

echo "==> Generating Cargo sources..."
python3 flatpak/flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json

echo "==> Building Flatpak..."
flatpak-builder --user --install --force-clean build-dir flatpak/io.github.rylan_x.cosmic-applet-systemstats.yml

echo ""
echo "==> Build complete! Run with:"
echo "    flatpak run io.github.rylan_x.cosmic-applet-systemstats"
