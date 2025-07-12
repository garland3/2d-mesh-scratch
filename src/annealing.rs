use rand::Rng;
use crate::geometry::Point;
use crate::elements::Triangle;
use crate::mesh::Mesh;
use crate::delaunay::DelaunayTriangulator;

pub struct SimulatedAnnealingMeshGenerator {
    boundary_points: Vec<Point>,
    internal_points: Vec<Point>,
    triangles: Vec<Triangle>,
    rng: rand::rngs::ThreadRng,
    quality_threshold: f64,
    temperature: f64,
    cooling_rate: f64,
}

impl SimulatedAnnealingMeshGenerator {
    pub fn new(boundary_points: Vec<Point>, quality_threshold: f64) -> Self {
        Self {
            boundary_points,
            internal_points: Vec::new(),
            triangles: Vec::new(),
            rng: rand::thread_rng(),
            quality_threshold,
            temperature: 1000.0,
            cooling_rate: 0.995,
        }
    }
    
    pub fn generate_mesh(&mut self, target_area: f64) -> Result<Mesh, String> {
        log::info!("ANNEALING - Starting simulated annealing mesh generation");
        
        self.refine_boundary_points(target_area)?;
        log::info!("ANNEALING - Refined boundary to {} points", self.boundary_points.len());
        
        self.generate_internal_grid(target_area)?;
        log::info!("ANNEALING - Generated internal grid with {} points", self.internal_points.len());
        
        self.create_initial_triangulation()?;
        log::info!("ANNEALING - Created initial triangulation with {} triangles", self.triangles.len());
        
        self.optimize_with_annealing()?;
        log::info!("ANNEALING - Optimization complete");
        
        self.create_final_mesh()
    }
    
    fn refine_boundary_points(&mut self, target_area: f64) -> Result<(), String> {
        let target_edge_length = (4.0 * target_area / 3.0_f64.sqrt()).sqrt();
        let mut refined_points = Vec::new();
        
        for i in 0..self.boundary_points.len() {
            let next_i = (i + 1) % self.boundary_points.len();
            let p1 = &self.boundary_points[i];
            let p2 = &self.boundary_points[next_i];
            
            refined_points.push(p1.clone());
            
            let edge_length = p1.distance_to(p2);
            
            if edge_length > target_edge_length {
                let num_subdivisions = (edge_length / target_edge_length).ceil() as usize;
                
                for j in 1..num_subdivisions {
                    let t = j as f64 / num_subdivisions as f64;
                    let new_point = Point::new(
                        p1.x + t * (p2.x - p1.x),
                        p1.y + t * (p2.y - p1.y),
                    );
                    refined_points.push(new_point);
                }
            }
        }
        
        self.boundary_points = refined_points;
        Ok(())
    }
    
    fn generate_internal_grid(&mut self, target_area: f64) -> Result<(), String> {
        let bounds = self.calculate_bounds();
        let (min_x, max_x, min_y, max_y) = bounds;
        
        let grid_spacing = (target_area.sqrt() * 0.7).max(1.0);
        
        let mut y = min_y + grid_spacing;
        while y < max_y {
            let mut x = min_x + grid_spacing;
            while x < max_x {
                let point = Point::new(x, y);
                if self.is_point_inside_polygon(&point) {
                    self.internal_points.push(point);
                }
                x += grid_spacing;
            }
            y += grid_spacing;
        }
        
        Ok(())
    }
    
    fn create_initial_triangulation(&mut self) -> Result<(), String> {
        let mut all_points = self.boundary_points.clone();
        all_points.extend(self.internal_points.clone());
        
        let mut triangulator = DelaunayTriangulator::new(all_points);
        let mesh = triangulator.triangulate()?;
        
        self.triangles = mesh.triangle_indices.iter().map(|&vertices| {
            Triangle::new(vertices, &mesh.vertices)
        }).collect();
        
        let boundary_count = self.boundary_points.len();
        self.boundary_points = mesh.vertices[..boundary_count].to_vec();
        self.internal_points = mesh.vertices[boundary_count..].to_vec();
        
        Ok(())
    }
    
