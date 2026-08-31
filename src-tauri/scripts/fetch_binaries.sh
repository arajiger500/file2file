#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT_DIR/binaries"
mkdir -p "$BIN_DIR"

echo "Downloading ffmpeg (static)..."
FF_URL="https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
tmpdir=$(mktemp -d)
pushd "$tmpdir" >/dev/null
curl -L -o ff.tar.xz "$FF_URL"
tar -xf ff.tar.xz
ffdir=$(find . -maxdepth 1 -type d -name 'ffmpeg-*' | head -n1)
if [ -z "$ffdir" ]; then
  echo "ffmpeg dir not found in archive" >&2
  exit 1
fi
cp "$ffdir/ffmpeg" "$BIN_DIR/ffmpeg-x86_64-unknown-linux-gnu"
cp "$ffdir/ffprobe" "$BIN_DIR/ffprobe-x86_64-unknown-linux-gnu"
chmod +x "$BIN_DIR/ffmpeg-x86_64-unknown-linux-gnu" "$BIN_DIR/ffprobe-x86_64-unknown-linux-gnu"
popd >/dev/null
rm -rf "$tmpdir"

echo "Downloading pandoc (linux amd64)..."
PANDOC_VER="3.1.8"
PANDOC_URL="https://github.com/jgm/pandoc/releases/download/${PANDOC_VER}/pandoc-${PANDOC_VER}-linux-amd64.tar.gz"
tmpdir=$(mktemp -d)
pushd "$tmpdir" >/dev/null
curl -L -o pandoc.tar.gz "$PANDOC_URL"
tar -xzf pandoc.tar.gz
cp pandoc-*/bin/pandoc "$BIN_DIR/pandoc-x86_64-unknown-linux-gnu"
chmod +x "$BIN_DIR/pandoc-x86_64-unknown-linux-gnu"
popd >/dev/null
rm -rf "$tmpdir"

echo "Done. Binaries placed in $BIN_DIR"
echo "Note: For PDF->DOCX reliably you must bundle LibreOffice (soffice). For image conversions you may need ImageMagick or poppler (pdftoppm)."
