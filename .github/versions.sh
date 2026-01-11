#!/usr/bin/env bash
# Usage: ./versions.sh <version>
# Example: ./versions.sh 1.2.3

set -euo pipefail

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
	echo "Usage: $0 <version>" >&2
	echo "Example: $0 1.2.3" >&2
	exit 1
fi

VERSION="${VERSION#v}"

echo "Updating version to ${VERSION}"

sed -i.bak 's/^version = ".*"/version = "'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-protocol = { path = "crates\/protocol", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-theme = { path = "crates\/theme", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-platform = { path = "crates\/platform", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
rm -f Cargo.toml.bak

echo "Updated versions:"
grep -E '^version|jolt-(protocol|theme|platform)' Cargo.toml
