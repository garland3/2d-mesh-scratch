#!/bin/bash

# Mesh Box Demo Script
# Demonstrates the CLI mesher tool with a simple box geometry

echo "=== Mesh Box Demo ==="
echo "This script demonstrates mesh generation for a 2x1 rectangular box"
echo

# Build the CLI tool if it doesn't exist
if [ ! -f "./target/debug/mesher" ]; then
    echo "Building CLI tool..."
    cargo build --bin mesher
    echo
fi

# Define the box geometry as JSON
# A 2x1 rectangle with corners at (0,0), (2,0), (2,1), (0,1)
BOX_JSON='{
  "geometry": {
    "points": [
      {"x": 0.0, "y": 0.0},
      {"x": 2.0, "y": 0.0},
      {"x": 2.0, "y": 1.0},
      {"x": 0.0, "y": 1.0}
    ],
    "name": "box_2x1"
  },
  "density": 0.15,
  "refine_metric": "angle",
  "refine_threshold": 25.0,
  "refine_iterations": 50,
  "smooth_iterations": 3
}'

echo "Input geometry: 2x1 rectangle"
echo "Mesh density: 0.15"
echo "Refinement: angle quality > 25°"
echo "Smoothing: 3 iterations"
echo

echo "Generating mesh..."
echo "$BOX_JSON" | ./target/debug/mesher json-stdin > box_mesh.json

if [ $? -eq 0 ]; then
    echo "✓ Mesh generated successfully!"
    echo
    
    # Extract and display statistics
    echo "=== Mesh Statistics ==="
    echo -n "Points: "
    jq -r '.stats.point_count' box_mesh.json 2>/dev/null || echo "N/A"
    
    echo -n "Triangles: "
    jq -r '.stats.triangle_count' box_mesh.json 2>/dev/null || echo "N/A"
    
    echo -n "Average minimum angle: "
    jq -r '(.stats.avg_angle_quality | tostring) + "°"' box_mesh.json 2>/dev/null || echo "N/A"
    
    echo -n "Average aspect ratio: "
    jq -r '.stats.avg_aspect_quality' box_mesh.json 2>/dev/null || echo "N/A"
    
    echo
    echo "Full mesh data saved to: box_mesh.json"
    echo
    
    # Show first few points as preview
    echo "=== Preview (first 5 points) ==="
    jq -r '.points[:5] | to_entries[] | "Point \(.key): x=\(.value.x), y=\(.value.y)"' box_mesh.json 2>/dev/null || echo "jq not available for preview"
    
else
    echo "✗ Failed to generate mesh"
    exit 1
fi

echo
echo "Demo complete! 🎉"
echo "Try opening box_mesh.json to see the full mesh data."