use std::f64;
use crate::geometry::Point;
use crate::elements::{Triangle, Edge};
use crate::mesh::Mesh;

pub struct DelaunayTriangulator {
    pub points: Vec<Point>,
    pub triangles: Vec<Triangle>,
    pub super_triangle_indices: [usize; 3],
}

impl DelaunayTriangulator {
    pub fn new(mut points: Vec<Point>) -> Self {
        let bounds = Self::calculate_bounds(&points);
        let super_triangle = Self::create_super_triangle(bounds);
        
        // Store original point count before adding super triangle
        let original_count = points.len();
        points.extend(super_triangle);
        
        let super_triangle_indices = [original_count, original_count + 1, original_count + 2];
        
        let mut triangulator = Self {
            points,
            triangles: Vec::new(),
            super_triangle_indices,
        };
        
        // Create super triangle with proper orientation
        let super_triangle = Triangle::new(super_triangle_indices, &triangulator.points);
        
        if super_triangle.jacobian(&triangulator.points) < 0.0 {
            triangulator.super_triangle_indices = [super_triangle_indices[2], super_triangle_indices[1], super_triangle_indices[0]];
        }
        
        triangulator.triangles.push(Triangle::new(triangulator.super_triangle_indices, &triangulator.points));
        
        triangulator
    }

