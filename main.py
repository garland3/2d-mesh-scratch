from fastapi import FastAPI, HTTPException
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

app = FastAPI(title="2D Geometry & FEA Mesh Generator")

# Data models
class Point(BaseModel):
    x: float
    y: float
    id: Optional[str] = None

class Geometry(BaseModel):
    points: List[Point]
    name: Optional[str] = "geometry"

class MeshRequest(BaseModel):
    geometry: Geometry
    max_area: Optional[float] = 0.1
    min_angle: Optional[float] = 20.0

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
            Max Area: <input type="number" id="maxArea" value="100" step="10"> (square units)
            Min Angle: <input type="number" id="minAngle" value="20" step="1"> (degrees)
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
                
                // Convert max area from UI units to mesh units
                // UI uses square units where 1 unit = 1 pixel
                // Max area of 100 square units = reasonable triangle size
                const meshMaxArea = maxArea;
                
                console.log('Starting mesh generation...');
                console.log('Points:', points);
                console.log('Max Area (UI):', maxArea);
                console.log('Max Area (mesh):', meshMaxArea);
                console.log('Min Angle:', minAngle);
                
                const requestData = {
                    geometry: {
                        points: points,
                        name: 'interactive_geometry'
                    },
                    max_area: meshMaxArea,
                    min_angle: minAngle
                };
                
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
                        document.getElementById('meshInfo').innerHTML = `
                            <h3>Mesh Info:</h3>
                            <p>Triangles: ${mesh.triangles.length}</p>
                            <p>Vertices: ${mesh.vertices.length}</p>
                        `;
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
        raise HTTPException(status_code=400, detail="Need at least 3 points to generate mesh")
    
    try:
        # Prepare input for Rust binary
        rust_input = {
            "geometry": {
                "points": [{"x": p.x, "y": p.y, "id": p.id} for p in request.geometry.points],
                "name": request.geometry.name
            },
            "max_area": request.max_area,
            "min_angle": request.min_angle
        }
        
        print(f"Calling Rust binary with input: {rust_input}")
        
        # Call Rust binary
        process = subprocess.Popen(
            ["./target/release/mesh-generator", "json-stdin"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        stdout, stderr = process.communicate(input=json.dumps(rust_input))
        
        if process.returncode != 0:
            print(f"Rust binary error: {stderr}")
            raise Exception(f"Rust binary failed: {stderr}")
        
        # Parse output from Rust binary
        mesh_output = json.loads(stdout)
        print(f"Rust binary output: {mesh_output}")
        
        # Convert to expected format (already in the right format from Rust)
        mesh_result = {
            'vertices': mesh_output['vertices'],
            'triangles': mesh_output['triangles'],
            'triangle_indices': mesh_output['triangle_indices']
        }
        
        # Store mesh
        mesh_id = datetime.now().isoformat()
        meshes[mesh_id] = mesh_result
        
        print(f"Successfully generated mesh with {len(mesh_result['triangles'])} triangles")
        return mesh_result
        
    except Exception as e:
        print(f"Mesh generation error: {str(e)}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=f"Mesh generation failed: {str(e)}")

@app.post("/export-csv")
async def export_csv(request: Geometry):
    try:
        print(f"CSV Export request received")
        print(f"Request points: {[{'x': p.x, 'y': p.y} for p in request.points]}")
        
        # Prepare input for Rust binary (need to create a dummy mesh request)
        rust_input = {
            "geometry": {
                "points": [{"x": p.x, "y": p.y, "id": p.id} for p in request.points],
                "name": request.name
            },
            "max_area": 0.1,  # Default values for CSV generation
            "min_angle": 20.0
        }
        
        print(f"Calling Rust binary for CSV export with input: {rust_input}")
        
        # Call Rust binary for CSV export
        process = subprocess.Popen(
            ["./target/release/mesh-generator", "csv-stdin"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        stdout, stderr = process.communicate(input=json.dumps(rust_input))
        
        if process.returncode != 0:
            print(f"Rust binary error: {stderr}")
            raise Exception(f"Rust binary failed: {stderr}")
        
        csv_content = stdout
        print(f"CSV content length: {len(csv_content)}")
        print(f"CSV content preview: {csv_content[:200]}...")
        
        # Create a temporary file for the CSV
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.csv') as tmp_file:
            tmp_file.write(csv_content)
            tmp_file_path = tmp_file.name
        
        print(f"CSV file created at: {tmp_file_path}")
        
        return FileResponse(
            path=tmp_file_path,
            filename="mesh_data.csv",
            media_type="text/csv"
        )
    
    except Exception as e:
        print(f"CSV export error: {str(e)}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=500, detail=f"CSV export failed: {str(e)}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)