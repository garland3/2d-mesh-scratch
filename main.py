from fastapi import FastAPI, HTTPException, Request
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from pydantic import BaseModel
from typing import List, Optional
import csv
import io
import os
import json
import subprocess
from datetime import datetime
import logging
import time
import uuid

app = FastAPI(title="2D Geometry & FEA Mesh Generator")

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Add middleware to log HTTP sessions
@app.middleware("http")
async def log_requests(request: Request, call_next):
    # Generate session ID
    session_id = str(uuid.uuid4())[:8]
    
    # Log basic request info
    start_time = time.time()
    client_ip = request.client.host if request.client else "unknown"
    user_agent = request.headers.get("user-agent", "unknown")
    
    logger.info(f"SESSION_START [{session_id}] - IP: {client_ip}, Method: {request.method}, Path: {request.url.path}, User-Agent: {user_agent}")
    
    # Process request
    try:
        response = await call_next(request)
        processing_time = time.time() - start_time
        
        # Log successful response
        logger.info(f"SESSION_END [{session_id}] - Status: {response.status_code}, Time: {processing_time:.3f}s, Size: {response.headers.get('content-length', 'unknown')}bytes")
        
        return response
    except Exception as e:
        processing_time = time.time() - start_time
        logger.error(f"SESSION_ERROR [{session_id}] - Error: {str(e)}, Time: {processing_time:.3f}s")
        raise

# Data models
class Point(BaseModel):
    x: float
    y: float
    id: Optional[str] = None

class Geometry(BaseModel):
    points: List[Point]
    name: Optional[str] = "geometry"

class AnnealingOptions(BaseModel):
    temperature: Optional[float] = 1000.0
    cooling_rate: Optional[float] = 0.995
    quality_threshold: Optional[float] = 0.8
    max_iterations: Optional[int] = 10000

class MeshRequest(BaseModel):
    geometry: Geometry
    max_area: Optional[float] = 0.1
    min_angle: Optional[float] = 20.0
    algorithm: Optional[str] = "delaunay"
    annealing_options: Optional[AnnealingOptions] = None

# Global storage (in production, use a database)
geometries = {}
meshes = {}

# Serve static files
if not os.path.exists("static"):
    os.makedirs("static")

app.mount("/static", StaticFiles(directory="static"), name="static")

