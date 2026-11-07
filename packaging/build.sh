#!/usr/bin/env bash
# Build BrambleRS and the three Linux packages into dist/.
#
#   packaging/build.sh            # .deb + .rpm + AppImage
#   packaging/build.sh deb rpm    # a subset
#
# One package, one binary: the desktop app and the command line are both `bramblers`.
# Needs: cargo, cargo-deb, cargo-generate-rpm (cargo install cargo-deb cargo-generate-rpm),
# and for the AppImage network access once: a pinned appimagetool is downloaded into
# packaging/.tools/ and checked against the SHA256 published in its release. Nothing here
# needs root, and nothing is installed.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')"
ARCH="$(uname -m)"
DIST="$ROOT/dist"
TOOLS="$ROOT/packaging/.tools"
mkdir -p "$DIST" "$TOOLS"

targets=("$@")
[ ${#targets[@]} -eq 0 ] && targets=(deb rpm appimage)

echo "== bramblers $VERSION ($ARCH): building the release binary"
cargo build --release -p bramblers --bin bramblers
cargo test --release --workspace --quiet

for t in "${targets[@]}"; do
  case "$t" in
    deb)
      echo "== .deb"
      cargo deb -p bramblers --no-build --quiet
      cp target/debian/bramblers_"$VERSION"-*.deb "$DIST/"
      ;;
    rpm)
      echo "== .rpm"
      cargo generate-rpm -p crates/bramblers
      cp target/generate-rpm/bramblers-"$VERSION"-*.rpm "$DIST/"
      ;;
    appimage)
      echo "== AppImage"
      # Pinned release, verified against the SHA256 published in the GitHub release.
      # `continuous` changes without notice; a downloaded tool is never executed unverified.
      APPIMAGETOOL_VERSION="1.9.1"
      case "$ARCH" in
        x86_64)
          TOOL_ARCH="x86_64"
          TOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
          ;;
        aarch64|arm64)
          TOOL_ARCH="aarch64"
          TOOL_SHA256="f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158"
          ;;
        armv7l|armhf)
          TOOL_ARCH="armhf"
          TOOL_SHA256="42b61cba5495d8aaf418a5c9a015a49b85ad92efabcbd3c341f1540440e4e23d"
          ;;
        i686|i386)
          TOOL_ARCH="i686"
          TOOL_SHA256="7ad9ff47c203aae0149b18f6df9e3018b2e2f470ea644a0413e3ded39e9e3bdb"
          ;;
        *)
          echo "unsupported architecture for appimagetool: $ARCH" >&2
          exit 2
          ;;
      esac
      TOOL="$TOOLS/appimagetool-$TOOL_ARCH"
      verify_tool() {
        ( cd "$TOOLS" && printf '%s  %s\n' "$TOOL_SHA256" "appimagetool-$TOOL_ARCH" | sha256sum -c - >/dev/null )
      }
      if [ ! -x "$TOOL" ] || ! verify_tool; then
        rm -f "$TOOL"
        curl -sSL -o "$TOOL" \
          "https://github.com/AppImage/appimagetool/releases/download/$APPIMAGETOOL_VERSION/appimagetool-$TOOL_ARCH.AppImage"
        chmod +x "$TOOL"
        verify_tool
      fi
      APPDIR="$ROOT/target/appimage/bramblers.AppDir"
      rm -rf "$APPDIR"
      mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
        "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$APPDIR/usr/share/icons/hicolor/scalable/apps"
      cp target/release/bramblers "$APPDIR/usr/bin/"
      cp assets/bramblers.desktop "$APPDIR/usr/share/applications/"
      cp assets/bramblers.desktop "$APPDIR/bramblers.desktop"
      cp assets/icon-256.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/bramblers.png"
      cp assets/icon.svg "$APPDIR/usr/share/icons/hicolor/scalable/apps/bramblers.svg"
      cp assets/icon-256.png "$APPDIR/bramblers.png"
      cp assets/icon-256.png "$APPDIR/.DirIcon"
      ln -sf usr/bin/bramblers "$APPDIR/AppRun"
      # appimagetool is itself an AppImage; extract-and-run avoids needing FUSE.
      APPIMAGE_EXTRACT_AND_RUN=1 ARCH="$ARCH" "$TOOL" --no-appstream \
        "$APPDIR" "$DIST/bramblers-$VERSION-$ARCH.AppImage" 2>&1 | grep -v '^$' | tail -3
      chmod +x "$DIST/bramblers-$VERSION-$ARCH.AppImage"
      ;;
    *)
      echo "unknown target: $t (deb, rpm, appimage)" >&2
      exit 2
      ;;
  esac
done

echo
echo "== dist/"
rm -f "$DIST/SHA256SUMS"
ls -la "$DIST"
# Only this version's packages: older ones may still sit in dist/.
(cd "$DIST" && sha256sum ./*"$VERSION"*.deb ./*"$VERSION"*.rpm ./*"$VERSION"*.AppImage 2>/dev/null) | tee "$DIST/SHA256SUMS"
