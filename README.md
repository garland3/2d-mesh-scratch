# 2D Geometry & FEA Mesh Generator

![Application Screenshot](imgs/Screenshot%202025-07-09%20170933.png)

A powerful web-based tool for generating high-quality 2D finite element meshes with multiple algorithms and interactive visualization.

## Features

- **Interactive Web Interface**: Click to add boundary points and visualize meshes in real-time
- **Multiple Mesh Algorithms**:
  - **Delaunay Triangulation**: Fast, robust triangulation for general geometries
  - **Paving (Quad-dominant)**: Structured grid-based approach for rectangular regions
  - **Simulated Annealing**: Advanced optimization for high-quality meshes
- **Quality Control**: Configurable area constraints and minimum angle requirements
- **Export Capabilities**: CSV export for use in FEA software
- **Comprehensive Logging**: Detailed telemetry and error tracking
- **Real-time Visualization**: Interactive canvas with grid and mesh display

## Quick Start

### Prerequisites

- Python 3.7+
- FastAPI and dependencies (see `requirements.txt`)
- Pre-compiled Rust binary (included) or Rust toolchain for development

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd special-funicular
   ```

2. **Run the setup script**:
   ```bash
   ./setup.sh
   ```

   For development with Rust:
   ```bash
   ./setup.sh --rust
   ```

3. **Start the server**:
   ```bash
   python main.py
   ```

4. **Open your browser** to `http://localhost:8000`

## Usage

### Web Interface

1. **Add Points**: Click on the canvas to add boundary points
2. **Configure Settings**:
   - **Algorithm**: Choose from Delaunay, Paving, or Simulated Annealing
   - **Max Area**: Control element size (smaller = finer mesh)
   - **Min Angle**: Set quality threshold (higher = better quality)
3. **Generate Mesh**: Click "Generate Mesh" to create the mesh
4. **Export**: Use "Export to CSV" to save mesh data

### Command Line Interface

The Rust binary supports multiple modes:

```bash
# Test with example data
./target/release/mesh-generator test

# Interactive mode
./target/release/mesh-generator interactive

# Process JSON from file
./target/release/mesh-generator json geometry.json

# Process JSON from stdin
echo '{"geometry":{"points":[...],"name":"test"},"max_area":0.1,"algorithm":"delaunay"}' | ./target/release/mesh-generator json-stdin
```

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

## API Reference

### Mesh Generation Endpoint

```
POST /generate-mesh
Content-Type: application/json

{
  "geometry": {
    "points": [
      {"x": 0, "y": 0},
      {"x": 1, "y": 0},
      {"x": 1, "y": 1},
      {"x": 0, "y": 1}
    ],
    "name": "rectangle"
  },
  "max_area": 0.1,
  "min_angle": 20.0,
  "algorithm": "delaunay"
}
```

### CSV Export Endpoint

```
POST /export-csv
Content-Type: application/json

{
  "points": [...],
  "name": "geometry"
}
```

## Quality Metrics

The mesh generator uses several quality metrics:

- **Minimum Angle**: Prevents thin, poorly-shaped triangles
- **Jacobian Determinant**: Ensures positive orientation and element validity
- **Area Constraints**: Controls element size distribution
- **Boundary Conformity**: Maintains geometric accuracy

## Logging and Monitoring

All operations are logged to the `log` file with detailed telemetry:

```
2025-07-09 23:09:18,913 - main - INFO - SESSION_START [d65f5002] - IP: 136.226.98.48, Method: POST, Path: /generate-mesh
2025-07-09 23:09:18,913 - main - INFO - MESH_GENERATION - Starting mesh generation with 4 points, algorithm: annealing
2025-07-09 23:09:18,927 - main - INFO - RUST_OUTPUT - ANNEALING - Starting simulated annealing mesh generation
2025-07-09 23:09:18,927 - main - INFO - RUST_OUTPUT - ANNEALING - Refined boundary to 82 points
2025-07-09 23:09:18,927 - main - INFO - RUST_OUTPUT - ANNEALING - Generated internal grid with 1704 points
```

## Development

### Project Structure

```
special-funicular/
├── main.py              # FastAPI web server
├── src/
│   ├── lib.rs           # Rust mesh generation library
│   └── main.rs          # Rust CLI binary
├── target/release/      # Compiled Rust binary
├── static/              # Static web assets
├── requirements.txt     # Python dependencies
├── Cargo.toml          # Rust dependencies
├── setup.sh            # Development setup script
└── README.md           # This file
```

### Building from Source

If you need to rebuild the Rust binary:

```bash
cargo build --release
```

### Running Tests

```bash
# Test fast build
./fast_test.sh

# Test specific algorithm
echo '{"geometry":{"points":[...],"max_area":0.1,"algorithm":"annealing"}' | ./target/release/mesh-generator json-stdin
```

## Configuration

### Environment Variables

- `RUST_LOG`: Set logging level (default: info)
- `PORT`: Web server port (default: 8000)

### Quality Thresholds

- **Delaunay**: Optimized for speed and robustness
- **Paving**: Balanced quality and structure
- **Annealing**: Customizable quality threshold via `min_angle` parameter

## Performance

### Typical Performance Metrics

- **Delaunay**: 1000+ triangles/second
- **Paving**: 500+ quads/second  
- **Annealing**: 10-100 triangles/second (depends on quality requirements)

### Optimization Tips

1. **Use appropriate max_area**: Smaller values = more elements = slower generation
2. **Choose algorithm wisely**: Delaunay for speed, Annealing for quality
3. **Set reasonable quality thresholds**: Higher min_angle = more optimization time
4. **Monitor logs**: Check `log` file for performance insights

## Troubleshooting

### Common Issues

1. **Port already in use**: Change port in `main.py` or kill existing process
2. **Rust binary not found**: Ensure `target/release/mesh-generator` exists
3. **Poor mesh quality**: Try Simulated Annealing with higher min_angle
4. **Slow generation**: Increase max_area or use Delaunay algorithm

### Debug Mode

Enable detailed logging:
```bash
RUST_LOG=debug python main.py
```

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
- Check the `log` file for error details
- Review this README for configuration options
- Submit issues via the project repository