// Combined JavaScript modules for deployment

// Emitter management module
class EmitterManager {
export class EmitterManager {
    constructor() {
        this.inlets = [];
        this.directionalEmitters = [];
        this.outlets = [];
        this.walls = [];
        
        // Emitter creation state
        this.placementMode = null;
        this.isDrawingWall = false;
        this.currentWallStart = null;
        this.currentEmitterStart = null;
        this.isCreatingDirectional = false;
        
        // Directional emitter parameters
        this.emitterVelocity = 50;
        this.emitterSpread = 0.3; // ~17 degrees
    }

    setMode(mode) {
        this.placementMode = mode;
        this.isDrawingWall = false;
        this.isCreatingDirectional = false;
        this.currentWallStart = null;
        this.currentEmitterStart = null;
        
        return this.getModeDisplay(mode);
    }

    getModeDisplay(mode) {
        switch(mode) {
            case 'inlet':
                return { text: 'Mode: Add Inlet (Right-click to cancel)', cursor: 'copy' };
            case 'directional':
                return { text: 'Mode: Add Directional Emitter - Click and drag (Right-click to cancel)', cursor: 'crosshair' };
            case 'outlet':
                return { text: 'Mode: Add Outlet (Right-click to cancel)', cursor: 'copy' };
            case 'wall':
                return { text: 'Mode: Draw Wall (Right-click to cancel)', cursor: 'crosshair' };
            case 'eraser':
                return { text: 'Mode: Erase Objects - Click to remove (Right-click to cancel)', cursor: 'not-allowed' };
            default:
                return { text: '', cursor: 'default' };
        }
    }

    handleMouseDown(x, y, simulator) {
        if (!this.placementMode || !simulator) return false;

        switch (this.placementMode) {
            case 'inlet':
                this.inlets.push({ x, y, radius: 20 });
                simulator.add_inlet(x, y, 20);
                return true;

            case 'directional':
                this.isCreatingDirectional = true;
                this.currentEmitterStart = { x, y };
                return true;

            case 'outlet':
                this.outlets.push({ x, y, radius: 30 });
                simulator.add_outlet(x, y, 30);
                return true;

            case 'wall':
                this.isDrawingWall = true;
                this.currentWallStart = { x, y };
                return true;

            case 'eraser':
                this.eraseObjectAt(x, y, simulator);
                return true;
        }
        return false;
    }

    handleMouseUp(x, y, simulator) {
        if (!this.placementMode || !simulator) return false;

        if (this.isCreatingDirectional && this.currentEmitterStart) {
            const dx = x - this.currentEmitterStart.x;
            const dy = y - this.currentEmitterStart.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            
            if (distance > 5) { // Minimum distance for direction
                const angle = Math.atan2(dy, dx);
                this.directionalEmitters.push({
                    x: this.currentEmitterStart.x,
                    y: this.currentEmitterStart.y,
                    radius: 15,
                    angle,
                    velocity: this.emitterVelocity,
                    spread: this.emitterSpread
                });
                
                simulator.add_directional_emitter(
                    this.currentEmitterStart.x,
                    this.currentEmitterStart.y,
                    15,
                    angle,
                    this.emitterVelocity,
                    this.emitterSpread
                );
            }
            
            this.isCreatingDirectional = false;
            this.currentEmitterStart = null;
            return true;
        }

        if (this.isDrawingWall && this.currentWallStart) {
            const dx = x - this.currentWallStart.x;
            const dy = y - this.currentWallStart.y;
            
            if (Math.sqrt(dx * dx + dy * dy) > 2) {
                this.walls.push({ 
                    start: this.currentWallStart, 
                    end: { x, y } 
                });
                simulator.add_wall(this.currentWallStart.x, this.currentWallStart.y, x, y);
            }
            
            this.isDrawingWall = false;
            this.currentWallStart = null;
            return true;
        }

        return false;
    }

    handleRightClick() {
        if (this.placementMode) {
            this.placementMode = null;
            this.isDrawingWall = false;
            this.isCreatingDirectional = false;
            this.currentWallStart = null;
            this.currentEmitterStart = null;
            return true;
        }
        return false;
    }

