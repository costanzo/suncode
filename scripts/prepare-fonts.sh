#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
lock_file="$repo_root/scripts/fonts.lock"
font_dir="$repo_root/apps/desktop-avalonia/Assets/fonts"

command -v curl >/dev/null 2>&1 || { echo "prepare-fonts: curl is required" >&2; exit 1; }

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

mkdir -p "$font_dir"

# The previous bundle used a variable Noto Sans SC face. Remove it so it
# cannot shadow the static faces below when rebuilding an existing checkout.
rm -f "$font_dir/NotoSansSC-wght.ttf"

while IFS='|' read -r name url expected_sha license || [[ -n "$name" ]]; do
  [[ -z "$name" || "$name" == \#* ]] && continue
  destination="$font_dir/$name"
  if [[ -f "$destination" ]] && [[ "$(sha256 "$destination")" == "$expected_sha" ]]; then
    continue
  fi
  tmp="$destination.tmp"
  rm -f "$tmp"
  echo "prepare-fonts: downloading $name ($license)"
  curl --fail --location --silent --show-error --retry 3 --output "$tmp" "$url"
  actual_sha="$(sha256 "$tmp")"
  if [[ "$actual_sha" != "$expected_sha" ]]; then
    rm -f "$tmp"
    echo "prepare-fonts: SHA-256 mismatch for $name (expected $expected_sha, got $actual_sha)" >&2
    exit 1
  fi
  mv "$tmp" "$destination"
done < "$lock_file"

echo "prepare-fonts: verified fonts in $font_dir"
