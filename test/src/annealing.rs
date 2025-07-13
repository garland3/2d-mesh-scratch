use rand::Rng;
use crate::geometry::Point;
use crate::elements::Triangle;
use crate::mesh::Mesh;
use crate::delaunay::DelaunayTriangulator;

pub struct GridAnnealingMeshGenerator {
    boundary_points: Vec<Point>,
    internal_points: Vec<Point>,
    triangles: Vec<Triangle>,
    rng: rand::rngs::ThreadRng,
    quality_threshold: f64,
    temperature: f64,
    cooling_rate: f64,
}

impl GridAnnealingMeshGenerator {
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
    
    pub fn with_options(
        boundary_points: Vec<Point>, 
        temperature: f64, 
        cooling_rate: f64, 
        quality_threshold: f64
    ) -> Self {
        Self {
            boundary_points,
            internal_points: Vec::new(),
            triangles: Vec::new(),
            rng: rand::thread_rng(),
            quality_threshold,
            temperature,
            cooling_rate,
        }
    }
    
    pub fn generate_mesh(&mut self, target_area: f64) -> Result<Mesh, String> {
        self.generate_mesh_with_iterations(target_area, 10000)
    }
    
    pub fn generate_mesh_with_iterations(&mut self, target_area: f64, max_iterations: u32) -> Result<Mesh, String> {
        log::info!("ANNEALING - Starting simulated annealing mesh generation");
        
        self.refine_boundary_points(target_area)?;
        log::info!("ANNEALING - Refined boundary to {} points", self.boundary_points.len());
        
        self.generate_internal_grid(target_area)?;
        log::info!("ANNEALING - Generated internal grid with {} points", self.internal_points.len());
        
        self.create_initial_triangulation()?;
        log::info!("ANNEALING - Created initial triangulation with {} triangles", self.triangles.len());
        
        self.optimize_with_annealing(max_iterations)?;
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
    
    fn optimize_with_annealing(&mut self, max_iterations: u32) -> Result<(), String> {
        let mut iterations = 0;
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

    fn is_boundary_vertex(&self, vertex: &Point) -> bool {
        if self.boundary_points.is_empty() {
            return false;
        }
        
        let tolerance = 1e-6;
        
        // Check if this vertex is very close to any boundary point
        for boundary_point in &self.boundary_points {
            let distance_sq = (vertex.x - boundary_point.x).powi(2) + (vertex.y - boundary_point.y).powi(2);
            if distance_sq < tolerance {
                return true;
            }
        }
        
        false
    }
}

pub struct GeneralAnnealingOptimizer {
    pub temperature: f64,
    pub cooling_rate: f64,
    pub max_iterations: u32,
    pub check_volume: bool,
    pub check_aspect_ratio: bool,
    pub target_aspect_ratio: f64,
    pub volume_weight: f64,
    pub aspect_ratio_weight: f64,
    pub check_size_uniformity: bool,
    pub size_uniformity_weight: f64,
    pub target_area: f64,
    pub min_area: f64,
    pub boundary_points: Vec<Point>, // Store original boundary for validation
    rng: rand::rngs::ThreadRng,
}

impl GeneralAnnealingOptimizer {
    pub fn new() -> Self {
        Self {
            temperature: 1000.0,
            cooling_rate: 0.995,
            max_iterations: 10000,
            check_volume: true,
            check_aspect_ratio: true,
            target_aspect_ratio: 1.73, // Ideal equilateral triangle ratio
            volume_weight: 0.3,
            aspect_ratio_weight: 0.4,
            check_size_uniformity: true,
            size_uniformity_weight: 0.3,
            target_area: 0.1,
            min_area: 0.01,
            boundary_points: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn from_options(options: &crate::geometry::AnnealingOptions) -> Self {
        Self {
            temperature: options.temperature.unwrap_or(1000.0),
            cooling_rate: options.cooling_rate.unwrap_or(0.995),
            max_iterations: options.max_iterations.unwrap_or(10000),
            check_volume: options.check_volume.unwrap_or(true),
            check_aspect_ratio: options.check_aspect_ratio.unwrap_or(true),
            target_aspect_ratio: options.target_aspect_ratio.unwrap_or(1.73),
            volume_weight: options.volume_weight.unwrap_or(0.3),
            aspect_ratio_weight: options.aspect_ratio_weight.unwrap_or(0.4),
            check_size_uniformity: options.check_size_uniformity.unwrap_or(true),
            size_uniformity_weight: options.size_uniformity_weight.unwrap_or(0.3),
            target_area: options.target_area.unwrap_or(0.1),
            min_area: options.min_area.unwrap_or(0.01),
            boundary_points: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn set_boundary(&mut self, boundary_points: Vec<Point>) {
        self.boundary_points = boundary_points;
    }

    pub fn optimize_mesh(&mut self, mesh: &mut Mesh) -> Result<(), String> {
        log::info!("GENERAL ANNEALING - Starting optimization");
        
        // Check if mesh has quads - if so, skip optimization
        if mesh.quads.is_some() && !mesh.quads.as_ref().unwrap().is_empty() {
            log::info!("GENERAL ANNEALING - Skipping optimization for quad-based mesh (not supported)");
            return Ok(());
        }
        
        // Ensure we have triangular elements to work with
        if mesh.triangles.is_empty() {
            log::info!("GENERAL ANNEALING - No triangular elements found, skipping optimization");
            return Ok(());
        }
        
        let mut temperature = self.temperature;
        let mut iterations = 0;
        
        let boundary_count = self.count_boundary_vertices(&mesh.vertices);
        
        while iterations < self.max_iterations && temperature > 0.1 {
            let current_quality = self.calculate_enhanced_quality(mesh);
            
            if !mesh.vertices.is_empty() && mesh.vertices.len() > boundary_count {
                let vertex_idx = self.rng.gen_range(boundary_count..mesh.vertices.len());
                let old_vertex = mesh.vertices[vertex_idx].clone();
                
                let perturbation_radius = temperature * 0.05;
                let dx = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                let dy = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                
                let new_vertex = Point::new(old_vertex.x + dx, old_vertex.y + dy);
                
                // Only move the vertex if it stays inside the boundary and is not a boundary vertex
                if self.is_point_inside_boundary(&new_vertex) && !self.is_boundary_vertex(&old_vertex) {
                    mesh.vertices[vertex_idx] = new_vertex;
                    
                    self.update_triangles_after_vertex_move(mesh, vertex_idx);
                    
                    let new_quality = self.calculate_enhanced_quality(mesh);
                    let quality_improvement = new_quality - current_quality;
                    
                    if quality_improvement > 0.0 || 
                       self.rng.gen::<f64>() < (quality_improvement / temperature).exp() {
                        if iterations % 1000 == 0 {
                            log::info!("GENERAL ANNEALING - Iteration {}: quality={:.4}, temp={:.2}", 
                                      iterations, new_quality, temperature);
                        }
                    } else {
                        mesh.vertices[vertex_idx] = old_vertex;
                        self.update_triangles_after_vertex_move(mesh, vertex_idx);
                    }
                } else {
                    // Skip this iteration if the move would be outside the boundary
                    if iterations % 5000 == 0 {
                        log::info!("GENERAL ANNEALING - Skipped move outside boundary at iteration {}", iterations);
                    }
                }
            }
            
            temperature *= self.cooling_rate;
            iterations += 1;
        }
        
        log::info!("GENERAL ANNEALING - Optimization finished after {} iterations", iterations);
        
        // Phase 2: Adaptive refinement for size control
        self.adaptive_size_refinement(mesh)?;
        
        Ok(())
    }

    fn adaptive_size_refinement(&mut self, mesh: &mut Mesh) -> Result<(), String> {
        log::info!("ADAPTIVE REFINEMENT - Starting size-based refinement");
        
        let max_refinement_iterations = 3;
        for iteration in 0..max_refinement_iterations {
            let mut changed = false;
            
            // Find triangles that need refinement
            let oversized_triangles = self.find_oversized_triangles(mesh);
            let undersized_triangles = self.find_undersized_triangles(mesh);
            
            log::info!("ADAPTIVE REFINEMENT - Iteration {}: {} oversized, {} undersized triangles", 
                      iteration, oversized_triangles.len(), undersized_triangles.len());
            
            // Split large triangles by adding vertices at centroids
            for triangle_idx in oversized_triangles {
                if self.split_triangle(mesh, triangle_idx)? {
                    changed = true;
                }
            }
            
            // Handle small triangles by vertex relocation or removal
            for triangle_idx in undersized_triangles {
                if self.handle_small_triangle(mesh, triangle_idx)? {
                    changed = true;
                }
            }
            
            if !changed {
                log::info!("ADAPTIVE REFINEMENT - Converged after {} iterations", iteration + 1);
                break;
            }
            
            // Re-triangulate to maintain valid mesh
            self.retriangulate_mesh(mesh)?;
        }
        
        log::info!("ADAPTIVE REFINEMENT - Completed");
        Ok(())
    }

    fn find_oversized_triangles(&self, mesh: &Mesh) -> Vec<usize> {
        let max_allowed_area = self.target_area * 2.0; // Allow some tolerance
        
        mesh.triangle_indices.iter().enumerate()
            .filter_map(|(i, &vertices)| {
                let triangle = Triangle::new(vertices, &mesh.vertices);
                let area = triangle.volume(&mesh.vertices);
                if area > max_allowed_area {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn find_undersized_triangles(&self, mesh: &Mesh) -> Vec<usize> {
        mesh.triangle_indices.iter().enumerate()
            .filter_map(|(i, &vertices)| {
                let triangle = Triangle::new(vertices, &mesh.vertices);
                let area = triangle.volume(&mesh.vertices);
                if area < self.min_area {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn split_triangle(&mut self, mesh: &mut Mesh, triangle_idx: usize) -> Result<bool, String> {
        if triangle_idx >= mesh.triangle_indices.len() {
            return Ok(false);
        }
        
        let triangle_vertices = mesh.triangle_indices[triangle_idx];
        let _triangle = Triangle::new(triangle_vertices, &mesh.vertices);
        
        // Calculate centroid
        let p1 = &mesh.vertices[triangle_vertices[0]];
        let p2 = &mesh.vertices[triangle_vertices[1]];
        let p3 = &mesh.vertices[triangle_vertices[2]];
        
        let centroid = Point::new(
            (p1.x + p2.x + p3.x) / 3.0,
            (p1.y + p2.y + p3.y) / 3.0
        );
        
        // Only add centroid if it's inside the boundary
        if self.is_point_inside_boundary(&centroid) {
            mesh.vertices.push(centroid);
            let _centroid_idx = mesh.vertices.len() - 1;
        } else {
            log::info!("ADAPTIVE REFINEMENT - Skipped adding centroid outside boundary for triangle {}", triangle_idx);
            return Ok(false);
        }
        
        // Mark for retriangulation (we'll handle this in retriangulate_mesh)
        log::info!("ADAPTIVE REFINEMENT - Split triangle {} by adding vertex at centroid", triangle_idx);
        
        Ok(true)
    }

    fn handle_small_triangle(&mut self, mesh: &mut Mesh, triangle_idx: usize) -> Result<bool, String> {
        if triangle_idx >= mesh.triangle_indices.len() {
            return Ok(false);
        }
        
        let triangle_vertices = mesh.triangle_indices[triangle_idx];
        
        // Try to relocate the vertex that creates the smallest triangle
        // For now, we'll use a simple approach: move vertices slightly toward centroid
        let _triangle = Triangle::new(triangle_vertices, &mesh.vertices);
        
        // Calculate triangle centroid
        let p1 = &mesh.vertices[triangle_vertices[0]];
        let p2 = &mesh.vertices[triangle_vertices[1]];
        let p3 = &mesh.vertices[triangle_vertices[2]];
        
        let centroid_x = (p1.x + p2.x + p3.x) / 3.0;
        let centroid_y = (p1.y + p2.y + p3.y) / 3.0;
        
        // Move each vertex slightly toward the centroid (but not too much)
        let move_factor = 0.1; // Move 10% toward centroid
        
        for &vertex_idx in &triangle_vertices {
            if vertex_idx < mesh.vertices.len() {
                let vertex = &mesh.vertices[vertex_idx];
                let new_x = vertex.x + (centroid_x - vertex.x) * move_factor;
                let new_y = vertex.y + (centroid_y - vertex.y) * move_factor;
                let new_point = Point::new(new_x, new_y);
                
                // Only move vertex if it stays inside the boundary
                if self.is_point_inside_boundary(&new_point) {
                    mesh.vertices[vertex_idx] = new_point;
                } else {
                    log::info!("ADAPTIVE REFINEMENT - Skipped moving vertex {} outside boundary", vertex_idx);
                }
            }
        }
        
        log::info!("ADAPTIVE REFINEMENT - Adjusted small triangle {} by moving vertices toward centroid", triangle_idx);
        
        Ok(true)
    }

    fn retriangulate_mesh(&self, mesh: &mut Mesh) -> Result<(), String> {
        // Use Delaunay triangulation to rebuild the mesh with all vertices
        let mut triangulator = DelaunayTriangulator::new(mesh.vertices.clone());
        let new_mesh = triangulator.triangulate()?;
        
        // Update the mesh
        mesh.vertices = new_mesh.vertices;
        mesh.triangle_indices = new_mesh.triangle_indices;
        mesh.triangles = new_mesh.triangles;
        
        log::info!("ADAPTIVE REFINEMENT - Retriangulated mesh: {} vertices, {} triangles", 
                  mesh.vertices.len(), mesh.triangles.len());
        
        Ok(())
    }

    fn calculate_enhanced_quality(&self, mesh: &Mesh) -> f64 {
        if mesh.triangles.is_empty() {
            return 0.0;
        }

        let mut total_quality = 0.0;
        let mut valid_triangles = 0;

        for (i, triangle_vertices) in mesh.triangle_indices.iter().enumerate() {
            let triangle = Triangle::new(*triangle_vertices, &mesh.vertices);
            
            let jacobian = triangle.jacobian(&mesh.vertices);
            if jacobian > 0.0 {
                let mut quality_score = 0.0;
                let mut weight_sum = 0.0;

                // Basic angle quality
                let min_angle = triangle.min_angle(&mesh.vertices);
                let angle_quality = min_angle / 60.0;
                quality_score += angle_quality * 0.5;
                weight_sum += 0.5;

                // Volume uniformity check
                if self.check_volume {
                    let volume = triangle.volume(&mesh.vertices);
                    let volume_quality = self.calculate_volume_quality(volume, mesh);
                    quality_score += volume_quality * self.volume_weight;
                    weight_sum += self.volume_weight;
                }

                // Aspect ratio check
                if self.check_aspect_ratio {
                    let aspect_ratio = triangle.aspect_ratio(&mesh.vertices);
                    let aspect_quality = self.calculate_aspect_ratio_quality(aspect_ratio);
                    quality_score += aspect_quality * self.aspect_ratio_weight;
                    weight_sum += self.aspect_ratio_weight;
                }

                // Size uniformity check
                if self.check_size_uniformity {
                    let volume = triangle.volume(&mesh.vertices);
                    let size_quality = self.calculate_size_uniformity_quality(volume);
                    quality_score += size_quality * self.size_uniformity_weight;
                    weight_sum += self.size_uniformity_weight;
                }

                if weight_sum > 0.0 {
                    total_quality += quality_score / weight_sum;
                    valid_triangles += 1;
                }
            }
        }

        if valid_triangles > 0 {
            total_quality / valid_triangles as f64
        } else {
            0.0
        }
    }

    fn calculate_volume_quality(&self, volume: f64, mesh: &Mesh) -> f64 {
        if mesh.triangle_indices.is_empty() {
            return 1.0;
        }

        // Calculate average volume for comparison
        let total_volume: f64 = mesh.triangle_indices.iter()
            .map(|&vertices| Triangle::new(vertices, &mesh.vertices).volume(&mesh.vertices))
            .sum();
        let avg_volume = total_volume / mesh.triangle_indices.len() as f64;

        if avg_volume == 0.0 {
            return 1.0;
        }

        // Quality decreases as volume deviates from average
        let volume_ratio = if volume > avg_volume {
            avg_volume / volume
        } else {
            volume / avg_volume
        };

        volume_ratio.max(0.1) // Minimum quality of 0.1
    }

    fn calculate_aspect_ratio_quality(&self, aspect_ratio: f64) -> f64 {
        // Quality decreases as aspect ratio deviates from target
        let ratio_diff = (aspect_ratio - self.target_aspect_ratio).abs();
        let normalized_diff = ratio_diff / self.target_aspect_ratio;
        
        (1.0 - normalized_diff.min(1.0)).max(0.1) // Minimum quality of 0.1
    }

    fn calculate_size_uniformity_quality(&self, area: f64) -> f64 {
        // Quality is highest when area matches target_area
        // Penalize both oversized and undersized triangles
        let size_ratio = if area > self.target_area {
            self.target_area / area
        } else if area < self.min_area {
            // Heavy penalty for very small triangles
            area / self.min_area * 0.5
        } else {
            area / self.target_area
        };
        
        size_ratio.max(0.05) // Minimum quality of 0.05 for very poor size matches
    }

    fn count_boundary_vertices(&self, vertices: &[Point]) -> usize {
        if self.boundary_points.is_empty() {
            return 0; // No boundary constraints
        }
        
        let tolerance = 1e-6;
        let mut boundary_vertex_count = 0;
        
        // Count vertices that are very close to any boundary point
        for vertex in vertices {
            for boundary_point in &self.boundary_points {
                let distance_sq = (vertex.x - boundary_point.x).powi(2) + (vertex.y - boundary_point.y).powi(2);
                if distance_sq < tolerance {
                    boundary_vertex_count += 1;
                    break; // Found match, no need to check other boundary points
                }
            }
        }
        
        // If we didn't find enough matches, be conservative and protect more vertices
        if boundary_vertex_count < self.boundary_points.len() {
            let conservative_estimate = self.boundary_points.len().min(vertices.len() / 3);
            log::info!("BOUNDARY DETECTION - Found {} exact matches, using conservative estimate of {}", 
                      boundary_vertex_count, conservative_estimate);
            return conservative_estimate;
        }
        
        log::info!("BOUNDARY DETECTION - Identified {} boundary vertices out of {} total", 
                  boundary_vertex_count, vertices.len());
        boundary_vertex_count
    }

    fn update_triangles_after_vertex_move(&self, mesh: &mut Mesh, moved_vertex_idx: usize) {
        // Update triangles that contain the moved vertex
        for (_i, triangle_vertices) in mesh.triangle_indices.iter().enumerate() {
            if triangle_vertices.contains(&moved_vertex_idx) {
                let triangle_points = vec![
                    mesh.vertices[triangle_vertices[0]].clone(),
                    mesh.vertices[triangle_vertices[1]].clone(),
                    mesh.vertices[triangle_vertices[2]].clone(),
                ];
                mesh.triangles[_i] = triangle_points;
            }
        }
    }

    fn is_point_inside_boundary(&self, point: &Point) -> bool {
        // If no boundary points are set, allow all moves (fallback behavior)
        if self.boundary_points.is_empty() {
            return true;
        }
        
        // Use ray casting algorithm to check if point is inside polygon
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

    fn is_boundary_vertex(&self, vertex: &Point) -> bool {
        if self.boundary_points.is_empty() {
            return false;
        }
        
        let tolerance = 1e-6;
        
        // Check if this vertex is very close to any boundary point
        for boundary_point in &self.boundary_points {
            let distance_sq = (vertex.x - boundary_point.x).powi(2) + (vertex.y - boundary_point.y).powi(2);
            if distance_sq < tolerance {
                return true;
            }
        }
        
        false
    }
}