    pub fn calculate_bounds(points: &[Point]) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for point in points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        (min_x, max_x, min_y, max_y)
    }

    fn create_super_triangle(bounds: (f64, f64, f64, f64)) -> Vec<Point> {
        let (min_x, max_x, min_y, max_y) = bounds;
        let dx = max_x - min_x;
        let dy = max_y - min_y;
        let delta_max = dx.max(dy);
        let mid_x = (min_x + max_x) / 2.0;
        let mid_y = (min_y + max_y) / 2.0;

        vec![
            Point::new(mid_x - 20.0 * delta_max, mid_y - delta_max),
            Point::new(mid_x, mid_y + 20.0 * delta_max),
            Point::new(mid_x + 20.0 * delta_max, mid_y - delta_max),
        ]
    }

    pub fn triangulate(&mut self) -> Result<Mesh, String> {
        let original_point_count = self.points.len() - 3;
        
        // Apply Bowyer-Watson algorithm for each point
        for i in 0..original_point_count {
            self.bowyer_watson_add_point(i)?;
        }

        self.remove_super_triangle();
        
        let vertices: Vec<Point> = self.points[..original_point_count].to_vec();
        let triangles = self.triangles.clone();
        
        Ok(Mesh::new(vertices, triangles))
    }

    // Proper Bowyer-Watson algorithm implementation
    pub fn bowyer_watson_add_point(&mut self, point_index: usize) -> Result<(), String> {
        let point = &self.points[point_index];
        let mut bad_triangles = Vec::new();

        // Find triangles whose circumcircle contains the point
        for (i, triangle) in self.triangles.iter().enumerate() {
            if triangle.contains_point_in_circumcircle(point) {
                bad_triangles.push(i);
            }
        }

        // Find boundary of polygonal hole
        let mut polygon_edges = Vec::new();
        for &bad_triangle_index in &bad_triangles {
            let triangle = &self.triangles[bad_triangle_index];
            for i in 0..3 {
                let edge = Edge::new(triangle.vertices[i], triangle.vertices[(i + 1) % 3]);
                
                // Check if this edge is shared with another bad triangle
                let mut is_shared = false;
                for &other_bad_triangle_index in &bad_triangles {
                    if other_bad_triangle_index == bad_triangle_index {
                        continue;
                    }
                    
                    let other_triangle = &self.triangles[other_bad_triangle_index];
                    if self.triangle_contains_edge(other_triangle, &edge) {
                        is_shared = true;
                        break;
                    }
                }
                
                // If edge is not shared, it's part of the polygon boundary
                if !is_shared {
                    polygon_edges.push(edge);
                }
            }
        }

        // Remove bad triangles (in reverse order to maintain indices)
        bad_triangles.sort_by(|a, b| b.cmp(a));
        for &index in &bad_triangles {
            self.triangles.remove(index);
        }

        // Add new triangles formed by connecting point to polygon
        for edge in polygon_edges {
            let vertices = [edge.vertices[0], edge.vertices[1], point_index];
            let new_triangle = Triangle::new(vertices, &self.points);
            
            // Ensure proper orientation
            if new_triangle.jacobian(&self.points) > 0.0 {
                self.triangles.push(new_triangle);
            } else {
                let corrected_vertices = [edge.vertices[1], edge.vertices[0], point_index];
                self.triangles.push(Triangle::new(corrected_vertices, &self.points));
            }
        }

        Ok(())
    }
    
    fn triangle_contains_edge(&self, triangle: &Triangle, edge: &Edge) -> bool {
        for i in 0..3 {
            let triangle_edge = Edge::new(triangle.vertices[i], triangle.vertices[(i + 1) % 3]);
            if triangle_edge == *edge {
                return true;
            }
        }
        false
    }
    
    // Keep the old add_point method for backward compatibility
    pub fn add_point(&mut self, point_index: usize) -> Result<(), String> {
        self.bowyer_watson_add_point(point_index)
    }

    pub fn remove_super_triangle(&mut self) {
        // Remove triangles that share vertices with super-triangle
        self.triangles.retain(|triangle| {
            !triangle.vertices.iter().any(|&v| {
                self.super_triangle_indices.contains(&v)
            })
        });
    }

    pub fn filter_outside_triangles(&mut self, boundary_count: usize) {
        let points = &self.points;
        self.triangles.retain(|triangle| {
            let centroid = Point::new(
                (points[triangle.vertices[0]].x + points[triangle.vertices[1]].x + points[triangle.vertices[2]].x) / 3.0,
                (points[triangle.vertices[0]].y + points[triangle.vertices[1]].y + points[triangle.vertices[2]].y) / 3.0,
            );
            
            let mut inside = false;
            let mut j = boundary_count - 1;

            for i in 0..boundary_count {
                let pi = &points[i];
                let pj = &points[j];
                
                if ((pi.y > centroid.y) != (pj.y > centroid.y)) &&
                   (centroid.x < (pj.x - pi.x) * (centroid.y - pi.y) / (pj.y - pi.y) + pi.x) {
                    inside = !inside;
                }
                j = i;
            }
            
            inside
        });
    }

    pub fn refine_interior(&mut self, max_area: f64, _min_angle: f64, boundary_count: usize) -> Result<(), String> {
        let mut iteration = 0;
        let max_iterations = 50;
        let max_points = 10000;

        while iteration < max_iterations && self.points.len() < max_points {
            let mut needs_refinement = false;
            let mut bad_triangles = Vec::new();

            for (i, triangle) in self.triangles.iter().enumerate() {
                let area = triangle.area(&self.points);
                if area > max_area {
                    let centroid = Point::new(
                        (self.points[triangle.vertices[0]].x + self.points[triangle.vertices[1]].x + self.points[triangle.vertices[2]].x) / 3.0,
                        (self.points[triangle.vertices[0]].y + self.points[triangle.vertices[1]].y + self.points[triangle.vertices[2]].y) / 3.0,
                    );
                    
                    if self.is_point_inside_polygon(&centroid, boundary_count) {
                        needs_refinement = true;
                        bad_triangles.push(i);
                    }
                }
            }

            if !needs_refinement || bad_triangles.is_empty() {
                break;
            }

            let triangles_to_refine = bad_triangles.into_iter().take(5).collect::<Vec<_>>();
            let mut new_points = Vec::new();

            for &triangle_index in &triangles_to_refine {
                if triangle_index < self.triangles.len() {
                    let triangle = &self.triangles[triangle_index];
                    let centroid = Point::new(
                        (self.points[triangle.vertices[0]].x + self.points[triangle.vertices[1]].x + self.points[triangle.vertices[2]].x) / 3.0,
                        (self.points[triangle.vertices[0]].y + self.points[triangle.vertices[1]].y + self.points[triangle.vertices[2]].y) / 3.0,
                    );
                    
                    if self.is_point_inside_polygon(&centroid, boundary_count) {
                        new_points.push(centroid);
                    }
                }
            }

            if new_points.is_empty() {
                break;
            }

            for new_point in new_points {
                if self.points.len() >= max_points {
                    break;
                }
                let point_index = self.points.len();
                self.points.push(new_point);
                self.add_point(point_index)?;
            }

            iteration += 1;
        }

        Ok(())
    }

    fn is_point_inside_polygon(&self, point: &Point, boundary_count: usize) -> bool {
        let mut inside = false;
        let mut j = boundary_count - 1;

        for i in 0..boundary_count {
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
    
    // Method 1: Hexagonal Grid Approach (from pseudocode)
    pub fn generate_hexagonal_grid(boundary: &[Point], target_edge_length: f64) -> Vec<Point> {
        let mut points = Vec::new();
        
        // Get bounding box of area
        let (min_x, max_x, min_y, max_y) = Self::calculate_bounds(boundary);
        
        // Generate hexagonal grid points
        let hex_height = target_edge_length * (3.0_f64).sqrt() / 2.0;
        let mut row = 0;
        let mut y = min_y;
        
        while y <= max_y {
            let x_offset = if row % 2 == 1 { target_edge_length / 2.0 } else { 0.0 };
            let mut col = 0;
            let mut x = min_x + x_offset;
            
            while x <= max_x {
                let point = Point::new(x, y);
                
                if Self::is_point_inside_boundary_static(&point, boundary) {
                    points.push(point);
                }
                
                col += 1;
                x = min_x + col as f64 * target_edge_length + x_offset;
            }
            
            row += 1;
            y = min_y + row as f64 * hex_height;
        }
        
        // Add boundary points
        let boundary_points = Self::sample_boundary_points(boundary, target_edge_length);
        points.extend(boundary_points);
        
        points
    }
    
    // Sample boundary points at regular intervals
    fn sample_boundary_points(boundary: &[Point], target_edge_length: f64) -> Vec<Point> {
        let mut boundary_points = Vec::new();
        
        for i in 0..boundary.len() {
            let next_i = (i + 1) % boundary.len();
            let p1 = &boundary[i];
            let p2 = &boundary[next_i];
            
            boundary_points.push(p1.clone());
            
            let edge_length = p1.distance_to(p2);
            if edge_length > target_edge_length {
                let num_subdivisions = (edge_length / target_edge_length).ceil() as usize;
                
                for j in 1..num_subdivisions {
                    let t = j as f64 / num_subdivisions as f64;
                    let new_point = Point::new(
                        p1.x + t * (p2.x - p1.x),
                        p1.y + t * (p2.y - p1.y),
                    );
                    boundary_points.push(new_point);
                }
            }
        }
        
        boundary_points
    }
    
    // Static version of is_point_inside_polygon for use with boundary arrays
    fn is_point_inside_boundary_static(point: &Point, boundary: &[Point]) -> bool {
        let mut inside = false;
        let mut j = boundary.len() - 1;

        for i in 0..boundary.len() {
            let pi = &boundary[i];
            let pj = &boundary[j];
            
            if ((pi.y > point.y) != (pj.y > point.y)) &&
               (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
    
    // Method for adaptive refinement (from pseudocode)
    pub fn generate_adaptive_triangulation(boundary: &[Point], target_area: f64, max_iterations: usize) -> Result<Vec<Triangle>, String> {
        // Start with coarse triangulation
        let mut points = Self::sample_boundary_points(boundary, (target_area * 4.0).sqrt());
        
        // Add a few initial interior points
        let (min_x, max_x, min_y, max_y) = Self::calculate_bounds(boundary);
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        
        // Add center point if inside
        let center = Point::new(center_x, center_y);
        if Self::is_point_inside_boundary_static(&center, boundary) {
            points.push(center);
        }
        
        // Add a few more strategic points
        for i in 1..4 {
            let offset = (max_x - min_x) / 4.0;
            let test_point = Point::new(center_x + (i as f64 - 2.0) * offset, center_y);
            if Self::is_point_inside_boundary_static(&test_point, boundary) {
                points.push(test_point);
            }
        }
        
        let mut triangulator = DelaunayTriangulator::new(points);
        let mut triangulation = triangulator.triangulate()?;
        
        for iteration in 0..max_iterations {
            let mut large_triangles = Vec::new();
            
            for (i, triangle) in triangulation.triangle_indices.iter().enumerate() {
                let triangle_obj = Triangle::new([triangle[0], triangle[1], triangle[2]], &triangulation.vertices);
                let area = triangle_obj.area(&triangulation.vertices);
                
                if area > target_area * 1.5 {
                    large_triangles.push(i);
                }
            }
            
            if large_triangles.is_empty() {
                break;
            }
            
            // Add points at centroids of large triangles
            let mut new_points = Vec::new();
            for &triangle_index in &large_triangles {
                if triangle_index < triangulation.triangle_indices.len() {
                    let triangle = &triangulation.triangle_indices[triangle_index];
                    let p1 = &triangulation.vertices[triangle[0]];
                    let p2 = &triangulation.vertices[triangle[1]];
                    let p3 = &triangulation.vertices[triangle[2]];
                    
                    let centroid = Point::new(
                        (p1.x + p2.x + p3.x) / 3.0,
                        (p1.y + p2.y + p3.y) / 3.0,
                    );
                    
                    if Self::is_point_inside_boundary_static(&centroid, boundary) {
                        new_points.push(centroid);
                    }
                }
            }
            
            if new_points.is_empty() {
                break;
            }
            
            // Add new points and retriangulate
            triangulator.points.extend(new_points);
            triangulation = triangulator.triangulate()?;
        }
        
        Ok(triangulator.triangles)
    }
}