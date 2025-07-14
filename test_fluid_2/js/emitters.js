// Emitter management module
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