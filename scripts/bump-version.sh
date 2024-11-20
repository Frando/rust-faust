#!/usr/bin/env bash

VERSION="$1"

cargo set-version -p faust-build $VERSION
cargo set-version -p faust-types $VERSION
cargo set-version -p faust-state $VERSION
cargo set-version -p faust-macro $VERSION

curl "https://img.shields.io/badge/version-$VERSION-pink" > assets/version-badge.svg

sed -i "/## Unreleased/a ## v$VERSION -- ??" CHANGELOG.md

nvim CHANGELOG.md

git add assets/version-badge.svg
git add */Cargo.toml
git add Cargo.toml Cargo.lock
git add CHANGELOG.md

git commit -m "v$VERSION"
git tag -a "v$VERSION" -m "v$VERSION" 

echo "To push new version & tag execute 'git push origin main --tags'"
