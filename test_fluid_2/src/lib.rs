use wasm_bindgen::prelude::*;
use js_sys::Math;

mod fluid;
use fluid::{FluidSimulation, SimParams};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct FluidSimulator {
    simulation: FluidSimulation,
}

#[wasm_bindgen]
impl FluidSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            simulation: FluidSimulation::new(width, height),
        }
    }

    #[wasm_bindgen]
    pub fn update(&mut self) {
        self.simulation.update();
    }

    #[wasm_bindgen]
    pub fn add_inlet(&mut self, x: f32, y: f32, radius: f32) {
        self.simulation.add_inlet(x, y, radius);
    }

    #[wasm_bindgen]
    pub fn add_directional_emitter(&mut self, x: f32, y: f32, radius: f32, angle: f32, velocity: f32, spread: f32) {
        self.simulation.add_directional_emitter(x, y, radius, angle, velocity, spread);
    }

    #[wasm_bindgen]
    pub fn add_outlet(&mut self, x: f32, y: f32, radius: f32) {
        self.simulation.add_outlet(x, y, radius);
    }

    #[wasm_bindgen]
    pub fn add_wall(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.simulation.add_wall(x1, y1, x2, y2);
    }

    #[wasm_bindgen]
    pub fn clear_environment(&mut self) {
        self.simulation.clear_environment();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    #[wasm_bindgen]
    pub fn set_gravity(&mut self, gravity: f32) {
        self.simulation.params.gravity = gravity;
    }

    #[wasm_bindgen]
    pub fn set_viscosity(&mut self, viscosity: f32) {
        self.simulation.params.viscosity = viscosity;
    }

    #[wasm_bindgen]
    pub fn set_particle_radius(&mut self, radius: f32) {
        self.simulation.params.particle_radius = radius;
        self.simulation.params.smoothing_radius = radius * 5.0;
    }

    #[wasm_bindgen]
    pub fn set_stiffness(&mut self, stiffness: f32) {
        self.simulation.params.stiffness = stiffness;
    }

    #[wasm_bindgen]
    pub fn set_rest_density(&mut self, density: f32) {
        self.simulation.params.rest_density = density;
    }

    #[wasm_bindgen]
    pub fn set_max_particles(&mut self, max: usize) {
        self.simulation.params.max_particles = max;
    }

    #[wasm_bindgen]
    pub fn set_time_step(&mut self, time_step: f32) {
        self.simulation.params.time_step = time_step;
    }

    #[wasm_bindgen]
    pub fn get_particle_positions(&self) -> Vec<f32> {
        let mut positions = Vec::new();
        for particle in &self.simulation.particles {
            positions.push(particle.x);
            positions.push(particle.y);
        }
        positions
    }

    #[wasm_bindgen]
    pub fn get_particle_velocities(&self) -> Vec<f32> {
        let mut velocities = Vec::new();
        for particle in &self.simulation.particles {
            velocities.push(particle.vx);
            velocities.push(particle.vy);
        }
        velocities
    }

    #[wasm_bindgen]
    pub fn get_vector_field(&mut self, grid_resolution: i32) -> Vec<f32> {
        self.simulation.calculate_vector_field(grid_resolution as usize);
        let mut field_data = Vec::new();
        
        for row in &self.simulation.vector_field {
            for cell in row {
                field_data.push(cell.vx);
                field_data.push(cell.vy);
            }
        }
        field_data
    }

    #[wasm_bindgen]
    pub fn get_particle_count(&self) -> usize {
        self.simulation.particles.len()
    }

    #[wasm_bindgen]
    pub fn get_particle_radius(&self) -> f32 {
        self.simulation.params.particle_radius
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Fluid simulator loaded!");
}