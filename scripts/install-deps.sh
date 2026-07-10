#!/bin/bash
set -e

if command -v apt-get &>/dev/null; then
    echo "Installing build dependencies with apt-get..."
    sudo apt-get update
    sudo apt-get install -y libx11-dev libxrandr-dev libxext-dev libwayland-dev pkg-config
elif command -v dnf &>/dev/null; then
    echo "Installing build dependencies with dnf..."
    sudo dnf install -y libX11-devel libXrandr-devel libXext-devel wayland-devel pkgconfig
elif command -v pacman &>/dev/null; then
    echo "Installing build dependencies with pacman..."
    sudo pacman -S --needed libx11 libxrandr libxext wayland pkgconf
else
    echo "Unsupported package manager. Install the equivalent X11/Wayland development headers for your distro manually:"
    echo "  - libx11-dev / libX11-devel"
    echo "  - libxrandr-dev / libXrandr-devel"
    echo "  - libxext-dev / libXext-devel"
    echo "  - libwayland-dev / wayland-devel"
    echo "  - pkg-config / pkgconfig"
    exit 1
fi

echo "Build dependencies installed."
