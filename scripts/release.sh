#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: pnpm release <version>"
  echo ""
  echo "Bumps the version in Cargo.toml and tauri.conf.json,"
  echo "commits the change, creates a git tag, and pushes both."
  echo ""
  echo "Examples:"
  echo "  pnpm release 0.3.0"
  echo "  pnpm release 1.0.0-beta.1"
  exit 1
}

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  usage
fi

# Strip leading 'v' if provided
VERSION="${VERSION#v}"
TAG="v${VERSION}"

# Ensure we're in the repo root
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Ensure working tree is clean
if [ -n "$(git status --porcelain)" ]; then
  echo "Error: working tree is not clean. Commit or stash changes first."
  exit 1
fi

# Ensure we're on main
BRANCH="$(git branch --show-current)"
if [ "$BRANCH" != "main" ]; then
  echo "Error: releases should be created from main (currently on '$BRANCH')."
  exit 1
fi

# Ensure tag doesn't already exist
if git rev-parse "$TAG" &>/dev/null; then
  echo "Error: tag $TAG already exists."
  exit 1
fi

# Update Cargo.toml version
sed -i "0,/^version = \".*\"/s//version = \"${VERSION}\"/" src-tauri/Cargo.toml

# Update tauri.conf.json version
node -e "
  const fs = require('fs');
  const path = 'src-tauri/tauri.conf.json';
  const conf = JSON.parse(fs.readFileSync(path, 'utf8'));
  conf.version = process.argv[1];
  fs.writeFileSync(path, JSON.stringify(conf, null, 2) + '\n');
" "$VERSION"

echo "Bumped version to ${VERSION}"

# Commit, tag, push
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to ${VERSION}"
git tag -a "$TAG" -m "Release ${TAG}"
git push origin main "$TAG"

echo ""
echo "Pushed ${TAG} — CI will handle the release."
