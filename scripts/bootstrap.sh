#!/usr/bin/env bash
#
# The script help you set up your develop envirnment
#
# --fast: fast mode will skip OS pacakge dependency, only install git hooks and Rust
#

if [[ "$1" != "--fast" ]]; then
    if [[ "$OSTYPE" == "linux-gnu" ]]; then
        set -e
        if [ -f /etc/redhat-release ]; then
            echo "Redhat Linux detected, but current not support sorry."
            echo "Contribution is always welcome."
            exit 1
        elif [ -f /etc/SuSE-release ]; then
            echo "Suse Linux detected, but current not support sorry."
            echo "Contribution is always welcome."
            exit 1
        elif [ -f /etc/arch-release ]; then
            echo "Arch Linux detected."
            sudo pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 clang llvm rocksdb curl
            export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
            export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
        elif [ -f /etc/mandrake-release ]; then
            echo "Mandrake Linux detected, but current not support sorry."
            echo "Contribution is always welcome."
            exit 1
        elif [ -f /etc/debian_version ]; then
            echo "Ubuntu/Debian Linux detected."
            sudo apt-get -y update
            sudo apt-get install -y cmake pkg-config libssl-dev
        else
            echo "Unknown Linux distribution."
            echo "Contribution is always welcome."
            exit 1
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        set -e
        echo "Mac OS (Darwin) detected."
        if ! which brew >/dev/null 2>&1; then
            /usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
        fi
        brew upgrade
        brew install openssl cmake llvm
    elif [[ "$OSTYPE" == "freebsd"* ]]; then
        echo "FreeBSD detected, but current not support sorry."
        echo "Contribution is always welcome."
        exit 1
    else
        echo "Unknown operating system."
        echo "Contribution is always welcome."
        exit 1
    fi
fi

# Setup git hooks
cp .hooks/* .git/hooks

# Install nightly Rust
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=nightly -y

# Install wasm toolchain
rustup target add wasm32-unknown-unknown

# Install rustfmt for coding style checking
rustup component add rustfmt --toolchain nightly
