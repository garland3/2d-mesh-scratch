use crate::geometry::Point;
use crate::elements::{Triangle, Quad};
use crate::mesh::Mesh;

pub struct PavingMeshGenerator {
    points: Vec<Point>,
    quads: Vec<Quad>,
    triangles: Vec<Triangle>,
}

impl PavingMeshGenerator {
    pub fn new(boundary_points: Vec<Point>) -> Self {
        Self {
            points: boundary_points,
            quads: Vec::new(),
            triangles: Vec::new(),
        }
    }
    
    pub fn generate_mesh(&mut self, target_size: f64) -> Result<Mesh, String> {
        if self.points.len() < 4 {
            return Err("Need at least 4 points for paving mesh".to_string());
        }
        
        let bounds = self.calculate_bounds();
        let (min_x, max_x, min_y, max_y) = bounds;
        
        let grid_size = (target_size.sqrt() * 0.8).max(1.0);
        
        let mut grid_points = Vec::new();
        let mut y = min_y + grid_size;
        while y < max_y {
            let mut x = min_x + grid_size;
            while x < max_x {
                let point = Point::new(x, y);
                if self.is_point_inside_polygon(&point) {
                    grid_points.push(point);
                }
                x += grid_size;
            }
            y += grid_size;
        }
        
        let boundary_count = self.points.len();
        self.points.extend(grid_points);
        
        let cols = ((max_x - min_x) / grid_size).ceil() as usize;
        let rows = ((max_y - min_y) / grid_size).ceil() as usize;
        
        for row in 0..rows-1 {
            for col in 0..cols-1 {
                let base_idx = boundary_count + row * cols + col;
                if base_idx + cols + 1 < self.points.len() {
                    if self.point_exists_and_inside(base_idx) &&
                       self.point_exists_and_inside(base_idx + 1) &&
                       self.point_exists_and_inside(base_idx + cols) &&
                       self.point_exists_and_inside(base_idx + cols + 1) {
                        
                        let mut vertices = [
                            base_idx,
                            base_idx + 1,
                            base_idx + cols + 1,
                            base_idx + cols,
                        ];
                        
                        let quad = Quad::new(vertices);
                        
                        if quad.min_jacobian(&self.points) <= 0.0 {
                            vertices = [
                                base_idx,
                                base_idx + cols,
                                base_idx + cols + 1,
                                base_idx + 1,
                            ];
                        }
                        
                        let corrected_quad = Quad::new(vertices);
                        self.quads.push(corrected_quad);
                    }
                }
            }
        }
        
        self.fill_boundary_with_triangles(boundary_count);
        
        let mesh = Mesh::new_with_quads(self.points.clone(), self.triangles.clone(), self.quads.clone());
        
        if let Err(e) = mesh.validate_jacobians() {
            return Err(format!("Paving mesh validation failed: {}", e));
        }
        
        let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
        log::info!("Paving mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", min_jac, max_jac, avg_jac);
        
        Ok(mesh)
    }
    
    fn calculate_bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = std::f64::INFINITY;
        let mut max_x = std::f64::NEG_INFINITY;
        let mut min_y = std::f64::INFINITY;
        let mut max_y = std::f64::NEG_INFINITY;

        for point in &self.points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        (min_x, max_x, min_y, max_y)
    }
    
    fn is_point_inside_polygon(&self, point: &Point) -> bool {
        let mut inside = false;
        let boundary_count = self.points.len().min(self.points.len());
        let mut j = boundary_count - 1;

        for i in 0..boundary_count {
            if i >= self.points.len() || j >= self.points.len() {
                break;
            }
            let pi = &self.points[i];
            let pj = &self.points[j];
            
            if ((pi.y > point.y) != (pj.y > point.y)) &&
               (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
    
    fn point_exists_and_inside(&self, idx: usize) -> bool {
        idx < self.points.len() && 
        (idx < self.points.len() || self.is_point_inside_polygon(&self.points[idx]))
    }
    
    fn fill_boundary_with_triangles(&mut self, boundary_count: usize) {
        if boundary_count >= 3 {
            for i in 1..boundary_count-1 {
                let mut vertices = [0, i, i+1];
                let triangle = Triangle::new(vertices, &self.points);
                
                if triangle.jacobian(&self.points) < 0.0 {
                    vertices = [0, i+1, i];
                }
                
                let corrected_triangle = Triangle::new(vertices, &self.points);
                self.triangles.push(corrected_triangle);
            }
        }
    }
}