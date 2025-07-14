use js_sys::Math;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub density: f32,
    pub pressure: f32,
    pub fx: f32,
    pub fy: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: (Math::random() as f32 - 0.5) * 2.0,
            vy: (Math::random() as f32 - 0.5) * 2.0,
            density: 0.0,
            pressure: 0.0,
            fx: 0.0,
            fy: 0.0,
        }
    }
}

#[derive(Clone)]
pub struct Inlet {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Clone)]
pub struct DirectionalEmitter {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub angle: f32,      // Direction in radians
    pub velocity: f32,   // Initial velocity magnitude
    pub spread: f32,     // Cone angle in radians
}

#[derive(Clone)]
pub struct Outlet {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Clone)]
pub struct Wall {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Clone)]
pub struct VectorCell {
    pub vx: f32,
    pub vy: f32,
}

impl VectorCell {
    pub fn new() -> Self {
        Self { vx: 0.0, vy: 0.0 }
    }
}

pub struct SimParams {
    pub gravity: f32,
    pub viscosity: f32,
    pub particle_radius: f32,
    pub smoothing_radius: f32,
    pub stiffness: f32,
    pub rest_density: f32,
    pub time_step: f32,
    pub max_particles: usize,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            gravity: 0.0,
            viscosity: 0.02,
            particle_radius: 5.0,
            smoothing_radius: 25.0,
            stiffness: 0.5,
            rest_density: 4.0,
            time_step: 0.016,
            max_particles: 500,
        }
    }
}

pub struct FluidSimulation {
    pub particles: Vec<Particle>,
    pub inlets: Vec<Inlet>,
    pub directional_emitters: Vec<DirectionalEmitter>,
    pub outlets: Vec<Outlet>,
    pub walls: Vec<Wall>,
    pub vector_field: Vec<Vec<VectorCell>>,
    pub params: SimParams,
    pub width: f32,
    pub height: f32,
}

