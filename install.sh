#!/bin/sh
set -eu

repo=https://github.com/lilyco-42/ai-session-export
die() { printf '%s\n' "Error: $*" >&2; exit 1; }
case "$(uname -s)" in
  Linux) platform=unknown-linux-gnu ;;
  Darwin) platform=apple-darwin ;;
  *) die "Unsupported OS; on Windows use install.ps1." ;;
esac
case "$(uname -m)" in
  x86_64|amd64) arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *) die "Unsupported CPU architecture." ;;
esac
# Prefer native ARM64 on Apple Silicon even under Rosetta.
if [ "$platform" = apple-darwin ] && [ "$(sysctl -n hw.optional.arm64 2>/dev/null || true)" = 1 ]; then
  arch=aarch64
fi
target=$arch-$platform
name=ai-session-export-$target
asset=$name.tar.gz
prefix=${AI_SESSION_INSTALL_PREFIX:-"$HOME/.local"}
root=$prefix/share/ai-session-export
app=$root/app
bin=$prefix/bin
link=$bin/ai-session-export
for cmd in tar awk mktemp; do command -v "$cmd" >/dev/null || die "$cmd is required"; done
mkdir -p "$root" "$bin"
[ ! -L "$app" ] || die "Install directory is a symlink: $app"
if [ -e "$app" ] && [ ! -x "$app/ai-session-export-server" ]; then
  die "Refusing to replace an unrecognized directory: $app"
fi
if [ -e "$link" ] || [ -L "$link" ]; then
  [ -L "$link" ] && [ "$(readlink "$link")" = "$app/ai-session-export-server" ] || die "Existing command: $link"
fi
tmp=$(mktemp -d "$root/.install.XXXXXXXX")
cleanup() {
  if [ -d "$tmp/previous" ] && [ ! -e "$app" ]; then
    mv "$tmp/previous" "$app" || { echo "Restore manually from $tmp/previous" >&2; return; }
  fi
  rm -rf "$tmp"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
if [ -n "${AI_SESSION_RELEASE_DIR:-}" ]; then
  cp "$AI_SESSION_RELEASE_DIR/$asset" "$AI_SESSION_RELEASE_DIR/SHA256SUMS" "$tmp/"
else
  command -v curl >/dev/null || die "curl is required"
  tag=${AI_SESSION_VERSION:-}
  if [ -z "$tag" ]; then
    url=$(curl -fsSL --retry 3 -o /dev/null -w '%{url_effective}' "$repo/releases/latest")
    tag=${url##*/}
  fi
  case "$tag" in v[0-9]*) ;; *) die "Invalid release tag: $tag";; esac
  case "$tag" in *[!A-Za-z0-9._-]*) die "Invalid release tag: $tag";; esac
  for file in "$asset" SHA256SUMS; do
    curl -fsSL --retry 3 "$repo/releases/download/$tag/$file" -o "$tmp/$file"
  done
fi
expected=$(awk -v name="$asset" '$2 == name || $2 == "*" name {print $1}' "$tmp/SHA256SUMS")
[ "${#expected}" -eq 64 ] || die "Missing or invalid checksum for $asset"
case "$expected" in *[!0-9a-fA-F]*) die "Invalid SHA256";; esac
if command -v sha256sum >/dev/null; then
  actual=$(sha256sum "$tmp/$asset" | awk '{print $1}')
elif command -v shasum >/dev/null; then
  actual=$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')
else
  die "sha256sum or shasum is required"
fi
[ "$(printf '%s' "$expected" | tr A-F a-f)" = "$actual" ] || die "SHA256 mismatch"
tar -xzf "$tmp/$asset" -C "$tmp"
stage=$tmp/$name
[ -x "$stage/ai-session-export-server" ] && [ -f "$stage/web/dist/index.html" ] || die "Incomplete release archive"
if [ -e "$app" ]; then mv "$app" "$tmp/previous"; fi
mv "$stage" "$app"
if [ ! -L "$link" ]; then ln -s "$app/ai-session-export-server" "$link"; fi
printf 'Installed: %s\n' "$link"
case ":$PATH:" in *":$bin:"*) ;; *) printf 'Add %s to PATH (or run the full path above).\n' "$bin";; esac
printf 'Run ai-session-export and open http://127.0.0.1:8788\n'
