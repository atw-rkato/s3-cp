#!/bin/sh
set -eu

APP_NAME=s3-cp
RUST_VERSION=1.59.0
BUILD_TARGET=x86_64-apple-darwin
RUST_TOOLCHAIN="${RUST_VERSION}-${BUILD_TARGET}"

EXIST_CARGO=false
EXIST_TOOLCHAIN=false
if type "cargo" > /dev/null 2>&1; then
  EXIST_CARGO=true
  if rustup toolchain list | grep -q "${RUST_TOOLCHAIN}"; then
    EXIST_TOOLCHAIN=true
  else
    rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal
  fi
else
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain "${RUST_VERSION}" --profile minimal -y
fi

cargo build --release -j 12 --target="${BUILD_TARGET}"
mkdir -p ./dist
cp  "./target/${BUILD_TARGET}/release/${APP_NAME}" "./dist/${APP_NAME}"

if ! "${EXIST_TOOLCHAIN}"; then
  rustup toolchain uninstall "${RUST_TOOLCHAIN}"
fi

if ! "${EXIST_CARGO}"; then
  rustup self uninstall -y
fi
