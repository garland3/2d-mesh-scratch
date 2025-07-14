#!/bin/bash
set -e

echo "Building fluid simulator WASM module..."

# Build with wasm-pack
wasm-pack build --target web --out-dir pkg

echo "Fluid simulator WASM module built successfully!"
echo "Files generated in pkg/ directory"
echo "Open example.html in a web server to test"