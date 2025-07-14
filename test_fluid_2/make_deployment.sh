#!/bin/bash
set -e

echo "🚀 Building deployment for GitHub Pages..."

# Create dist directory
mkdir -p dist

# Build WASM module
echo "📦 Building WASM module..."
wasm-pack build --target web --out-dir pkg

# Copy WASM files to dist
echo "📋 Copying WASM files..."
cp -r pkg dist/

# Create combined JS file by concatenating modules
echo "🔧 Combining JavaScript modules..."
cat > dist/combined.js << 'EOF'
// Combined JavaScript modules for deployment

// Emitter management module
class EmitterManager {
EOF

# Append emitter module (excluding export line)
sed '1d' js/emitters.js | sed '$d' >> dist/combined.js

cat >> dist/combined.js << 'EOF'
}

// Rendering and visualization module  
class FluidRenderer {
EOF

# Append renderer module (excluding export line)
sed '1d' js/renderer.js | sed '$d' >> dist/combined.js

cat >> dist/combined.js << 'EOF'
}

// UI Controls and parameter management
class SimulationControls {
EOF

# Append controls module (excluding export line)
sed '1d' js/controls.js | sed '$d' >> dist/combined.js

cat >> dist/combined.js << 'EOF'
}

// Expose classes globally for non-module usage
window.EmitterManager = EmitterManager;
window.FluidRenderer = FluidRenderer;
window.SimulationControls = SimulationControls;
EOF

echo "🎨 Creating deployment HTML from example.html..."

# Start with example.html as base
cp example.html dist/index.html

# Replace module script with combined script and deployment-specific changes
echo "🔧 Applying deployment modifications..."

# Create a temporary sed script for all replacements
cat > temp_sed_script << 'SEDEOF'
# Replace module imports with combined script
s|<script type="module">|<script src="combined.js"></script>\
    <script type="module">|

# Remove ES6 import statements
/import.*from.*emitters\.js/d
/import.*from.*renderer\.js/d
/import.*from.*controls\.js/d

# Replace class instantiation to use global objects
s/new EmitterManager(/new window.EmitterManager(/g
s/new FluidRenderer(/new window.FluidRenderer(/g
s/new SimulationControls(/new window.SimulationControls(/g
SEDEOF

# Apply the modifications
sed -f temp_sed_script dist/index.html > dist/index_temp.html
mv dist/index_temp.html dist/index.html

# Add parameter slider setup if missing (inject before the closing script tag)
echo "📊 Adding parameter slider setup..."

# Create the parameter setup code
cat > temp_param_setup << 'PARAMEOF'

            // Setup parameter sliders
            document.querySelectorAll('.slider-container input[type="range"]').forEach(slider => {
                const valueElem = slider.parentElement.querySelector('.slider-label span:last-child');
                if (valueElem) {
                    valueElem.textContent = slider.value;
                }
                
                slider.addEventListener('input', function() {
                    if (valueElem) {
                        valueElem.textContent = this.value;
                    }
                    
                    if (simulator) {
                        const param = this.dataset.param;
                        const value = parseFloat(this.value);
                        
                        switch(param) {
                            case 'GRAVITY':
                                simulator.set_gravity(value);
                                break;
                            case 'VISCOSITY':
                                simulator.set_viscosity(value);
                                break;
                            case 'PRESSURE':
                                simulator.set_pressure_multiplier(value);
                                break;
                            case 'VELOCITY_DAMPING':
                                simulator.set_velocity_damping(value);
                                break;
                            case 'PARTICLE_COUNT':
                                // Handle particle count change
                                break;
                        }
                    }
                });
            });
PARAMEOF

# Insert the parameter setup before the closing script tag
sed -i '/^[[:space:]]*<\/script>$/i\
'"$(cat temp_param_setup | sed 's/$/\\/')" dist/index.html

# Clean up temporary files
rm temp_sed_script temp_param_setup

echo "✅ Deployment ready in dist/ directory!"
echo "📁 Files created:"
echo "   - dist/index.html (deployment version)"
echo "   - dist/combined.js (bundled modules)"
echo "   - dist/pkg/ (WASM files)"
echo ""
echo "🌐 To test locally: python -m http.server 8000 --directory dist"
echo "📤 Ready for GitHub Pages deployment!"