    eraseObjectAt(x, y, simulator) {
        let erased = false;
        const eraseTolerance = 25; // pixels
        
        // Check inlets
        for (let i = this.inlets.length - 1; i >= 0; i--) {
            const inlet = this.inlets[i];
            const dx = x - inlet.x;
            const dy = y - inlet.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            
            if (distance <= inlet.radius + eraseTolerance) {
                this.inlets.splice(i, 1);
                erased = true;
                break;
            }
        }
        
        // Check directional emitters
        if (!erased) {
            for (let i = this.directionalEmitters.length - 1; i >= 0; i--) {
                const emitter = this.directionalEmitters[i];
                const dx = x - emitter.x;
                const dy = y - emitter.y;
                const distance = Math.sqrt(dx * dx + dy * dy);
                
                if (distance <= emitter.radius + eraseTolerance) {
                    this.directionalEmitters.splice(i, 1);
                    erased = true;
                    break;
                }
            }
        }
        
        // Check outlets
        if (!erased) {
            for (let i = this.outlets.length - 1; i >= 0; i--) {
                const outlet = this.outlets[i];
                const dx = x - outlet.x;
                const dy = y - outlet.y;
                const distance = Math.sqrt(dx * dx + dy * dy);
                
                if (distance <= outlet.radius + eraseTolerance) {
                    this.outlets.splice(i, 1);
                    erased = true;
                    break;
                }
            }
        }
        
        // Check walls
        if (!erased) {
            for (let i = this.walls.length - 1; i >= 0; i--) {
                const wall = this.walls[i];
                const distance = this.pointToLineDistance(x, y, wall.start.x, wall.start.y, wall.end.x, wall.end.y);
                
                if (distance <= eraseTolerance) {
                    this.walls.splice(i, 1);
                    erased = true;
                    break;
                }
            }
        }
        
        // Rebuild the entire environment in the simulator
        if (erased && simulator) {
            simulator.clear_environment();
            
            // Re-add all remaining objects
            this.inlets.forEach(inlet => {
                simulator.add_inlet(inlet.x, inlet.y, inlet.radius);
            });
            
            this.directionalEmitters.forEach(emitter => {
                simulator.add_directional_emitter(
                    emitter.x, emitter.y, emitter.radius,
                    emitter.angle, emitter.velocity, emitter.spread
                );
            });
            
            this.outlets.forEach(outlet => {
                simulator.add_outlet(outlet.x, outlet.y, outlet.radius);
            });
            
            this.walls.forEach(wall => {
                simulator.add_wall(wall.start.x, wall.start.y, wall.end.x, wall.end.y);
            });
        }
        
        return erased;
    }

    pointToLineDistance(px, py, x1, y1, x2, y2) {
        const dx = x2 - x1;
        const dy = y2 - y1;
        const length = Math.sqrt(dx * dx + dy * dy);
        
        if (length === 0) {
            // Line is actually a point
            return Math.sqrt((px - x1) * (px - x1) + (py - y1) * (py - y1));
        }
        
        const t = Math.max(0, Math.min(1, ((px - x1) * dx + (py - y1) * dy) / (length * length)));
        const projection_x = x1 + t * dx;
        const projection_y = y1 + t * dy;
        
        return Math.sqrt((px - projection_x) * (px - projection_x) + (py - projection_y) * (py - projection_y));
    }

    clear(simulator) {
        this.inlets = [];
        this.directionalEmitters = [];
        this.outlets = [];
        this.walls = [];
        this.placementMode = null;
        this.isDrawingWall = false;
        this.isCreatingDirectional = false;
        this.currentWallStart = null;
        this.currentEmitterStart = null;
        
        if (simulator) {
            simulator.clear_environment();
        }
    }

