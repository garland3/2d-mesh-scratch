#!/bin/bash

# Fast test script for mesh generator
# Tests various scenarios to ensure unit consistency

echo "=== Fast Test Script for Mesh Generator ==="
echo

# Build the project
echo "Building Rust binary..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi
echo "Build successful!"
echo

# Test 1: Basic triangulation (no refinement)
echo "Test 1: Basic triangulation (3 points, no refinement)"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":50.0,"y":100.0}],"name":"test"},"max_area":null,"min_angle":null}' | ./target/release/mesh-generator json-stdin
echo

# Test 2: Delaunay algorithm with area refinement
echo "Test 2: Delaunay algorithm with area refinement"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":200.0,"y":0.0},{"x":200.0,"y":200.0},{"x":0.0,"y":200.0}],"name":"test"},"max_area":500.0,"min_angle":20.0,"algorithm":"delaunay"}' | timeout 5s ./target/release/mesh-generator json-stdin
if [ $? -eq 124 ]; then
    echo "Delaunay test timed out"
fi
echo

# Test 3: Paving algorithm (quad-dominant)
echo "Test 3: Paving algorithm (quad-dominant)"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"test"},"max_area":200.0,"min_angle":20.0,"algorithm":"paving"}' | timeout 5s ./target/release/mesh-generator json-stdin
if [ $? -eq 124 ]; then
    echo "Paving test timed out"
fi
echo

# Test 4: CSV export (Delaunay)
echo "Test 4: CSV export (Delaunay)"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":50.0,"y":100.0}],"name":"test"},"max_area":50.0,"min_angle":20.0,"algorithm":"delaunay"}' | timeout 5s ./target/release/mesh-generator csv-stdin
if [ $? -eq 124 ]; then
    echo "CSV export timed out"
fi
echo

# Test 5: Algorithm comparison
echo "Test 5: Algorithm comparison (triangle count)"
echo "Delaunay triangles:"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"square"},"max_area":300.0,"min_angle":20.0,"algorithm":"delaunay"}' | timeout 3s ./target/release/mesh-generator json-stdin | jq '.triangles | length'
echo "Paving quads:"
echo '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"square"},"max_area":300.0,"min_angle":20.0,"algorithm":"paving"}' | timeout 3s ./target/release/mesh-generator json-stdin | jq '.quads | length // 0'
if [ $? -eq 124 ]; then
    echo "Algorithm comparison timed out"
fi
echo

echo "=== Test Summary ==="
echo "If tests timed out, the mesh refinement algorithm needs optimization"
echo "Expected behavior:"
echo "- Test 1: 1 triangle (basic triangulation)"
echo "- Test 2: Multiple triangles (Delaunay with refinement)"
echo "- Test 3: Structured quads (Paving algorithm)"
echo "- Test 4: CSV output with mesh data"
echo "- Test 5: Algorithm comparison (triangles vs quads)"
echo
echo "Algorithm differences:"
echo "- Delaunay: Triangular mesh, good for irregular shapes"
echo "- Paving: Quad-dominant mesh, structured grid-like"
echo
echo "Current coordinate system:"
echo "- Canvas: 800x600 pixels"
echo "- World coordinates: 1 unit = 1 pixel"
echo "- Grid spacing: 50 units"
echo "- Max area: square units (e.g., 100 = 10x10 pixel triangle/quad)"