impl FluidSimulation {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            particles: Vec::new(),
            inlets: Vec::new(),
            directional_emitters: Vec::new(),
            outlets: Vec::new(),
            walls: Vec::new(),
            vector_field: Vec::new(),
            params: SimParams::default(),
            width,
            height,
        }
    }

    pub fn update(&mut self) {
        self.spawn_particles();
        self.calculate_density();
        self.calculate_forces();
        self.integrate();
        self.handle_boundaries();
        self.remove_outlet_particles();
    }

    fn spawn_particles(&mut self) {
        // Regular inlets
        for inlet in &self.inlets {
            if self.particles.len() < self.params.max_particles {
                for _ in 0..2 {
                    if self.particles.len() >= self.params.max_particles {
                        break;
                    }
                    let angle = Math::random() as f32 * PI * 2.0;
                    let radius = Math::random() as f32 * inlet.radius * 0.8;
                    let x = inlet.x + angle.cos() * radius;
                    let y = inlet.y + angle.sin() * radius;
                    self.particles.push(Particle::new(x, y));
                }
            }
        }
        
        // Directional emitters
        for emitter in &self.directional_emitters {
            if self.particles.len() < self.params.max_particles {
                for _ in 0..2 {
                    if self.particles.len() >= self.params.max_particles {
                        break;
                    }
                    
                    // Position within emitter radius
                    let spawn_angle = Math::random() as f32 * PI * 2.0;
                    let spawn_radius = Math::random() as f32 * emitter.radius * 0.8;
                    let x = emitter.x + spawn_angle.cos() * spawn_radius;
                    let y = emitter.y + spawn_angle.sin() * spawn_radius;
                    
                    // Direction with spread
                    let direction_variation = (Math::random() as f32 - 0.5) * emitter.spread;
                    let particle_angle = emitter.angle + direction_variation;
                    
                    let mut particle = Particle::new(x, y);
                    particle.vx = particle_angle.cos() * emitter.velocity;
                    particle.vy = particle_angle.sin() * emitter.velocity;
                    
                    self.particles.push(particle);
                }
            }
        }
    }

    fn calculate_density(&mut self) {
        for i in 0..self.particles.len() {
            let mut density = 0.0;
            let p = &self.particles[i];
            
            for other in &self.particles {
                let dx = other.x - p.x;
                let dy = other.y - p.y;
                let r2 = dx * dx + dy * dy;
                
                if r2 < self.params.smoothing_radius * self.params.smoothing_radius {
                    density += self.poly6_kernel(r2.sqrt(), self.params.smoothing_radius);
                }
            }
            
            self.particles[i].density = density;
            self.particles[i].pressure = self.params.stiffness * (density - self.params.rest_density);
        }
    }

    fn calculate_forces(&mut self) {
        for i in 0..self.particles.len() {
            let mut f_p_x = 0.0;
            let mut f_p_y = 0.0;
            let mut f_v_x = 0.0;
            let mut f_v_y = 0.0;
            
            let p = &self.particles[i];
            
            for (j, other) in self.particles.iter().enumerate() {
                if i == j {
                    continue;
                }
                
                let dx = other.x - p.x;
                let dy = other.y - p.y;
                let r = (dx * dx + dy * dy).sqrt();
                
                if r > 0.0 && r < self.params.smoothing_radius {
                    // Pressure force
                    let p_factor = (p.pressure + other.pressure) / (2.0 * other.density);
                    let grad = self.spiky_kernel_gradient(r, self.params.smoothing_radius);
                    f_p_x -= (dx / r) * p_factor * grad;
                    f_p_y -= (dy / r) * p_factor * grad;
                    
                    // Viscosity force
                    let lap = self.viscosity_kernel_laplacian(r, self.params.smoothing_radius);
                    f_v_x += self.params.viscosity * (other.vx - p.vx) * lap / other.density;
                    f_v_y += self.params.viscosity * (other.vy - p.vy) * lap / other.density;
                }
            }
            
            self.particles[i].fx = f_p_x + f_v_x;
            self.particles[i].fy = f_p_y + f_v_y + self.params.gravity;
        }
    }

    fn integrate(&mut self) {
        for particle in &mut self.particles {
            if particle.density > 0.0 {
                particle.vx += (particle.fx / particle.density) * self.params.time_step;
                particle.vy += (particle.fy / particle.density) * self.params.time_step;
                particle.x += particle.vx * self.params.time_step;
                particle.y += particle.vy * self.params.time_step;
            }
        }
    }

    fn handle_boundaries(&mut self) {
        let restitution = 0.5;
        
        for particle in &mut self.particles {
            // Canvas boundaries
            if particle.x < self.params.particle_radius {
                particle.x = self.params.particle_radius;
                particle.vx *= -restitution;
            } else if particle.x > self.width - self.params.particle_radius {
                particle.x = self.width - self.params.particle_radius;
                particle.vx *= -restitution;
            }
            
            if particle.y < self.params.particle_radius {
                particle.y = self.params.particle_radius;
                particle.vy *= -restitution;
            } else if particle.y > self.height - self.params.particle_radius {
                particle.y = self.height - self.params.particle_radius;
                particle.vy *= -restitution;
            }
            
            // Wall collisions
            for wall in &self.walls {
                let dx = wall.x2 - wall.x1;
                let dy = wall.y2 - wall.y1;
                let t = ((particle.x - wall.x1) * dx + (particle.y - wall.y1) * dy) / (dx * dx + dy * dy);
                let t_clamped = t.max(0.0).min(1.0);
                
                let closest_x = wall.x1 + t_clamped * dx;
                let closest_y = wall.y1 + t_clamped * dy;
                
                let dist_x = particle.x - closest_x;
                let dist_y = particle.y - closest_y;
                let dist2 = dist_x * dist_x + dist_y * dist_y;
                
                if dist2 < self.params.particle_radius * self.params.particle_radius {
                    let dist = dist2.sqrt();
                    let overlap = self.params.particle_radius - dist;
                    
                    if dist > 0.0 {
                        particle.x += (dist_x / dist) * overlap;
                        particle.y += (dist_y / dist) * overlap;
                        
                        let wall_normal_x = -dy;
                        let wall_normal_y = dx;
                        let len = (wall_normal_x * wall_normal_x + wall_normal_y * wall_normal_y).sqrt();
                        
                        if len > 0.0 {
                            let nx = wall_normal_x / len;
                            let ny = wall_normal_y / len;
                            let dot = particle.vx * nx + particle.vy * ny;
                            
                            particle.vx -= 2.0 * dot * nx * restitution;
                            particle.vy -= 2.0 * dot * ny * restitution;
                        }
                    }
                }
            }
        }
    }

    fn remove_outlet_particles(&mut self) {
        self.particles.retain(|particle| {
            for outlet in &self.outlets {
                let dx = particle.x - outlet.x;
                let dy = particle.y - outlet.y;
                if dx * dx + dy * dy < outlet.radius * outlet.radius {
                    return false;
                }
            }
            true
        });
    }

    pub fn add_inlet(&mut self, x: f32, y: f32, radius: f32) {
        self.inlets.push(Inlet { x, y, radius });
    }

    pub fn add_directional_emitter(&mut self, x: f32, y: f32, radius: f32, angle: f32, velocity: f32, spread: f32) {
        self.directional_emitters.push(DirectionalEmitter { x, y, radius, angle, velocity, spread });
    }

    pub fn add_outlet(&mut self, x: f32, y: f32, radius: f32) {
        self.outlets.push(Outlet { x, y, radius });
    }

    pub fn add_wall(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.walls.push(Wall { x1, y1, x2, y2 });
    }

    pub fn clear_environment(&mut self) {
        self.inlets.clear();
        self.directional_emitters.clear();
        self.outlets.clear();
        self.walls.clear();
    }

    pub fn reset(&mut self) {
        self.particles.clear();
        self.clear_environment();
        self.vector_field.clear();
    }

    pub fn calculate_vector_field(&mut self, grid_resolution: usize) {
        let rows = (self.height as usize / grid_resolution) + 1;
        let cols = (self.width as usize / grid_resolution) + 1;
        
        self.vector_field = vec![vec![VectorCell::new(); cols]; rows];
        
        let alpha = 0.1; // EMA smoothing factor
        
        // Create temporary field for this frame
        let mut temp_field = vec![vec![(0.0f32, 0.0f32, 0i32); cols]; rows];
        
        for particle in &self.particles {
            let grid_x = (particle.x / grid_resolution as f32) as usize;
            let grid_y = (particle.y / grid_resolution as f32) as usize;
            
            if grid_y < rows && grid_x < cols {
                temp_field[grid_y][grid_x].0 += particle.vx;
                temp_field[grid_y][grid_x].1 += particle.vy;
                temp_field[grid_y][grid_x].2 += 1;
            }
        }
        
        // Apply EMA smoothing
        for i in 0..rows {
            for j in 0..cols {
                let mut avg_vx = self.vector_field[i][j].vx;
                let mut avg_vy = self.vector_field[i][j].vy;
                
                if temp_field[i][j].2 > 0 {
                    avg_vx = temp_field[i][j].0 / temp_field[i][j].2 as f32;
                    avg_vy = temp_field[i][j].1 / temp_field[i][j].2 as f32;
                }
                
                self.vector_field[i][j].vx = avg_vx * alpha + self.vector_field[i][j].vx * (1.0 - alpha);
                self.vector_field[i][j].vy = avg_vy * alpha + self.vector_field[i][j].vy * (1.0 - alpha);
            }
        }
    }

    // SPH kernel functions
    fn poly6_kernel(&self, r: f32, h: f32) -> f32 {
        if r >= 0.0 && r <= h {
            let h2 = h * h;
            let r2 = r * r;
            315.0 / (64.0 * PI * h.powi(9)) * (h2 - r2).powi(3)
        } else {
            0.0
        }
    }

    fn spiky_kernel_gradient(&self, r: f32, h: f32) -> f32 {
        if r > 0.0 && r <= h {
            -45.0 / (PI * h.powi(6)) * (h - r).powi(2)
        } else {
            0.0
        }
    }

    fn viscosity_kernel_laplacian(&self, r: f32, h: f32) -> f32 {
        if r >= 0.0 && r <= h {
            45.0 / (PI * h.powi(6)) * (h - r)
        } else {
            0.0
        }
    }
}