@app.get("/", response_class=HTMLResponse)
async def read_root():
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>2D Geometry & FEA Mesh Generator</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            #canvas { border: 1px solid #ccc; cursor: crosshair; margin: 30px; }
            .controls { margin: 20px 0; }
            button { margin: 5px; padding: 10px 20px; }
            input { margin: 5px; padding: 5px; }
            .point-list { margin: 20px 0; }
            .point-item { margin: 5px 0; padding: 5px; background: #f0f0f0; }
        </style>
    </head>
    <body>
        <h1>2D Geometry & FEA Mesh Generator</h1>
        
        <div class="controls">
            <button onclick="clearPoints()">Clear Points</button>
            <button onclick="generateMesh()">Generate Mesh</button>
            <button onclick="clearMesh()">Clear Mesh</button>
            <button onclick="exportCSV()">Export to CSV</button>
            <br>
            Algorithm: 
            <select id="meshAlgorithm">
                <option value="delaunay">Delaunay Triangulation</option>
                <option value="paving">Paving (Quad-dominant)</option>
                <option value="annealing">Simulated Annealing</option>
            </select>
            <br>
            Max Area: <input type="number" id="maxArea" value="100" step="10"> (square units)
            Min Angle: <input type="number" id="minAngle" value="20" step="1"> (degrees)
            <br>
            <div id="annealingOptions" style="display: none; margin-top: 10px; padding: 10px; border: 1px solid #ccc; background: #f9f9f9;">
                <strong>Annealing Options:</strong><br>
                Initial Temperature: <input type="number" id="temperature" value="1000" step="100" min="1"> 
                Cooling Rate: <input type="number" id="coolingRate" value="0.995" step="0.001" min="0.001" max="0.999">
                <br>
                Quality Threshold: <input type="number" id="qualityThreshold" value="0.8" step="0.1" min="0.1" max="1.0">
                Max Iterations: <input type="number" id="maxIterations" value="10000" step="1000" min="100">
            </div>
        </div>
        
        <canvas id="canvas" width="800" height="600"></canvas>
        
        <div class="point-list">
            <h3>Points:</h3>
            <div id="pointsList"></div>
        </div>
        
        <div id="meshInfo"></div>
        
        <script>
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');
            let points = [];
            let mesh = null;
            
            // Coordinate system: 1 unit = 1 pixel, but with origin at bottom-left
            // Canvas is 800x600, so world coordinates go from (0,0) to (800,600)
            // This makes the max area setting meaningful (e.g., 100 = 10x10 pixel triangle)
            
            // Convert canvas coordinates to world coordinates
            function canvasToWorld(canvasX, canvasY) {
                return {
                    x: canvasX,
                    y: canvas.height - canvasY  // Flip Y axis
                };
            }
            
            // Convert world coordinates to canvas coordinates
            function worldToCanvas(worldX, worldY) {
                return {
                    x: worldX,
                    y: canvas.height - worldY  // Flip Y axis
                };
            }
            
            canvas.addEventListener('click', addPoint);
            
            // Show/hide annealing options based on algorithm selection
            document.getElementById('meshAlgorithm').addEventListener('change', function() {
                const annealingOptions = document.getElementById('annealingOptions');
                if (this.value === 'annealing') {
                    annealingOptions.style.display = 'block';
                } else {
                    annealingOptions.style.display = 'none';
                }
            });
            
            function addPoint(event) {
                const rect = canvas.getBoundingClientRect();
                const canvasX = event.clientX - rect.left;
                const canvasY = event.clientY - rect.top;
                
                // Convert to world coordinates
                const worldCoords = canvasToWorld(canvasX, canvasY);
                
                const point = {
                    x: worldCoords.x,
                    y: worldCoords.y,
                    id: Date.now().toString()
                };
                
                points.push(point);
                updatePointsList();
                drawPoints();
            }
            
            function drawGrid() {
                // Draw grid lines
                ctx.strokeStyle = '#e0e0e0';
                ctx.lineWidth = 0.5;
                
                const gridSize = 50; // Grid spacing in world units (50 units = 50 pixels)
                
                // Vertical grid lines
                for (let worldX = 0; worldX <= canvas.width; worldX += gridSize) {
                    const canvasCoords = worldToCanvas(worldX, 0);
                    ctx.beginPath();
                    ctx.moveTo(canvasCoords.x, 0);
                    ctx.lineTo(canvasCoords.x, canvas.height);
                    ctx.stroke();
                }
                
                // Horizontal grid lines
                for (let worldY = 0; worldY <= canvas.height; worldY += gridSize) {
                    const canvasCoords = worldToCanvas(0, worldY);
                    ctx.beginPath();
                    ctx.moveTo(0, canvasCoords.y);
                    ctx.lineTo(canvas.width, canvasCoords.y);
                    ctx.stroke();
                }
                
                // Draw scale marks and labels
                ctx.fillStyle = '#666';
                ctx.font = '12px Arial';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'top';
                
                // X-axis scale marks (bottom)
                for (let worldX = 0; worldX <= canvas.width; worldX += gridSize) {
                    const canvasCoords = worldToCanvas(worldX, 0);
                    ctx.fillText(worldX.toFixed(0), canvasCoords.x, canvas.height - 15);
                    
                    // Draw tick marks
                    ctx.strokeStyle = '#333';
                    ctx.lineWidth = 1;
                    ctx.beginPath();
                    ctx.moveTo(canvasCoords.x, canvas.height - 5);
                    ctx.lineTo(canvasCoords.x, canvas.height);
                    ctx.stroke();
                }
                
                // Y-axis scale marks (left side)
                ctx.textAlign = 'right';
                ctx.textBaseline = 'middle';
                for (let worldY = 0; worldY <= canvas.height; worldY += gridSize) {
                    const canvasCoords = worldToCanvas(0, worldY);
                    ctx.fillText(worldY.toFixed(0), 25, canvasCoords.y);
                    
                    // Draw tick marks
                    ctx.strokeStyle = '#333';
                    ctx.lineWidth = 1;
                    ctx.beginPath();
                    ctx.moveTo(0, canvasCoords.y);
                    ctx.lineTo(5, canvasCoords.y);
                    ctx.stroke();
                }
                
                // Draw axes
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 2;
                
                // X-axis (at world Y=0)
                const xAxisY = worldToCanvas(0, 0).y;
                ctx.beginPath();
                ctx.moveTo(0, xAxisY);
                ctx.lineTo(canvas.width, xAxisY);
                ctx.stroke();
                
                // Y-axis (at world X=0)
                const yAxisX = worldToCanvas(0, 0).x;
                ctx.beginPath();
                ctx.moveTo(yAxisX, 0);
                ctx.lineTo(yAxisX, canvas.height);
                ctx.stroke();
            }
            
            function drawPoints() {
                ctx.clearRect(0, 0, canvas.width, canvas.height);
                
                // Draw grid first
                drawGrid();
                
                // Draw points
                ctx.fillStyle = 'red';
                points.forEach(point => {
                    const canvasCoords = worldToCanvas(point.x, point.y);
                    ctx.beginPath();
                    ctx.arc(canvasCoords.x, canvasCoords.y, 5, 0, 2 * Math.PI);
                    ctx.fill();
                });
                
                // Draw lines between consecutive points
                if (points.length > 1) {
                    ctx.strokeStyle = 'blue';
                    ctx.lineWidth = 2;
                    ctx.beginPath();
                    const firstCanvas = worldToCanvas(points[0].x, points[0].y);
                    ctx.moveTo(firstCanvas.x, firstCanvas.y);
                    for (let i = 1; i < points.length; i++) {
                        const canvasCoords = worldToCanvas(points[i].x, points[i].y);
                        ctx.lineTo(canvasCoords.x, canvasCoords.y);
                    }
                    // Close the polygon if we have more than 2 points
                    if (points.length > 2) {
                        ctx.lineTo(firstCanvas.x, firstCanvas.y);
                    }
                    ctx.stroke();
                }
                
                // Draw mesh if available
                if (mesh) {
                    drawMesh();
                }
            }
            
            function drawMesh() {
                // Draw triangles
                if (mesh.triangles && mesh.triangles.length > 0) {
                    ctx.strokeStyle = 'green';
                    ctx.lineWidth = 1;
                    
                    mesh.triangles.forEach(triangle => {
                        ctx.beginPath();
                        const p1 = worldToCanvas(triangle[0].x, triangle[0].y);
                        const p2 = worldToCanvas(triangle[1].x, triangle[1].y);
                        const p3 = worldToCanvas(triangle[2].x, triangle[2].y);
                        ctx.moveTo(p1.x, p1.y);
                        ctx.lineTo(p2.x, p2.y);
                        ctx.lineTo(p3.x, p3.y);
                        ctx.closePath();
                        ctx.stroke();
                    });
                }
                
                // Draw quads
                if (mesh.quads && mesh.quads.length > 0) {
                    ctx.strokeStyle = 'purple';
                    ctx.lineWidth = 1;
                    
                    mesh.quads.forEach(quad => {
                        ctx.beginPath();
                        const p1 = worldToCanvas(quad[0].x, quad[0].y);
                        const p2 = worldToCanvas(quad[1].x, quad[1].y);
                        const p3 = worldToCanvas(quad[2].x, quad[2].y);
                        const p4 = worldToCanvas(quad[3].x, quad[3].y);
                        ctx.moveTo(p1.x, p1.y);
                        ctx.lineTo(p2.x, p2.y);
                        ctx.lineTo(p3.x, p3.y);
                        ctx.lineTo(p4.x, p4.y);
                        ctx.closePath();
                        ctx.stroke();
                    });
                }
            }
            
            function updatePointsList() {
                const pointsList = document.getElementById('pointsList');
                pointsList.innerHTML = '';
                
                points.forEach((point, index) => {
                    const div = document.createElement('div');
                    div.className = 'point-item';
                    div.innerHTML = `
                        Point ${index + 1}: (${point.x.toFixed(1)}, ${point.y.toFixed(1)})
                        <button onclick="removePoint(${index})">Remove</button>
                    `;
                    pointsList.appendChild(div);
                });
            }
            
            function removePoint(index) {
                points.splice(index, 1);
                updatePointsList();
                drawPoints();
            }
            
            function clearPoints() {
                points = [];
                mesh = null;
                updatePointsList();
                drawPoints();
                document.getElementById('meshInfo').innerHTML = '';
            }
            
            function clearMesh() {
                mesh = null;
                drawPoints(); // Redraw to remove mesh but keep points
                document.getElementById('meshInfo').innerHTML = '';
            }
            
            async function generateMesh() {
                if (points.length < 3) {
                    alert('Need at least 3 points to generate a mesh');
                    return;
                }
                
                const maxArea = parseFloat(document.getElementById('maxArea').value);
                const minAngle = parseFloat(document.getElementById('minAngle').value);
                const algorithm = document.getElementById('meshAlgorithm').value;
                
                // Convert max area from UI units to mesh units
                // UI uses square units where 1 unit = 1 pixel
                // Max area of 100 square units = reasonable triangle size
                const meshMaxArea = maxArea;
                
                console.log('Starting mesh generation...');
                console.log('Points:', points);
                console.log('Max Area (UI):', maxArea);
                console.log('Max Area (mesh):', meshMaxArea);
                console.log('Min Angle:', minAngle);
                console.log('Algorithm:', algorithm);
                
                const requestData = {
                    geometry: {
                        points: points,
                        name: 'interactive_geometry'
                    },
                    max_area: meshMaxArea,
                    min_angle: minAngle,
                    algorithm: algorithm
                };
                
                // Add annealing-specific options if annealing algorithm is selected
                if (algorithm === 'annealing') {
                    requestData.annealing_options = {
                        temperature: parseFloat(document.getElementById('temperature').value),
                        cooling_rate: parseFloat(document.getElementById('coolingRate').value),
                        quality_threshold: parseFloat(document.getElementById('qualityThreshold').value),
                        max_iterations: parseInt(document.getElementById('maxIterations').value)
                    };
                }
                
                console.log('Request data:', JSON.stringify(requestData, null, 2));
                
                try {
                    const response = await fetch('/generate-mesh', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify(requestData)
                    });
                    
                    console.log('Response status:', response.status);
                    console.log('Response headers:', response.headers);
                    
                    if (response.ok) {
                        mesh = await response.json();
                        console.log('Received mesh:', mesh);
                        console.log('Mesh triangles count:', mesh.triangles.length);
                        console.log('Mesh vertices count:', mesh.vertices.length);
                        
                        drawPoints();
                        
                        let meshInfoHtml = `
                            <h3>Mesh Info:</h3>
                            <p>Vertices: ${mesh.vertices.length}</p>
                            <p>Triangles: ${mesh.triangles.length}</p>
                        `;
                        
                        if (mesh.quads && mesh.quads.length > 0) {
                            meshInfoHtml += `<p>Quads: ${mesh.quads.length}</p>`;
                        }
                        
                        document.getElementById('meshInfo').innerHTML = meshInfoHtml;
                    } else {
                        const errorText = await response.text();
                        console.error('Error response:', errorText);
                        alert('Error generating mesh: ' + errorText);
                    }
                } catch (error) {
                    console.error('Network error:', error);
                    alert('Network error: ' + error.message);
                }
            }
            
            async function exportCSV() {
                if (!mesh) {
                    alert('Generate a mesh first');
                    return;
                }
                
                console.log('Starting CSV export...');
                console.log('Current mesh:', mesh);
                console.log('Current points:', points);
                
                const exportData = {
                    geometry: {
                        points: points,
                        name: 'interactive_geometry'
                    }
                };
                
                console.log('Export data:', JSON.stringify(exportData, null, 2));
                
                try {
                    const response = await fetch('/export-csv', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify(exportData)
                    });
                    
                    console.log('Export response status:', response.status);
                    
                    if (response.ok) {
                        const blob = await response.blob();
                        console.log('CSV blob size:', blob.size);
                        
                        const url = window.URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'mesh_data.csv';
                        document.body.appendChild(a);
                        a.click();
                        window.URL.revokeObjectURL(url);
                        document.body.removeChild(a);
                        
                        console.log('CSV download initiated');
                    } else {
                        const errorText = await response.text();
                        console.error('CSV export error:', errorText);
                        alert('Error exporting CSV: ' + errorText);
                    }
                } catch (error) {
                    console.error('CSV export network error:', error);
                    alert('Error: ' + error.message);
                }
            }
        </script>
    </body>
    </html>
    """

@app.post("/geometry")
async def create_geometry(geometry: Geometry):
    geometry_id = datetime.now().isoformat()
    geometries[geometry_id] = geometry
    return {"id": geometry_id, "geometry": geometry}

@app.get("/geometry/{geometry_id}")
async def get_geometry(geometry_id: str):
    if geometry_id not in geometries:
        raise HTTPException(status_code=404, detail="Geometry not found")
    return geometries[geometry_id]

@app.post("/generate-mesh")
async def generate_mesh(request: MeshRequest):
    if len(request.geometry.points) < 3:
        logger.warning(f"MESH_GENERATION - Insufficient points: {len(request.geometry.points)}")
        raise HTTPException(status_code=400, detail="Need at least 3 points to generate mesh")
    
    start_time = time.time()
    logger.info(f"MESH_GENERATION - Starting mesh generation with {len(request.geometry.points)} points, algorithm: {request.algorithm}")
    
    try:
        # Prepare input for Rust binary
        rust_input = {
            "geometry": {
                "points": [{"x": p.x, "y": p.y, "id": p.id} for p in request.geometry.points],
                "name": request.geometry.name
            },
            "max_area": request.max_area,
            "min_angle": request.min_angle,
            "algorithm": request.algorithm
        }
        
        # Add annealing options if provided
        if request.annealing_options:
            rust_input["annealing_options"] = {
                "temperature": request.annealing_options.temperature,
                "cooling_rate": request.annealing_options.cooling_rate,
                "quality_threshold": request.annealing_options.quality_threshold,
                "max_iterations": request.annealing_options.max_iterations
            }
        
        logger.info(f"MESH_GENERATION - Calling Rust binary with {len(rust_input['geometry']['points'])} points")
        
        # Call Rust binary
        process = subprocess.Popen(
            ["./target/release/mesh-generator", "json-stdin"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        stdout, stderr = process.communicate(input=json.dumps(rust_input))
        
        # Log Rust binary output to our log file
        if stderr:
            for line in stderr.strip().split('\n'):
                if line.strip():
                    logger.info(f"RUST_OUTPUT - {line}")
        
        if process.returncode != 0:
            logger.error(f"MESH_GENERATION - Rust binary error: {stderr}")
            raise Exception(f"Rust binary failed: {stderr}")
        
        # Debug: Log stdout details
        logger.info(f"MESH_GENERATION - Stdout length: {len(stdout)}")
        logger.info(f"MESH_GENERATION - Stdout content (first 200 chars): {stdout[:200]}")
        
        # Parse output from Rust binary
        if not stdout.strip():
            logger.error(f"MESH_GENERATION - Empty stdout from Rust binary")
            raise Exception("Rust binary produced empty stdout")
        
        try:
            mesh_output = json.loads(stdout)
        except json.JSONDecodeError as e:
            logger.error(f"MESH_GENERATION - JSON decode error: {e}")
            logger.error(f"MESH_GENERATION - Stdout was: {stdout}")
            raise Exception(f"Failed to parse JSON from Rust binary: {e}")
        logger.info(f"MESH_GENERATION - Rust binary completed successfully")
        
        # Convert to expected format (already in the right format from Rust)
        mesh_result = {
            'vertices': mesh_output['vertices'],
            'triangles': mesh_output['triangles'],
            'triangle_indices': mesh_output['triangle_indices']
        }
        
        # Add quads if they exist (for paving algorithm)
        if 'quads' in mesh_output and mesh_output['quads']:
            mesh_result['quads'] = mesh_output['quads']
        if 'quad_indices' in mesh_output and mesh_output['quad_indices']:
            mesh_result['quad_indices'] = mesh_output['quad_indices']
        
        # Store mesh
        mesh_id = datetime.now().isoformat()
        meshes[mesh_id] = mesh_result
        
        processing_time = time.time() - start_time
        logger.info(f"MESH_GENERATION - Successfully generated mesh with {len(mesh_result['triangles'])} triangles, {len(mesh_result['vertices'])} vertices in {processing_time:.3f}s")
        
        return mesh_result
        
    except Exception as e:
        processing_time = time.time() - start_time
        logger.error(f"MESH_GENERATION - Error after {processing_time:.3f}s: {str(e)}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=f"Mesh generation failed: {str(e)}")

@app.post("/export-csv")
async def export_csv(request: Geometry):
    start_time = time.time()
    logger.info(f"CSV_EXPORT - Starting CSV export with {len(request.points)} points")
    
    try:
        # Prepare input for Rust binary (need to create a dummy mesh request)
        rust_input = {
            "geometry": {
                "points": [{"x": p.x, "y": p.y, "id": p.id} for p in request.points],
                "name": request.name
            },
            "max_area": 0.1,  # Default values for CSV generation
            "min_angle": 20.0
        }
        
        logger.info(f"CSV_EXPORT - Calling Rust binary for CSV export")
        
        # Call Rust binary for CSV export
        process = subprocess.Popen(
            ["./target/release/mesh-generator", "csv-stdin"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        stdout, stderr = process.communicate(input=json.dumps(rust_input))
        
        # Log Rust binary output to our log file
        if stderr:
            for line in stderr.strip().split('\n'):
                if line.strip():
                    logger.info(f"RUST_OUTPUT - {line}")
        
        if process.returncode != 0:
            logger.error(f"CSV_EXPORT - Rust binary error: {stderr}")
            raise Exception(f"Rust binary failed: {stderr}")
        
        csv_content = stdout
        logger.info(f"CSV_EXPORT - Generated CSV content with {len(csv_content)} characters")
        
        # Create a temporary file for the CSV
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.csv') as tmp_file:
            tmp_file.write(csv_content)
            tmp_file_path = tmp_file.name
        
        processing_time = time.time() - start_time
        logger.info(f"CSV_EXPORT - Successfully created CSV file at {tmp_file_path} in {processing_time:.3f}s")
        
        return FileResponse(
            path=tmp_file_path,
            filename="mesh_data.csv",
            media_type="text/csv"
        )
    
    except Exception as e:
        processing_time = time.time() - start_time
        logger.error(f"CSV_EXPORT - Error after {processing_time:.3f}s: {str(e)}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=f"CSV export failed: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)