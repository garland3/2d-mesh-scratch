// Rendering and visualization module
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