    fn optimize_with_annealing(&mut self) -> Result<(), String> {
        let mut iterations = 0;
        let max_iterations = 10000;
        let mut temperature = self.temperature;
        
        log::info!("ANNEALING - Starting optimization with temperature: {:.2}", temperature);
        
        while iterations < max_iterations && temperature > 0.1 {
            let current_quality = self.calculate_mesh_quality();
            
            if current_quality > self.quality_threshold {
                log::info!("ANNEALING - Quality threshold reached: {:.4}", current_quality);
                break;
            }
            
            if !self.internal_points.is_empty() {
                let point_idx = self.rng.gen_range(0..self.internal_points.len());
                let old_point = self.internal_points[point_idx].clone();
                
                let perturbation_radius = temperature * 0.1;
                let dx = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                let dy = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                
                let new_point = Point::new(old_point.x + dx, old_point.y + dy);
                
                if self.is_point_inside_polygon(&new_point) {
                    self.internal_points[point_idx] = new_point;
                    
                    self.update_triangulation_after_move(point_idx + self.boundary_points.len())?;
                    
                    let new_quality = self.calculate_mesh_quality();
                    let quality_improvement = new_quality - current_quality;
                    
                    if quality_improvement > 0.0 || 
                       self.rng.gen::<f64>() < (quality_improvement / temperature).exp() {
                        if iterations % 1000 == 0 {
                            log::info!("ANNEALING - Iteration {}: quality={:.4}, temp={:.2}", 
                                      iterations, new_quality, temperature);
                        }
                    } else {
                        self.internal_points[point_idx] = old_point;
                        self.update_triangulation_after_move(point_idx + self.boundary_points.len())?;
                    }
                }
            }
            
            temperature *= self.cooling_rate;
            iterations += 1;
        }
        
        log::info!("ANNEALING - Optimization finished after {} iterations", iterations);
        Ok(())
    }
    
    fn calculate_mesh_quality(&self) -> f64 {
        if self.triangles.is_empty() {
            return 0.0;
        }
        
        let all_points: Vec<Point> = self.boundary_points.iter()
            .chain(self.internal_points.iter())
            .cloned()
            .collect();
        
        let mut total_quality = 0.0;
        let mut valid_triangles = 0;
        
        for triangle in &self.triangles {
            let min_angle = triangle.min_angle(&all_points);
            let jacobian = triangle.jacobian(&all_points);
            
            if jacobian > 0.0 {
                let angle_quality = min_angle / 60.0;
                let jacobian_quality = jacobian.min(1.0);
                total_quality += angle_quality * jacobian_quality;
                valid_triangles += 1;
            }
        }
        
        if valid_triangles > 0 {
            total_quality / valid_triangles as f64
        } else {
            0.0
        }
    }
    
    fn update_triangulation_after_move(&mut self, _moved_point_idx: usize) -> Result<(), String> {
        let mut all_points = self.boundary_points.clone();
        all_points.extend(self.internal_points.clone());
        
        let mut triangulator = DelaunayTriangulator::new(all_points);
        let mesh = triangulator.triangulate()?;
        
        self.triangles = mesh.triangle_indices.iter().map(|&vertices| {
            Triangle::new(vertices, &mesh.vertices)
        }).collect();
        
        Ok(())
    }
    
    fn create_final_mesh(&self) -> Result<Mesh, String> {
        let mut all_points = self.boundary_points.clone();
        all_points.extend(self.internal_points.clone());
        
        let triangle_indices: Vec<[usize; 3]> = self.triangles.iter()
            .map(|t| t.vertices)
            .collect();
        
        let triangle_points: Vec<Vec<Point>> = self.triangles.iter()
            .map(|t| vec![
                all_points[t.vertices[0]].clone(),
                all_points[t.vertices[1]].clone(),
                all_points[t.vertices[2]].clone(),
            ])
            .collect();
        
        let mesh = Mesh {
            vertices: all_points,
            triangles: triangle_points,
            triangle_indices,
            quads: None,
            quad_indices: None,
        };
        
        if let Err(e) = mesh.validate_jacobians() {
            return Err(format!("Annealing mesh validation failed: {}", e));
        }
        
        let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
        log::info!("Annealing mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", 
                  min_jac, max_jac, avg_jac);
        
        Ok(mesh)
    }
    
    fn calculate_bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = std::f64::INFINITY;
        let mut max_x = std::f64::NEG_INFINITY;
        let mut min_y = std::f64::INFINITY;
        let mut max_y = std::f64::NEG_INFINITY;

        for point in &self.boundary_points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        (min_x, max_x, min_y, max_y)
    }
    
    fn is_point_inside_polygon(&self, point: &Point) -> bool {
        let mut inside = false;
        let boundary_count = self.boundary_points.len();
        let mut j = boundary_count - 1;

        for i in 0..boundary_count {
            let pi = &self.boundary_points[i];
            let pj = &self.boundary_points[j];
            
            if ((pi.y > point.y) != (pj.y > point.y)) &&
               (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
}