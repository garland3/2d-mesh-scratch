// UI Controls and parameter management
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