    drawEnvironment(ctx) {
        // Draw inlets (blue circles)
        this.inlets.forEach(inlet => {
            ctx.beginPath();
            ctx.arc(inlet.x, inlet.y, inlet.radius, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(59, 130, 246, 0.7)';
            ctx.lineWidth = 3;
            ctx.stroke();
        });

        // Draw directional emitters (purple circles with direction arrow)
        this.directionalEmitters.forEach(emitter => {
            // Circle
            ctx.beginPath();
            ctx.arc(emitter.x, emitter.y, emitter.radius, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(147, 51, 234, 0.7)';
            ctx.lineWidth = 3;
            ctx.stroke();
            
            // Direction arrow
            const arrowLength = emitter.radius + 15;
            const endX = emitter.x + Math.cos(emitter.angle) * arrowLength;
            const endY = emitter.y + Math.sin(emitter.angle) * arrowLength;
            
            ctx.beginPath();
            ctx.moveTo(emitter.x, emitter.y);
            ctx.lineTo(endX, endY);
            ctx.strokeStyle = 'rgba(147, 51, 234, 0.9)';
            ctx.lineWidth = 2;
            ctx.stroke();
            
            // Arrowhead
            const headLength = 8;
            const headAngle = Math.PI / 6;
            ctx.beginPath();
            ctx.moveTo(endX, endY);
            ctx.lineTo(
                endX - headLength * Math.cos(emitter.angle - headAngle),
                endY - headLength * Math.sin(emitter.angle - headAngle)
            );
            ctx.moveTo(endX, endY);
            ctx.lineTo(
                endX - headLength * Math.cos(emitter.angle + headAngle),
                endY - headLength * Math.sin(emitter.angle + headAngle)
            );
            ctx.stroke();
        });

        // Draw outlets (red circles)
        this.outlets.forEach(outlet => {
            ctx.beginPath();
            ctx.arc(outlet.x, outlet.y, outlet.radius, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(239, 68, 68, 0.7)';
            ctx.lineWidth = 3;
            ctx.stroke();
        });

        // Draw walls (green lines)
        this.walls.forEach(wall => {
            ctx.beginPath();
            ctx.moveTo(wall.start.x, wall.start.y);
            ctx.lineTo(wall.end.x, wall.end.y);
            ctx.strokeStyle = 'rgba(34, 197, 94, 0.8)';
            ctx.lineWidth = 4;
            ctx.stroke();
        });
    }

    drawPreview(ctx, mousePos) {
        if (!mousePos) return;

        // Wall preview
        if (this.isDrawingWall && this.currentWallStart) {
            ctx.beginPath();
            ctx.moveTo(this.currentWallStart.x, this.currentWallStart.y);
            ctx.lineTo(mousePos.x, mousePos.y);
            ctx.strokeStyle = 'rgba(34, 197, 94, 0.5)';
            ctx.lineWidth = 4;
            ctx.setLineDash([5, 10]);
            ctx.stroke();
            ctx.setLineDash([]);
        }

        // Directional emitter preview
        if (this.isCreatingDirectional && this.currentEmitterStart) {
            const dx = mousePos.x - this.currentEmitterStart.x;
            const dy = mousePos.y - this.currentEmitterStart.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            
            if (distance > 5) {
                const angle = Math.atan2(dy, dx);
                
                // Preview circle
                ctx.beginPath();
                ctx.arc(this.currentEmitterStart.x, this.currentEmitterStart.y, 15, 0, Math.PI * 2);
                ctx.strokeStyle = 'rgba(147, 51, 234, 0.5)';
                ctx.lineWidth = 3;
                ctx.setLineDash([3, 3]);
                ctx.stroke();
                ctx.setLineDash([]);
                
                // Preview arrow
                const arrowLength = 30;
                const endX = this.currentEmitterStart.x + Math.cos(angle) * arrowLength;
                const endY = this.currentEmitterStart.y + Math.sin(angle) * arrowLength;
                
                ctx.beginPath();
                ctx.moveTo(this.currentEmitterStart.x, this.currentEmitterStart.y);
                ctx.lineTo(endX, endY);
                ctx.strokeStyle = 'rgba(147, 51, 234, 0.7)';
                ctx.lineWidth = 2;
                ctx.stroke();
            }
        }
    }

    setEmitterVelocity(velocity) {
        this.emitterVelocity = velocity;
    }

    setEmitterSpread(spread) {
        this.emitterSpread = spread;
    }
}

// Rendering and visualization module  
class FluidRenderer {
export class FluidRenderer {
    constructor(canvas, ctx) {
        this.canvas = canvas;
        this.ctx = ctx;
        
        // Performance state
        this.frameCount = 0;
        this.cachedVectorField = null;
        this.lastVectorFieldUpdate = 0;
        
        // Rendering options
        this.showVectorField = false;
        this.gridResolution = 30;
        this.vectorFieldUpdateInterval = 3;
        this.performanceCheckInterval = 60;
    }

    drawParticles(simulator) {
        if (!simulator) return;
        
        const positions = simulator.get_particle_positions();
        const velocities = simulator.get_particle_velocities();
        const particleCount = positions.length / 2;
        
        // Get actual particle radius from simulator
        const baseRadius = simulator.get_particle_radius();
        
        // Adaptive particle size based on count for performance
        const radius = particleCount > 1000 ? Math.max(1, baseRadius * 0.7) : 
                      particleCount > 2000 ? Math.max(0.8, baseRadius * 0.5) : baseRadius;
        
        // Batch drawing for better performance
        let currentColor = '';
        this.ctx.beginPath();
        
        for (let i = 0; i < positions.length; i += 2) {
            const x = positions[i];
            const y = positions[i + 1];
            const vx = velocities[i] || 0;
            const vy = velocities[i + 1] || 0;
            
            const vMag = Math.sqrt(vx * vx + vy * vy);
            const blue = Math.min(255, 50 + vMag * 20);
            const red = Math.max(50, 200 - vMag * 15);
            const green = Math.max(50, 180 - vMag * 15);
            
            const color = `rgb(${Math.floor(red)},${Math.floor(green)},${Math.floor(blue)})`;
            
            // Only change fill style when color changes for batching
            if (color !== currentColor) {
                if (currentColor) this.ctx.fill();
                this.ctx.fillStyle = color;
                currentColor = color;
                this.ctx.beginPath();
            }
            
            this.ctx.moveTo(x + radius, y);
            this.ctx.arc(x, y, radius, 0, Math.PI * 2);
        }
        
        if (currentColor) this.ctx.fill();
    }

    drawVectorField(simulator) {
        if (!simulator || !this.showVectorField) return;
        
        // Update vector field data only every few frames for performance
        if (this.frameCount % this.vectorFieldUpdateInterval === 0 || !this.cachedVectorField) {
            this.cachedVectorField = simulator.get_vector_field(this.gridResolution);
            this.lastVectorFieldUpdate = this.frameCount;
        }
        
        if (!this.cachedVectorField || this.cachedVectorField.length === 0) return;
        
        this.ctx.strokeStyle = 'rgba(255, 255, 0, 0.7)';
        this.ctx.lineWidth = 1;

        const rows = Math.ceil(this.canvas.height / this.gridResolution);
        const cols = Math.ceil(this.canvas.width / this.gridResolution);
        
        // Find max magnitude for scaling (cached calculation)
        let maxMag = 0;
        for (let i = 0; i < this.cachedVectorField.length; i += 2) {
            const mag = Math.hypot(this.cachedVectorField[i], this.cachedVectorField[i + 1]);
            if (mag > maxMag) maxMag = mag;
        }

        const maxVectorLength = this.gridResolution * 0.8;
        const scale = (maxMag > 0.01) ? maxVectorLength / maxMag : 0;

        // Batch drawing for better performance
        this.ctx.beginPath();
        for (let i = 0; i < rows; i++) {
            for (let j = 0; j < cols; j++) {
                const idx = (i * cols + j) * 2;
                if (idx + 1 < this.cachedVectorField.length) {
                    const vx = this.cachedVectorField[idx];
                    const vy = this.cachedVectorField[idx + 1];
                    const mag = Math.hypot(vx, vy);
                    
                    if (mag > 0.01) {
                        const startX = j * this.gridResolution + this.gridResolution / 2;
                        const startY = i * this.gridResolution + this.gridResolution / 2;
                        
                        const endX = startX + vx * scale;
                        const endY = startY + vy * scale;

                        this.ctx.moveTo(startX, startY);
                        this.ctx.lineTo(endX, endY);
                        
                        // Simplified arrowhead for performance
                        const angle = Math.atan2(endY - startY, endX - startX);
                        this.ctx.moveTo(endX, endY);
                        this.ctx.lineTo(endX - 3 * Math.cos(angle - Math.PI / 6), endY - 3 * Math.sin(angle - Math.PI / 6));
                        this.ctx.moveTo(endX, endY);
                        this.ctx.lineTo(endX - 3 * Math.cos(angle + Math.PI / 6), endY - 3 * Math.sin(angle + Math.PI / 6));
                    }
                }
            }
        }
        this.ctx.stroke();
    }

    render(simulator, emitterManager, mousePos = null) {
        const frameStart = performance.now();
        
        // Clear canvas
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        
        // Draw vector field if enabled (drawn first, so it's in the background)
        this.drawVectorField(simulator);

        // Draw environment objects
        emitterManager.drawEnvironment(this.ctx);

        // Draw particles
        this.drawParticles(simulator);

        // Draw previews for objects being created
        emitterManager.drawPreview(this.ctx, mousePos);
        
        this.frameCount++;
        
        // Performance monitoring
        if (this.frameCount % this.performanceCheckInterval === 0) {
            const frameTime = performance.now() - frameStart;
            if (frameTime > 16.67) { // 60fps threshold
                console.log(`Frame took ${frameTime.toFixed(1)}ms (target: 16.7ms)`);
            }
        }
    }

    setShowVectorField(show) {
        this.showVectorField = show;
        if (!show) {
            this.cachedVectorField = null;
        }
    }

    setGridResolution(resolution) {
        this.gridResolution = resolution;
        this.cachedVectorField = null; // Force update
    }

    resizeCanvas() {
        const container = this.canvas.parentElement;
        const size = Math.min(container.clientWidth, container.clientHeight);
        this.canvas.width = size;
        this.canvas.height = size;
        this.cachedVectorField = null; // Force vector field recalculation
    }
}

// UI Controls and parameter management
class SimulationControls {
export class SimulationControls {
    constructor() {
        this.isRunning = false;
        this.animationFrameId = null;
        this.simulator = null;
        
        // UI elements
        this.startStopBtn = null;
        this.resetBtn = null;
        this.modeIndicator = null;
        
        // Emitter control elements
        this.velocitySlider = null;
        this.spreadSlider = null;
        
        this.initializeElements();
        this.setupEventListeners();
    }

    initializeElements() {
        this.startStopBtn = document.getElementById('startStop');
        this.resetBtn = document.getElementById('reset');
        this.modeIndicator = document.getElementById('modeIndicator');
        
        // Create emitter controls if they don't exist
        this.createEmitterControls();
        
        // Initialize parameter sliders
        this.setupParameterSliders();
    }

    createEmitterControls() {
        const controlsContainer = document.querySelector('#controls .grid');
        if (!controlsContainer) return;

        // Add directional emitter button if it doesn't exist
        if (!document.getElementById('addDirectionalEmitter')) {
            const dirEmitterBtn = document.createElement('button');
            dirEmitterBtn.id = 'addDirectionalEmitter';
            dirEmitterBtn.className = 'btn bg-purple-500 hover:bg-purple-600 col-span-2 px-3 py-2 rounded-lg font-semibold';
            dirEmitterBtn.textContent = 'Add Directional Emitter';
            
            // Insert after the wall button
            const wallBtn = document.getElementById('drawWall');
            if (wallBtn) {
                wallBtn.parentNode.insertBefore(dirEmitterBtn, wallBtn.nextSibling);
            }
        }

        // Add emitter parameter controls
        this.addEmitterParameterControls();
    }

    addEmitterParameterControls() {
        const controlsDiv = document.getElementById('controls');
        if (!controlsDiv || document.getElementById('emitterControls')) return;

        const emitterControlsHTML = `
            <div id="emitterControls" class="mt-6">
                <h2 class="text-lg font-semibold mb-2 text-center text-indigo-300">Emitter Settings</h2>
                <div class="slider-container">
                    <label for="emitterVelocity" class="slider-label">
                        <span>Emission Velocity</span>
                        <span id="emitterVelocityValue">50</span>
                    </label>
                    <input type="range" id="emitterVelocity" min="10" max="200" step="5" value="50" class="w-full">
                </div>
                <div class="slider-container">
                    <label for="emitterSpread" class="slider-label">
                        <span>Emission Spread (°)</span>
                        <span id="emitterSpreadValue">17</span>
                    </label>
                    <input type="range" id="emitterSpread" min="0" max="90" step="1" value="17" class="w-full">
                </div>
            </div>
        `;

        // Insert before simulation control buttons
        const buttonSection = controlsDiv.querySelector('.mt-8');
        if (buttonSection) {
            buttonSection.insertAdjacentHTML('beforebegin', emitterControlsHTML);
        }

        this.velocitySlider = document.getElementById('emitterVelocity');
        this.spreadSlider = document.getElementById('emitterSpread');
    }

    setupParameterSliders() {
        document.querySelectorAll('.slider-container input[type="range"]').forEach(slider => {
            const valueElem = slider.parentElement.querySelector('.slider-label span:last-child');
            slider.addEventListener('input', () => {
                valueElem.textContent = slider.value;
                const value = parseFloat(slider.value);
                
                if (!this.simulator) return;
                
                switch (slider.dataset.param) {
                    case 'GRAVITY':
                        this.simulator.set_gravity(value);
                        break;
                    case 'VISCOSITY':
                        this.simulator.set_viscosity(value);
                        break;
                    case 'PARTICLE_RADIUS':
                        this.simulator.set_particle_radius(value);
                        break;
                    case 'STIFFNESS':
                        this.simulator.set_stiffness(value);
                        break;
                    case 'REST_DENSITY':
                        this.simulator.set_rest_density(value);
                        break;
                    case 'MAX_PARTICLES':
                        this.simulator.set_max_particles(parseInt(value));
                        break;
                    case 'TIME_STEP':
                        this.simulator.set_time_step(value);
                        break;
                }
            });
            slider.dispatchEvent(new Event('input'));
        });
    }

    setupEventListeners() {
        // Emitter velocity control
        if (this.velocitySlider) {
            this.velocitySlider.addEventListener('input', (e) => {
                const value = parseFloat(e.target.value);
                document.getElementById('emitterVelocityValue').textContent = value;
                // This will be handled by the emitter manager
                this.onEmitterVelocityChange?.(value);
            });
        }

        // Emitter spread control
        if (this.spreadSlider) {
            this.spreadSlider.addEventListener('input', (e) => {
                const degrees = parseFloat(e.target.value);
                const radians = degrees * Math.PI / 180;
                document.getElementById('emitterSpreadValue').textContent = degrees;
                // This will be handled by the emitter manager
                this.onEmitterSpreadChange?.(radians);
            });
        }
    }

    setSimulator(simulator) {
        this.simulator = simulator;
    }

    startSimulation(animateCallback) {
        if (!this.isRunning) {
            this.isRunning = true;
            this.startStopBtn.textContent = 'Stop';
            this.startStopBtn.classList.replace('bg-indigo-600', 'bg-pink-600');
            this.startStopBtn.classList.replace('hover:bg-indigo-700', 'hover:bg-pink-700');
            this.animate(animateCallback);
        }
    }

    stopSimulation() {
        if (this.isRunning) {
            this.isRunning = false;
            this.startStopBtn.textContent = 'Start';
            this.startStopBtn.classList.replace('bg-pink-600', 'bg-indigo-600');
            this.startStopBtn.classList.replace('hover:bg-pink-700', 'hover:bg-indigo-700');
            cancelAnimationFrame(this.animationFrameId);
        }
    }

    animate(callback) {
        if (this.isRunning && callback) {
            callback();
            this.animationFrameId = requestAnimationFrame(() => this.animate(callback));
        }
    }

    updateModeIndicator(text, cursor) {
        if (this.modeIndicator) {
            this.modeIndicator.textContent = text;
        }
        
        // This will be set by the main application
        this.onCursorChange?.(cursor);
    }

    resetSimulation(resetCallback) {
        this.stopSimulation();
        if (resetCallback) {
            resetCallback();
        }
    }

    // Callback setters
    onEmitterVelocityChange = null;
    onEmitterSpreadChange = null;
    onCursorChange = null;
}

// Expose classes globally for non-module usage
window.EmitterManager = EmitterManager;
window.FluidRenderer = FluidRenderer;
window.SimulationControls = SimulationControls;
