#!/usr/bin/env bash
set -euo pipefail

VER="${1:?Usage: $0 vX.Y.Z}"

TARGETS=(aarch64-pc-windows-msvc aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu x86_64-pc-windows-msvc x86_64-apple-darwin aarch64-apple-darwin)

command -v cargo >/dev/null || { echo "cargo not found"; exit 1; }
command -v cbindgen >/dev/null || cargo install cbindgen
command -v zip >/dev/null || { echo "zip not found"; exit 1; }

mkdir -p include
cbindgen --config cbindgen.toml --crate . --output include/foxdbg.h

rm -rf dist && mkdir -p dist

for T in "${TARGETS[@]}"; do
  rustup target add "$T" >/dev/null
  cargo build --release --target "$T"

  OUT="foxdbg-${VER}-${T}"
  PKG_DIR="dist/${OUT}"
  mkdir -p "${PKG_DIR}/include" "${PKG_DIR}/lib" "${PKG_DIR}/cmake"

  cp include/foxdbg.h "${PKG_DIR}/include/"
  if [[ "$T" == *windows-msvc ]]; then
    cp "target/${T}/release/foxdbg.lib" "${PKG_DIR}/lib/"
  else
    cp "target/${T}/release/libfoxdbg.a" "${PKG_DIR}/lib/"
  fi
  [[ -f cmake/foxdbgConfig.cmake ]] && cp cmake/foxdbgConfig.cmake "${PKG_DIR}/cmake/"

  (cd dist && zip -r "${OUT}.zip" "${OUT}")
  echo "Packaged dist/${OUT}.zip"
done

echo "Done. Upload dist/*.zip to GitHub Release for ${VER}."