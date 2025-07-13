# 2D Geometry & FEA Mesh Generator

![Application Screenshot](imgs/Screenshot%202025-07-09%20170933.png)

A powerful mesh generation tool with both WebAssembly web interface and command-line capabilities for generating high-quality 2D finite element meshes.

## Features

- **Dual Interface**: Both web browser (WASM) and command-line tool
- **WebAssembly Performance**: Fast mesh generation running directly in the browser
- **Interactive Web Interface**: Click to add boundary points and visualize meshes in real-time
- **Command Line Tool**: Scriptable mesh generation for automation and batch processing
- **Multiple Mesh Algorithms**:
  - **Delaunay Triangulation**: Fast, robust triangulation for general geometries
  - **Paving (Quad-dominant)**: Structured grid-based approach for rectangular regions
  - **Simulated Annealing**: Advanced optimization for high-quality meshes
- **Quality Control**: Configurable refinement and smoothing algorithms
- **Export Capabilities**: JSON output for use in FEA software and analysis tools

## Quick Start

### Prerequisites

- Modern web browser with WebAssembly support (for web interface)
- Rust toolchain (for building CLI tool from source)

### Web Interface

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd special-funicular
   ```

2. **Open the web application**:
   ```bash
   open index.html
   ```
   Or simply double-click `index.html` in your file manager

### Command Line Tool

1. **Build the CLI tool**:
   ```bash
   cargo build --bin mesher --release
   ```

2. **Run the demo**:
   ```bash
   ./mesh_box.sh
   ```

3. **Use the CLI tool**:
   ```bash
   # Quick test
   ./target/release/mesher test
   
   # Process JSON input
   echo '{"geometry":{"points":[...]}, "density":0.1}' | ./target/release/mesher json-stdin
   ```

## How to Use

### Web Interface

1. **Add Points**: Click on the canvas to add boundary points
2. **Configure Settings**:
   - **Algorithm**: Choose from Delaunay, Paving, or Simulated Annealing
   - **Max Area**: Control element size (smaller = finer mesh)
   - **Min Angle**: Set quality threshold (higher = better quality)
3. **Generate Mesh**: Click "Generate Mesh" to create the mesh
4. **Export**: Use "Export" to download mesh data

### Command Line Interface

The mesh generator includes a powerful CLI tool for batch processing and automation.

#### Usage Modes

```bash
# Test with built-in example
./target/release/mesher test

# Process JSON from file
./target/release/mesher json geometry.json

# Process JSON from stdin
echo '{"geometry": {...}, "density": 0.1}' | ./target/release/mesher json-stdin

# Interactive mode
./target/release/mesher interactive
```

#### JSON Input Format

```json
{
  "geometry": {
    "points": [
      {"x": 0.0, "y": 0.0},
      {"x": 2.0, "y": 0.0},
      {"x": 2.0, "y": 1.0},
      {"x": 0.0, "y": 1.0}
    ],
    "name": "rectangle"
  },
  "density": 0.15,
  "refine_metric": "angle",
  "refine_threshold": 25.0,
  "refine_iterations": 50,
  "smooth_iterations": 3
}
```

#### Example: Box Mesh Generation

Run the included demo script:
```bash
./mesh_box.sh
```

This generates a mesh for a 2×1 rectangle with refinement and smoothing, outputting detailed statistics and saving the result to `box_mesh.json`.

### Development

The legacy Python/FastAPI version is available in the `test/` directory for reference and development.

## Mesh Algorithms

### Delaunay Triangulation
- **Best for**: General geometries, fast meshing
- **Features**: Guaranteed triangle quality, boundary refinement
- **Use case**: Standard triangulation for most applications

### Paving (Quad-dominant)
- **Best for**: Rectangular regions, structured meshes
- **Features**: Generates quadrilateral elements with triangular fill
- **Use case**: Structured analysis requiring quad elements

### Simulated Annealing
- **Best for**: High-quality meshes, complex geometries
- **Features**: Iterative optimization, configurable quality thresholds
- **Process**:
  1. Refines boundary points based on target area
  2. Generates internal grid of points
  3. Creates initial Delaunay triangulation
  4. Optimizes point positions using simulated annealing
  5. Stops when quality threshold is reached

## WebAssembly API

The mesh generation is handled by WebAssembly functions exported from the Rust library:

- `generate_mesh()`: Main mesh generation function
- `set_algorithm()`: Configure mesh algorithm
- `export_data()`: Export mesh data for download

## Quality Metrics

The mesh generator uses several quality metrics:

- **Minimum Angle**: Prevents thin, poorly-shaped triangles
- **Jacobian Determinant**: Ensures positive orientation and element validity
- **Area Constraints**: Controls element size distribution
- **Boundary Conformity**: Maintains geometric accuracy

## Browser Debugging

Check browser console for mesh generation output and any errors.

## Development

### Project Structure

```
special-funicular/
├── index.html           # Main web application
├── js_mesher.mthml     # Web interface code
├── mesh_box.sh         # CLI demo script
├── src/
│   ├── lib.rs          # WASM interface wrapper
│   ├── main.rs         # CLI tool main
│   ├── geometry.rs     # Point and Triangle geometry
│   └── mesher.rs       # Core mesh generation algorithms
├── pkg/                # Generated WebAssembly package
│   ├── rust_mesher.js  # WASM bindings
│   └── rust_mesher_bg.wasm # Compiled WebAssembly
├── target/
│   └── release/
│       └── mesher      # Compiled CLI binary
├── Cargo.toml          # Rust dependencies and binary config
├── test/               # Legacy Python/FastAPI version
└── README.md           # This file
```

### Building from Source

Build the WebAssembly module:
```bash
wasm-pack build --target web
```

Build the CLI tool:
```bash
cargo build --bin mesher --release
```

### Running Tests

```bash
# Test the CLI tool
./target/release/mesher test

# Test with demo script
./mesh_box.sh

# Test the WASM build
./fast_test.sh
```

## Performance

WebAssembly provides near-native performance for mesh generation directly in the browser.

### Optimization Tips

1. **Use appropriate max_area**: Smaller values = more elements = slower generation
2. **Choose algorithm wisely**: Delaunay for speed, Annealing for quality
3. **Set reasonable quality thresholds**: Higher min_angle = more optimization time

## Troubleshooting

### Common Issues

1. **WebAssembly not loading**: Ensure you're serving from a web server (not file://)
2. **Browser compatibility**: Use a modern browser with WASM support
3. **Poor mesh quality**: Try Simulated Annealing with higher min_angle
4. **Slow generation**: Increase max_area or use Delaunay algorithm

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is open source. See LICENSE file for details.

## Support

For issues and questions:
- Check browser console for error details
- Review this README for configuration options
- Submit issues via the project repository