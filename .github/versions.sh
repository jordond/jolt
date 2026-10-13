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

# Re-resolve Cargo.lock so it pins the versions just written to Cargo.toml.
# Without this the lock file keeps the previous versions and tagged releases
# ship a stale lock (see issue #159).
#
# Never fails the script: this also runs in jobs that only need the manifest
# rewritten, and a missing cargo or an unreachable registry must not break them.
update_lockfile() {
	if ! command -v cargo >/dev/null 2>&1; then
		echo "cargo not found, leaving Cargo.lock untouched" >&2
		return 0
	fi

	# --workspace re-resolves only the local crates, so third-party
	# dependencies keep the versions already pinned in the lock file.
	# Offline first: re-resolving path dependencies needs no registry lookup.
	if cargo update --workspace --offline --quiet; then
		return 0
	fi

	echo "Offline Cargo.lock update failed, retrying with network access" >&2
	if ! cargo update --workspace --quiet; then
		echo "Warning: could not update Cargo.lock" >&2
	fi

	return 0
}

echo "Updating version to ${VERSION}"

sed -i.bak 's/^version = ".*"/version = "'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-protocol = { path = "crates\/protocol", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-theme = { path = "crates\/theme", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
sed -i.bak 's/\(jolt-platform = { path = "crates\/platform", version = "\)[^"]*"/\1'"${VERSION}"'"/' Cargo.toml
rm -f Cargo.toml.bak

update_lockfile

echo "Updated versions:"
grep -E '^version|jolt-(protocol|theme|platform)' Cargo.toml
grep -A1 '^name = "jolt-tui"' Cargo.lock || echo "jolt-tui not found in Cargo.lock" >&2
