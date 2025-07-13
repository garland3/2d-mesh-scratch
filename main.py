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
    check_volume: Optional[bool] = True
    check_aspect_ratio: Optional[bool] = True
    target_aspect_ratio: Optional[float] = 1.73
    volume_weight: Optional[float] = 0.3
    aspect_ratio_weight: Optional[float] = 0.4
    check_size_uniformity: Optional[bool] = True
    size_uniformity_weight: Optional[float] = 0.3
    target_area: Optional[float] = None
    min_area: Optional[float] = None

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
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body { 
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: #333;
                min-height: 100vh;
                padding: 20px;
            }
            
            .container {
                max-width: 1400px;
                margin: 0 auto;
                background: rgba(255, 255, 255, 0.95);
                border-radius: 15px;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
                padding: 30px;
                backdrop-filter: blur(10px);
            }
            
            h1 {
                text-align: center;
                color: #4a5568;
                margin-bottom: 30px;
                font-size: 2.5em;
                text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
            }
            
            .main-content {
                display: grid;
                grid-template-columns: 1fr 350px;
                gap: 30px;
                align-items: start;
            }
            
            .canvas-section {
                background: white;
                border-radius: 12px;
                padding: 20px;
                box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
            }
            
            #canvas { 
                border: 2px solid #e2e8f0;
                border-radius: 8px;
                cursor: crosshair;
                display: block;
                margin: 0 auto;
                box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
            }
            
            .sidebar {
                background: white;
                border-radius: 12px;
                padding: 25px;
                box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
                max-height: 80vh;
                overflow-y: auto;
            }
            
            .controls {
                margin-bottom: 25px;
            }
            
            .control-group {
                margin-bottom: 20px;
                padding: 15px;
                background: #f8fafc;
                border-radius: 8px;
                border-left: 4px solid #667eea;
            }
            
            .control-group h3 {
                margin-bottom: 15px;
                color: #4a5568;
                font-size: 1.1em;
            }
            
            button { 
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                border: none;
                padding: 12px 20px;
                margin: 5px;
                border-radius: 8px;
                cursor: pointer;
                font-weight: 600;
                transition: all 0.3s ease;
                box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3);
            }
            
            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
            }
            
            button:active {
                transform: translateY(0);
            }
            
            .clear-btn {
                background: linear-gradient(135deg, #f56565 0%, #e53e3e 100%);
                box-shadow: 0 4px 15px rgba(245, 101, 101, 0.3);
            }
            
            .clear-btn:hover {
                box-shadow: 0 6px 20px rgba(245, 101, 101, 0.4);
            }
            
            input, select { 
                padding: 10px;
                margin: 5px 0;
                border: 2px solid #e2e8f0;
                border-radius: 6px;
                width: 100%;
                font-size: 14px;
                transition: border-color 0.3s ease;
            }
            
            input:focus, select:focus {
                outline: none;
                border-color: #667eea;
                box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
            }
            
            .input-row {
                display: flex;
                gap: 10px;
                align-items: center;
            }
            
            .input-row label {
                min-width: 80px;
                font-weight: 600;
                color: #4a5568;
            }
            
            .point-list {
                background: #f8fafc;
                border-radius: 8px;
                padding: 15px;
                max-height: 300px;
                overflow-y: auto;
            }
            
            .point-list h3 {
                margin-bottom: 15px;
                color: #4a5568;
            }
            
            .point-item { 
                margin: 8px 0;
                padding: 12px;
                background: white;
                border-radius: 6px;
                box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
                display: flex;
                justify-content: space-between;
                align-items: center;
            }
            
            .point-item button {
                background: #e53e3e;
                padding: 6px 12px;
                margin: 0;
                font-size: 12px;
            }
            
            .annealing-options {
                background: #edf2f7;
                border: 2px solid #cbd5e0;
                border-radius: 8px;
                padding: 20px;
                margin: 15px 0;
            }
            
            .annealing-options h4 {
                color: #2d3748;
                margin-bottom: 15px;
                font-size: 1.1em;
            }
            
            .checkbox-group {
                display: flex;
                align-items: center;
                margin: 10px 0;
                gap: 8px;
            }
            
            .checkbox-group input[type="checkbox"] {
                width: auto;
                margin: 0;
            }
            
            .checkbox-group label {
                margin: 0;
                font-weight: 500;
            }
            
            #meshInfo {
                background: #f0fff4;
                border: 2px solid #9ae6b4;
                border-radius: 8px;
                padding: 15px;
                margin-top: 20px;
            }
            
            #meshInfo h3 {
                color: #22543d;
                margin-bottom: 10px;
            }
            
            #meshInfo p {
                margin: 5px 0;
                color: #2f855a;
                font-weight: 600;
            }
            
            .toggle-btn {
                background: linear-gradient(135deg, #48bb78 0%, #38a169 100%);
                box-shadow: 0 4px 15px rgba(72, 187, 120, 0.3);
            }
            
            .toggle-btn:hover {
                box-shadow: 0 6px 20px rgba(72, 187, 120, 0.4);
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>2D Geometry & FEA Mesh Generator</h1>
            
            <div class="main-content">
                <div class="canvas-section">
                    <canvas id="canvas" width="800" height="600"></canvas>
                    <div id="meshInfo"></div>
                </div>
                
                <div class="sidebar">
                    <div class="controls">
                        <div class="control-group">
                            <h3>Actions</h3>
                            <button class="clear-btn" onclick="clearPoints()">Clear Points</button>
                            <button onclick="generateMesh()">Generate Mesh</button>
                            <button class="clear-btn" onclick="clearMesh()">Clear Mesh</button>
                            <button class="toggle-btn" onclick="exportCSV()">Export to CSV</button>
                            <button onclick="resetZoom()">Reset Zoom</button>
                        </div>
                        
                        <div class="control-group">
                            <h3>View</h3>
                            <div class="input-row">
                                <label>Zoom:</label>
                                <span id="zoomLevel">100%</span>
                            </div>
                            <small style="color: #666;">Use mouse wheel to zoom in/out</small>
                        </div>
                        
                        <div class="control-group">
                            <h3>Mesh Settings</h3>
                            <div class="input-row">
                                <label>Algorithm:</label>
                                <select id="meshAlgorithm">
                                    <option value="delaunay">Delaunay Triangulation</option>
                                    <option value="paving">Paving (Quad-dominant)</option>
                                    <option value="grid-annealing">Grid Annealing</option>
                                </select>
                            </div>
                            <div class="input-row">
                                <label>Max Area:</label>
                                <input type="number" id="maxArea" value="100" step="10" title="Maximum triangle area in square units">
                            </div>
                            <div class="input-row">
                                <label>Min Angle:</label>
                                <input type="number" id="minAngle" value="20" step="1" title="Minimum triangle angle in degrees">
                            </div>
                        </div>
                        
                        <div class="control-group">
                            <h3>Annealing Options</h3>
                            <div class="annealing-options">
                                <div class="checkbox-group">
                                    <input type="checkbox" id="enableAnnealing" checked>
                                    <label for="enableAnnealing">Enable Annealing (recommended)</label>
                                </div>
                                <div style="margin: 10px 0; padding: 8px; background: #fff3cd; border: 1px solid #ffeaa7; border-radius: 4px; font-size: 12px; color: #856404;">
                                    <strong>Note:</strong> Annealing options apply to Delaunay and Grid-Annealing algorithms only. Paving algorithm has built-in quality control.
                                </div>
                                <div class="checkbox-group">
                                    <input type="checkbox" id="checkVolume" checked>
                                    <label for="checkVolume">Check Volume Uniformity</label>
                                </div>
                                <div class="checkbox-group">
                                    <input type="checkbox" id="checkAspectRatio" checked>
                                    <label for="checkAspectRatio">Check Aspect Ratio</label>
                                </div>
                                <div class="checkbox-group">
                                    <input type="checkbox" id="checkSizeUniformity" checked>
                                    <label for="checkSizeUniformity">Check Size Uniformity</label>
                                </div>
                                <div class="input-row">
                                    <label>Temperature:</label>
                                    <input type="number" id="temperature" value="1000" step="100" min="1">
                                </div>
                                <div class="input-row">
                                    <label>Cooling Rate:</label>
                                    <input type="number" id="coolingRate" value="0.995" step="0.001" min="0.001" max="0.999">
                                </div>
                                <div class="input-row">
                                    <label>Quality:</label>
                                    <input type="number" id="qualityThreshold" value="0.8" step="0.1" min="0.1" max="1.0">
                                </div>
                                <div class="input-row">
                                    <label>Max Iter:</label>
                                    <input type="number" id="maxIterations" value="10000" step="1000" min="100">
                                </div>
                                <div class="input-row">
                                    <label>Target AR:</label>
                                    <input type="number" id="targetAspectRatio" value="1.73" step="0.1" min="1.0" max="10.0">
                                </div>
                                <div class="input-row">
                                    <label>Vol Weight:</label>
                                    <input type="number" id="volumeWeight" value="0.3" step="0.1" min="0.0" max="1.0">
                                </div>
                                <div class="input-row">
                                    <label>AR Weight:</label>
                                    <input type="number" id="aspectRatioWeight" value="0.4" step="0.1" min="0.0" max="1.0">
                                </div>
                                <div class="input-row">
                                    <label>Size Weight:</label>
                                    <input type="number" id="sizeUniformityWeight" value="0.3" step="0.1" min="0.0" max="1.0">
                                </div>
                                <div class="input-row">
                                    <label>Min Area:</label>
                                    <input type="number" id="minArea" value="10" step="5" min="1" title="Minimum triangle area">
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="point-list">
                        <h3>Points</h3>
                        <div id="pointsList"></div>
                    </div>
                </div>
            </div>
        </div>
        
        <script>
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');
            let points = [];
            let mesh = null;
            
            // Zoom and pan variables
            let zoom = 1.0;
            let panX = 0;
            let panY = 0;
            const minZoom = 0.1;
            const maxZoom = 10.0;
            
            // Coordinate system: 1 unit = 1 pixel, but with origin at bottom-left
            // Canvas is 800x600, so world coordinates go from (0,0) to (800,600)
            // This makes the max area setting meaningful (e.g., 100 = 10x10 pixel triangle)
            
            // Convert canvas coordinates to world coordinates (accounting for zoom and pan)
            function canvasToWorld(canvasX, canvasY) {
                const worldX = (canvasX - panX) / zoom;
                const worldY = canvas.height - (canvasY - panY) / zoom;
                return { x: worldX, y: worldY };
            }
            
            // Convert world coordinates to canvas coordinates (accounting for zoom and pan)
            function worldToCanvas(worldX, worldY) {
                const canvasX = worldX * zoom + panX;
                const canvasY = (canvas.height - worldY) * zoom + panY;
                return { x: canvasX, y: canvasY };
            }
            
            canvas.addEventListener('click', addPoint);
            canvas.addEventListener('wheel', handleWheel);
            
            // Handle algorithm selection to show/hide annealing options
            document.getElementById('meshAlgorithm').addEventListener('change', function() {
                const annealingSection = document.querySelector('.annealing-options');
                const enableAnnealingCheckbox = document.getElementById('enableAnnealing');
                
                if (this.value === 'paving') {
                    // Disable annealing for paving algorithm
                    annealingSection.style.opacity = '0.5';
                    enableAnnealingCheckbox.checked = false;
                    enableAnnealingCheckbox.disabled = true;
                    
                    // Disable all annealing controls
                    const annealingInputs = annealingSection.querySelectorAll('input, select');
                    annealingInputs.forEach(input => {
                        if (input.id !== 'enableAnnealing') {
                            input.disabled = true;
                        }
                    });
                } else {
                    // Enable annealing for other algorithms
                    annealingSection.style.opacity = '1';
                    enableAnnealingCheckbox.disabled = false;
                    enableAnnealingCheckbox.checked = true;
                    
                    // Enable all annealing controls
                    const annealingInputs = annealingSection.querySelectorAll('input, select');
                    annealingInputs.forEach(input => {
                        input.disabled = false;
                    });
                }
            });
            
            // Handle mouse wheel for zooming
            function handleWheel(event) {
                event.preventDefault();
                
                const rect = canvas.getBoundingClientRect();
                const mouseX = event.clientX - rect.left;
                const mouseY = event.clientY - rect.top;
                
                // Get the world coordinates of the mouse position before zoom
                const worldCoordsBefore = canvasToWorld(mouseX, mouseY);
                
                // Apply zoom
                const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
                const newZoom = Math.max(minZoom, Math.min(maxZoom, zoom * zoomFactor));
                
                if (newZoom !== zoom) {
                    zoom = newZoom;
                    
                    // Get the world coordinates of the mouse position after zoom
                    const worldCoordsAfter = canvasToWorld(mouseX, mouseY);
                    
                    // Adjust pan to keep the mouse position fixed in world coordinates
                    const deltaX = (worldCoordsAfter.x - worldCoordsBefore.x) * zoom;
                    const deltaY = (worldCoordsAfter.y - worldCoordsBefore.y) * zoom;
                    
                    panX += deltaX;
                    panY -= deltaY; // Flip Y axis
                    
                    updateZoomDisplay();
                    drawPoints();
                }
            }
            
            // Reset zoom and pan to default
            function resetZoom() {
                zoom = 1.0;
                panX = 0;
                panY = 0;
                updateZoomDisplay();
                drawPoints();
            }
            
            // Update zoom level display
            function updateZoomDisplay() {
                document.getElementById('zoomLevel').textContent = (zoom * 100).toFixed(0) + '%';
            }
            
            // Annealing options are now always visible, no need to toggle visibility
            
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
                // Calculate visible world bounds
                const topLeft = canvasToWorld(0, 0);
                const bottomRight = canvasToWorld(canvas.width, canvas.height);
                
                // Adaptive grid size based on zoom level
                let baseGridSize = 50;
                let gridSize = baseGridSize;
                while (gridSize * zoom < 20) {
                    gridSize *= 2;
                }
                while (gridSize * zoom > 100) {
                    gridSize /= 2;
                }
                
                // Draw grid lines
                ctx.strokeStyle = '#e0e0e0';
                ctx.lineWidth = 0.5 / zoom;
                
                // Calculate grid start and end points
                const startX = Math.floor(topLeft.x / gridSize) * gridSize;
                const endX = Math.ceil(bottomRight.x / gridSize) * gridSize;
                const startY = Math.floor(bottomRight.y / gridSize) * gridSize;
                const endY = Math.ceil(topLeft.y / gridSize) * gridSize;
                
                // Vertical grid lines
                for (let worldX = startX; worldX <= endX; worldX += gridSize) {
                    const canvasCoords1 = worldToCanvas(worldX, topLeft.y);
                    const canvasCoords2 = worldToCanvas(worldX, bottomRight.y);
                    ctx.beginPath();
                    ctx.moveTo(canvasCoords1.x, canvasCoords1.y);
                    ctx.lineTo(canvasCoords2.x, canvasCoords2.y);
                    ctx.stroke();
                }
                
                // Horizontal grid lines
                for (let worldY = startY; worldY <= endY; worldY += gridSize) {
                    const canvasCoords1 = worldToCanvas(topLeft.x, worldY);
                    const canvasCoords2 = worldToCanvas(bottomRight.x, worldY);
                    ctx.beginPath();
                    ctx.moveTo(canvasCoords1.x, canvasCoords1.y);
                    ctx.lineTo(canvasCoords2.x, canvasCoords2.y);
                    ctx.stroke();
                }
                
                // Draw scale marks and labels (only if zoom is reasonable)
                if (zoom > 0.3) {
                    ctx.fillStyle = '#666';
                    ctx.font = `${12 / zoom}px Arial`;
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'top';
                    
                    // X-axis scale marks
                    for (let worldX = startX; worldX <= endX; worldX += gridSize) {
                        const canvasCoords = worldToCanvas(worldX, 0);
                        if (canvasCoords.y >= 0 && canvasCoords.y <= canvas.height) {
                            ctx.save();
                            ctx.scale(zoom, zoom);
                            ctx.fillText(worldX.toFixed(0), canvasCoords.x / zoom, (canvasCoords.y + 5) / zoom);
                            ctx.restore();
                        }
                    }
                    
                    // Y-axis scale marks
                    ctx.textAlign = 'right';
                    ctx.textBaseline = 'middle';
                    for (let worldY = startY; worldY <= endY; worldY += gridSize) {
                        const canvasCoords = worldToCanvas(0, worldY);
                        if (canvasCoords.x >= 0 && canvasCoords.x <= canvas.width) {
                            ctx.save();
                            ctx.scale(zoom, zoom);
                            ctx.fillText(worldY.toFixed(0), (canvasCoords.x - 5) / zoom, canvasCoords.y / zoom);
                            ctx.restore();
                        }
                    }
                }
                
                // Draw axes
                ctx.strokeStyle = '#333';
                ctx.lineWidth = 2 / zoom;
                
                // X-axis (at world Y=0)
                const xAxis1 = worldToCanvas(topLeft.x, 0);
                const xAxis2 = worldToCanvas(bottomRight.x, 0);
                if (xAxis1.y >= 0 && xAxis1.y <= canvas.height) {
                    ctx.beginPath();
                    ctx.moveTo(xAxis1.x, xAxis1.y);
                    ctx.lineTo(xAxis2.x, xAxis2.y);
                    ctx.stroke();
                }
                
                // Y-axis (at world X=0)
                const yAxis1 = worldToCanvas(0, topLeft.y);
                const yAxis2 = worldToCanvas(0, bottomRight.y);
                if (yAxis1.x >= 0 && yAxis1.x <= canvas.width) {
                    ctx.beginPath();
                    ctx.moveTo(yAxis1.x, yAxis1.y);
                    ctx.lineTo(yAxis2.x, yAxis2.y);
                    ctx.stroke();
                }
            }
            
            function drawPoints() {
                ctx.clearRect(0, 0, canvas.width, canvas.height);
                
                // Draw grid first
                drawGrid();
                
                // Draw points
                ctx.fillStyle = 'red';
                points.forEach(point => {
                    const canvasCoords = worldToCanvas(point.x, point.y);
                    if (canvasCoords.x >= -10 && canvasCoords.x <= canvas.width + 10 && 
                        canvasCoords.y >= -10 && canvasCoords.y <= canvas.height + 10) {
                        ctx.beginPath();
                        ctx.arc(canvasCoords.x, canvasCoords.y, Math.max(3, 5 / zoom), 0, 2 * Math.PI);
                        ctx.fill();
                    }
                });
                
                // Draw lines between consecutive points
                if (points.length > 1) {
                    ctx.strokeStyle = 'blue';
                    ctx.lineWidth = Math.max(1, 2 / zoom);
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
                    ctx.lineWidth = Math.max(0.5, 1 / zoom);
                    
                    mesh.triangles.forEach(triangle => {
                        const p1 = worldToCanvas(triangle[0].x, triangle[0].y);
                        const p2 = worldToCanvas(triangle[1].x, triangle[1].y);
                        const p3 = worldToCanvas(triangle[2].x, triangle[2].y);
                        
                        // Only draw if at least one point is visible
                        if ((p1.x >= -10 && p1.x <= canvas.width + 10 && p1.y >= -10 && p1.y <= canvas.height + 10) ||
                            (p2.x >= -10 && p2.x <= canvas.width + 10 && p2.y >= -10 && p2.y <= canvas.height + 10) ||
                            (p3.x >= -10 && p3.x <= canvas.width + 10 && p3.y >= -10 && p3.y <= canvas.height + 10)) {
                            ctx.beginPath();
                            ctx.moveTo(p1.x, p1.y);
                            ctx.lineTo(p2.x, p2.y);
                            ctx.lineTo(p3.x, p3.y);
                            ctx.closePath();
                            ctx.stroke();
                        }
                    });
                }
                
                // Draw quads
                if (mesh.quads && mesh.quads.length > 0) {
                    ctx.strokeStyle = 'purple';
                    ctx.lineWidth = Math.max(0.5, 1 / zoom);
                    
                    mesh.quads.forEach(quad => {
                        const p1 = worldToCanvas(quad[0].x, quad[0].y);
                        const p2 = worldToCanvas(quad[1].x, quad[1].y);
                        const p3 = worldToCanvas(quad[2].x, quad[2].y);
                        const p4 = worldToCanvas(quad[3].x, quad[3].y);
                        
                        // Only draw if at least one point is visible
                        if ((p1.x >= -10 && p1.x <= canvas.width + 10 && p1.y >= -10 && p1.y <= canvas.height + 10) ||
                            (p2.x >= -10 && p2.x <= canvas.width + 10 && p2.y >= -10 && p2.y <= canvas.height + 10) ||
                            (p3.x >= -10 && p3.x <= canvas.width + 10 && p3.y >= -10 && p3.y <= canvas.height + 10) ||
                            (p4.x >= -10 && p4.x <= canvas.width + 10 && p4.y >= -10 && p4.y <= canvas.height + 10)) {
                            ctx.beginPath();
                            ctx.moveTo(p1.x, p1.y);
                            ctx.lineTo(p2.x, p2.y);
                            ctx.lineTo(p3.x, p3.y);
                            ctx.lineTo(p4.x, p4.y);
                            ctx.closePath();
                            ctx.stroke();
                        }
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
            
            // Initialize the interface
            updateZoomDisplay();
            drawPoints();
            
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
                
                // Add annealing options if annealing is enabled
                const enableAnnealing = document.getElementById('enableAnnealing').checked;
                const checkVolume = document.getElementById('checkVolume').checked;
                const checkAspectRatio = document.getElementById('checkAspectRatio').checked;
                const checkSizeUniformity = document.getElementById('checkSizeUniformity').checked;
                
                if (enableAnnealing && (algorithm === 'grid-annealing' || checkVolume || checkAspectRatio || checkSizeUniformity)) {
                    requestData.annealing_options = {
                        temperature: parseFloat(document.getElementById('temperature').value),
                        cooling_rate: parseFloat(document.getElementById('coolingRate').value),
                        quality_threshold: parseFloat(document.getElementById('qualityThreshold').value),
                        max_iterations: parseInt(document.getElementById('maxIterations').value),
                        check_volume: checkVolume,
                        check_aspect_ratio: checkAspectRatio,
                        check_size_uniformity: checkSizeUniformity,
                        target_aspect_ratio: parseFloat(document.getElementById('targetAspectRatio').value),
                        volume_weight: parseFloat(document.getElementById('volumeWeight').value),
                        aspect_ratio_weight: parseFloat(document.getElementById('aspectRatioWeight').value),
                        size_uniformity_weight: parseFloat(document.getElementById('sizeUniformityWeight').value),
                        target_area: meshMaxArea, // Use the same area as mesh generation
                        min_area: parseFloat(document.getElementById('minArea').value)
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
                "max_iterations": request.annealing_options.max_iterations,
                "check_volume": request.annealing_options.check_volume,
                "check_aspect_ratio": request.annealing_options.check_aspect_ratio,
                "target_aspect_ratio": request.annealing_options.target_aspect_ratio,
                "volume_weight": request.annealing_options.volume_weight,
                "aspect_ratio_weight": request.annealing_options.aspect_ratio_weight,
                "check_size_uniformity": request.annealing_options.check_size_uniformity,
                "size_uniformity_weight": request.annealing_options.size_uniformity_weight,
                "target_area": request.annealing_options.target_area,
                "min_area": request.annealing_options.min_area
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