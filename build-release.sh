#!/bin/bash
set -e

TARGET="x86_64-pc-windows-gnu"
EXE="target/$TARGET/release/caffeinate.exe"
CERT="certs/caffeinate.pfx"

echo "Building release..."
cargo build --target "$TARGET" --release

if [ -f "$CERT" ]; then
    echo "Signing..."
    UNSIGNED="$EXE.unsigned"
    mv "$EXE" "$UNSIGNED"
    osslsigncode sign -pkcs12 "$CERT" -n "Caffeinate" \
        -t http://timestamp.digicert.com \
        -in "$UNSIGNED" -out "$EXE"
    rm "$UNSIGNED"
    echo "Signed: $EXE"
else
    echo "No certificate found at $CERT — skipping signing"
    echo "Built: $EXE"
fi

ls -lh "$EXE"
