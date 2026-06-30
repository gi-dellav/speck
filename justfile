# Justfile
# https://github.com/casey/just

[private]
default:
    @just --list

# ---- Build ----

build:
    cargo build --release

run *args:
    cargo run -- {{ args }}

# ---- Quality ----

fmt:
    cargo fmt
    cargo clippy --all-targets -- -D warnings

check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

test: fmt
    cargo test

# ---- Git hooks ----

install-hook:
    #!/usr/bin/env bash
    cat > .git/hooks/pre-commit << 'EOF'
    #!/bin/sh
    set -e
    echo "Running pre-commit quality checks..."
    just check
    EOF
    chmod +x .git/hooks/pre-commit
    echo "Pre-commit hook installation confirmed."

remove-hook:
    rm .git/hooks/pre-commit
    echo "Pre-commit hook uninstallation confirmed."

# ---- Tags ----

add-tag:
    #!/usr/bin/env bash
    set -euo pipefail
    git push origin main
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    git tag -a "v${VERSION}" -m "Release v${VERSION}"
    git push origin "v${VERSION}"
    echo "Created and pushed tag v${VERSION}"

remove-tag VERSION="":
    #!/usr/bin/env bash
    set -e
    tag="{{ VERSION }}"
    if [ -z "$tag" ]; then
        tag=$(git tag | sort -V | fzf --prompt="Select tag to remove: ")
    fi
    if [ -z "$tag" ]; then
        echo "No tag selected"
        exit 1
    fi
    git tag -d "$tag" || {
        echo "Local tag not found"
        exit 1
    }
    git push --delete origin "$tag"
    echo "Removed tag $tag"

# ---- Packaging: version sync ----

sync-version:
    bash scripts/sync-version.sh

# ---- Packaging: checksums ----

homebrew-checksums:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Computing SHA256 sums for v${VERSION}..."

    SHA_DARWIN_X86=$(curl -sL "https://github.com/gi-dellav/speck/releases/download/v${VERSION}/speck-x86_64-apple-darwin.tar.gz" | sha256sum | cut -d' ' -f1)
    SHA_DARWIN_ARM=$(curl -sL "https://github.com/gi-dellav/speck/releases/download/v${VERSION}/speck-aarch64-apple-darwin.tar.gz" | sha256sum | cut -d' ' -f1)
    SHA_LINUX_X86=$(curl -sL "https://github.com/gi-dellav/speck/releases/download/v${VERSION}/speck-x86_64-unknown-linux-musl.tar.gz" | sha256sum | cut -d' ' -f1)
    SHA_LINUX_ARM=$(curl -sL "https://github.com/gi-dellav/speck/releases/download/v${VERSION}/speck-aarch64-unknown-linux-musl.tar.gz" | sha256sum | cut -d' ' -f1)

    sed -i "/speck-x86_64-apple-darwin.tar.gz/{n;s/sha256 \".*\"/sha256 \"${SHA_DARWIN_X86}\"/}" packaging/homebrew/speck.rb
    sed -i "/speck-aarch64-apple-darwin.tar.gz/{n;s/sha256 \".*\"/sha256 \"${SHA_DARWIN_ARM}\"/}" packaging/homebrew/speck.rb
    sed -i "/speck-x86_64-unknown-linux-musl.tar.gz/{n;s/sha256 \".*\"/sha256 \"${SHA_LINUX_X86}\"/}" packaging/homebrew/speck.rb
    sed -i "/speck-aarch64-unknown-linux-musl.tar.gz/{n;s/sha256 \".*\"/sha256 \"${SHA_LINUX_ARM}\"/}" packaging/homebrew/speck.rb

    echo "Updated SHA256 sums in packaging/homebrew/speck.rb"

# ---- Packaging: release workflow ----

pre-release: sync-version
    @echo "=== pre-release done: version synced across all packaging files ==="
    @echo "Next: just add-tag, wait for GitHub release, then: just post-release"

post-release: homebrew-checksums
    @echo "=== post-release done: all checksums updated ==="
    @echo "Ready for:"
    @echo "  homebrew: push packaging/homebrew/speck.rb to homebrew-tap repo"
