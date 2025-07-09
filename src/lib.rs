use serde::{Deserialize, Serialize};
use std::f64;
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub id: Option<String>,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, id: None }
    }

    pub fn with_id(x: f64, y: f64, id: String) -> Self {
        Self { x, y, id: Some(id) }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn distance_squared_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geometry {
    pub points: Vec<Point>,
    pub name: Option<String>,
}

impl Geometry {
    pub fn new(points: Vec<Point>) -> Self {
        Self { points, name: None }
    }

    pub fn with_name(points: Vec<Point>, name: String) -> Self {
        Self { points, name: Some(name) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRequest {
    pub geometry: Geometry,
    pub max_area: Option<f64>,
    pub min_angle: Option<f64>,
    pub algorithm: Option<String>,
}

impl MeshRequest {
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            max_area: Some(0.1),
            min_angle: Some(20.0),
            algorithm: Some("delaunay".to_string()),
        }
    }

    pub fn with_constraints(geometry: Geometry, max_area: f64, min_angle: f64) -> Self {
        Self {
            geometry,
            max_area: Some(max_area),
            min_angle: Some(min_angle),
            algorithm: Some("delaunay".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [usize; 3],
    pub circumcenter: Point,
    pub circumradius_squared: f64,
}

impl Triangle {
    pub fn new(vertices: [usize; 3], points: &[Point]) -> Self {
        let (circumcenter, circumradius_squared) = Self::calculate_circumcircle(vertices, points);
        Self {
            vertices,
            circumcenter,
            circumradius_squared,
        }
    }

    fn calculate_circumcircle(vertices: [usize; 3], points: &[Point]) -> (Point, f64) {
        let p1 = &points[vertices[0]];
        let p2 = &points[vertices[1]];
        let p3 = &points[vertices[2]];

        let ax = p1.x;
        let ay = p1.y;
        let bx = p2.x;
        let by = p2.y;
        let cx = p3.x;
        let cy = p3.y;

        let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        
        if d.abs() < 1e-10 {
            return (Point::new(0.0, 0.0), f64::INFINITY);
        }

        let ux = ((ax * ax + ay * ay) * (by - cy) + (bx * bx + by * by) * (cy - ay) + (cx * cx + cy * cy) * (ay - by)) / d;
        let uy = ((ax * ax + ay * ay) * (cx - bx) + (bx * bx + by * by) * (ax - cx) + (cx * cx + cy * cy) * (bx - ax)) / d;

        let circumcenter = Point::new(ux, uy);
        let circumradius_squared = circumcenter.distance_squared_to(p1);

        (circumcenter, circumradius_squared)
    }

    pub fn contains_point_in_circumcircle(&self, point: &Point) -> bool {
        let dist_squared = self.circumcenter.distance_squared_to(point);
        dist_squared < self.circumradius_squared - 1e-10
    }

    pub fn area(&self, points: &[Point]) -> f64 {
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];

        0.5 * ((p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y)).abs()
    }

    pub fn angles(&self, points: &[Point]) -> [f64; 3] {
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];

        let a_squared = p2.distance_squared_to(p3);
        let b_squared = p1.distance_squared_to(p3);
        let c_squared = p1.distance_squared_to(p2);

        let a = a_squared.sqrt();
        let b = b_squared.sqrt();
        let c = c_squared.sqrt();

        let angle1 = ((b_squared + c_squared - a_squared) / (2.0 * b * c)).acos();
        let angle2 = ((a_squared + c_squared - b_squared) / (2.0 * a * c)).acos();
        let angle3 = ((a_squared + b_squared - c_squared) / (2.0 * a * b)).acos();

        [angle1.to_degrees(), angle2.to_degrees(), angle3.to_degrees()]
    }

    pub fn min_angle(&self, points: &[Point]) -> f64 {
        let angles = self.angles(points);
        angles.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }

    pub fn jacobian(&self, points: &[Point]) -> f64 {
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];

        // For a triangle, the Jacobian determinant is twice the signed area
        // J = |x2-x1  x3-x1|
        //     |y2-y1  y3-y1|
        // det(J) = (x2-x1)(y3-y1) - (x3-x1)(y2-y1)
        
        let dx21 = p2.x - p1.x;
        let dx31 = p3.x - p1.x;
        let dy21 = p2.y - p1.y;
        let dy31 = p3.y - p1.y;
        
        dx21 * dy31 - dx31 * dy21
    }

    pub fn is_properly_oriented(&self, points: &[Point]) -> bool {
        self.jacobian(points) > 0.0
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub vertices: [usize; 2],
}

impl Edge {
    pub fn new(v1: usize, v2: usize) -> Self {
        let vertices = if v1 < v2 { [v1, v2] } else { [v2, v1] };
        Self { vertices }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.vertices == other.vertices
    }
}

impl Eq for Edge {}

impl std::hash::Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertices.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quad {
    pub vertices: [usize; 4],
}

impl Quad {
    pub fn new(vertices: [usize; 4]) -> Self {
        Self { vertices }
    }
    
    pub fn area(&self, points: &[Point]) -> f64 {
        // Calculate quad area using shoelace formula
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];
        let p4 = &points[self.vertices[3]];
        
        0.5 * ((p1.x * p2.y - p2.x * p1.y) + 
               (p2.x * p3.y - p3.x * p2.y) + 
               (p3.x * p4.y - p4.x * p3.y) + 
               (p4.x * p1.y - p1.x * p4.y)).abs()
    }

    pub fn jacobian_at_center(&self, points: &[Point]) -> f64 {
        // For a quad, compute Jacobian at the center (xi=0, eta=0)
        // Using bilinear shape functions N1 = (1-xi)(1-eta)/4, etc.
        // J = |dx/dxi  dx/deta|
        //     |dy/dxi  dy/deta|
        
        let p1 = &points[self.vertices[0]]; // xi=-1, eta=-1
        let p2 = &points[self.vertices[1]]; // xi=+1, eta=-1
        let p3 = &points[self.vertices[2]]; // xi=+1, eta=+1
        let p4 = &points[self.vertices[3]]; // xi=-1, eta=+1
        
        // At center (xi=0, eta=0):
        // dx/dxi = 0.25 * (-p1.x + p2.x + p3.x - p4.x)
        // dx/deta = 0.25 * (-p1.x - p2.x + p3.x + p4.x)
        // dy/dxi = 0.25 * (-p1.y + p2.y + p3.y - p4.y)  
        // dy/deta = 0.25 * (-p1.y - p2.y + p3.y + p4.y)
        
        let dx_dxi = 0.25 * (-p1.x + p2.x + p3.x - p4.x);
        let dx_deta = 0.25 * (-p1.x - p2.x + p3.x + p4.x);
        let dy_dxi = 0.25 * (-p1.y + p2.y + p3.y - p4.y);
        let dy_deta = 0.25 * (-p1.y - p2.y + p3.y + p4.y);
        
        // det(J) = dx/dxi * dy/deta - dx/deta * dy/dxi
        dx_dxi * dy_deta - dx_deta * dy_dxi
    }

    pub fn min_jacobian(&self, points: &[Point]) -> f64 {
        // For a more thorough check, evaluate Jacobian at all four corners
        // This is more expensive but gives better quality assurance
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];
        let p4 = &points[self.vertices[3]];
        
        let mut min_jac = f64::INFINITY;
        
        // Check at the four Gauss points (±1/√3, ±1/√3)
        let gauss_point = 1.0 / 3.0_f64.sqrt();
        let xi_vals = [-gauss_point, gauss_point, gauss_point, -gauss_point];
        let eta_vals = [-gauss_point, -gauss_point, gauss_point, gauss_point];
        
        for i in 0..4 {
            let xi = xi_vals[i];
            let eta = eta_vals[i];
            
            // Compute Jacobian at (xi, eta)
            let dx_dxi = 0.25 * (-(1.0-eta)*p1.x + (1.0-eta)*p2.x + (1.0+eta)*p3.x - (1.0+eta)*p4.x);
            let dx_deta = 0.25 * (-(1.0-xi)*p1.x - (1.0+xi)*p2.x + (1.0+xi)*p3.x + (1.0-xi)*p4.x);
            let dy_dxi = 0.25 * (-(1.0-eta)*p1.y + (1.0-eta)*p2.y + (1.0+eta)*p3.y - (1.0+eta)*p4.y);
            let dy_deta = 0.25 * (-(1.0-xi)*p1.y - (1.0+xi)*p2.y + (1.0+xi)*p3.y + (1.0-xi)*p4.y);
            
            let jac = dx_dxi * dy_deta - dx_deta * dy_dxi;
            min_jac = min_jac.min(jac);
        }
        
        min_jac
    }

    pub fn is_properly_oriented(&self, points: &[Point]) -> bool {
        self.min_jacobian(points) > 0.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Vec<Point>>,
    pub triangle_indices: Vec<[usize; 3]>,
    pub quads: Option<Vec<Vec<Point>>>,
    pub quad_indices: Option<Vec<[usize; 4]>>,
}

impl Mesh {
    pub fn new(vertices: Vec<Point>, triangles: Vec<Triangle>) -> Self {
        let triangle_indices: Vec<[usize; 3]> = triangles.iter().map(|t| t.vertices).collect();
        let triangle_points: Vec<Vec<Point>> = triangles.iter().map(|t| {
            vec![
                vertices[t.vertices[0]].clone(),
                vertices[t.vertices[1]].clone(),
                vertices[t.vertices[2]].clone(),
            ]
        }).collect();

        Self {
            vertices,
            triangles: triangle_points,
            triangle_indices,
            quads: None,
            quad_indices: None,
        }
    }

    pub fn validate_jacobians(&self) -> Result<(), String> {
        // Check triangle Jacobians
        for (i, triangle_verts) in self.triangle_indices.iter().enumerate() {
            let triangle = Triangle::new(*triangle_verts, &self.vertices);
            let jacobian = triangle.jacobian(&self.vertices);
            if jacobian <= 0.0 {
                return Err(format!("Triangle {} has non-positive Jacobian: {}", i, jacobian));
            }
        }

        // Check quad Jacobians if present
        if let Some(quad_indices) = &self.quad_indices {
            for (i, quad_verts) in quad_indices.iter().enumerate() {
                let quad = Quad::new(*quad_verts);
                let min_jacobian = quad.min_jacobian(&self.vertices);
                if min_jacobian <= 0.0 {
                    return Err(format!("Quad {} has non-positive minimum Jacobian: {}", i, min_jacobian));
                }
            }
        }

        Ok(())
    }

    pub fn get_jacobian_stats(&self) -> (f64, f64, f64) {
        let mut min_jac = f64::INFINITY;
        let mut max_jac = f64::NEG_INFINITY;
        let mut sum_jac = 0.0;
        let mut count = 0;

        // Process triangles
        for triangle_verts in &self.triangle_indices {
            let triangle = Triangle::new(*triangle_verts, &self.vertices);
            let jacobian = triangle.jacobian(&self.vertices);
            min_jac = min_jac.min(jacobian);
            max_jac = max_jac.max(jacobian);
            sum_jac += jacobian;
            count += 1;
        }

        // Process quads if present
        if let Some(quad_indices) = &self.quad_indices {
            for quad_verts in quad_indices {
                let quad = Quad::new(*quad_verts);
                let jacobian = quad.min_jacobian(&self.vertices);
                min_jac = min_jac.min(jacobian);
                max_jac = max_jac.max(jacobian);
                sum_jac += jacobian;
                count += 1;
            }
        }

        let avg_jac = if count > 0 { sum_jac / count as f64 } else { 0.0 };
        (min_jac, max_jac, avg_jac)
    }
    
    pub fn new_with_quads(vertices: Vec<Point>, triangles: Vec<Triangle>, quads: Vec<Quad>) -> Self {
        let triangle_indices: Vec<[usize; 3]> = triangles.iter().map(|t| t.vertices).collect();
        let triangle_points: Vec<Vec<Point>> = triangles.iter().map(|t| {
            vec![
                vertices[t.vertices[0]].clone(),
                vertices[t.vertices[1]].clone(),
                vertices[t.vertices[2]].clone(),
            ]
        }).collect();
        
        let quad_indices: Vec<[usize; 4]> = quads.iter().map(|q| q.vertices).collect();
        let quad_points: Vec<Vec<Point>> = quads.iter().map(|q| {
            vec![
                vertices[q.vertices[0]].clone(),
                vertices[q.vertices[1]].clone(),
                vertices[q.vertices[2]].clone(),
                vertices[q.vertices[3]].clone(),
            ]
        }).collect();

        Self {
            vertices,
            triangles: triangle_points,
            triangle_indices,
            quads: Some(quad_points),
            quad_indices: Some(quad_indices),
        }
    }
}

pub struct DelaunayTriangulator {
    points: Vec<Point>,
    triangles: Vec<Triangle>,
}

impl DelaunayTriangulator {
    pub fn new(mut points: Vec<Point>) -> Self {
        let bounds = Self::calculate_bounds(&points);
        let super_triangle = Self::create_super_triangle(bounds);
        
        points.extend(super_triangle);
        
        let mut triangulator = Self {
            points,
            triangles: Vec::new(),
        };
        
        let super_triangle_vertices = [triangulator.points.len() - 3, triangulator.points.len() - 2, triangulator.points.len() - 1];
        let super_triangle = Triangle::new(super_triangle_vertices, &triangulator.points);
        
        // Ensure super triangle is properly oriented
        let mut vertices = super_triangle_vertices;
        if super_triangle.jacobian(&triangulator.points) < 0.0 {
            vertices = [super_triangle_vertices[2], super_triangle_vertices[1], super_triangle_vertices[0]];
        }
        
        triangulator.triangles.push(Triangle::new(vertices, &triangulator.points));
        
        triangulator
    }

    fn calculate_bounds(points: &[Point]) -> (f64, f64, f64, f64) {
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
        
        for i in 0..original_point_count {
            self.add_point(i)?;
        }

        self.remove_super_triangle();
        
        let vertices: Vec<Point> = self.points[..original_point_count].to_vec();
        let triangles = self.triangles.clone();
        
        Ok(Mesh::new(vertices, triangles))
    }

    fn add_point(&mut self, point_index: usize) -> Result<(), String> {
        let point = &self.points[point_index];
        let mut bad_triangles = Vec::new();

        for (i, triangle) in self.triangles.iter().enumerate() {
            if triangle.contains_point_in_circumcircle(point) {
                bad_triangles.push(i);
            }
        }

        let mut polygon = Vec::new();
        for &bad_triangle_index in &bad_triangles {
            let triangle = &self.triangles[bad_triangle_index];
            for i in 0..3 {
                let edge = Edge::new(triangle.vertices[i], triangle.vertices[(i + 1) % 3]);
                let mut is_shared = false;
                
                for &other_bad_triangle_index in &bad_triangles {
                    if other_bad_triangle_index == bad_triangle_index {
                        continue;
                    }
                    
                    let other_triangle = &self.triangles[other_bad_triangle_index];
                    for j in 0..3 {
                        let other_edge = Edge::new(other_triangle.vertices[j], other_triangle.vertices[(j + 1) % 3]);
                        if edge == other_edge {
                            is_shared = true;
                            break;
                        }
                    }
                    if is_shared {
                        break;
                    }
                }
                
                if !is_shared {
                    polygon.push(edge);
                }
            }
        }

        bad_triangles.sort_by(|a, b| b.cmp(a));
        for &index in &bad_triangles {
            self.triangles.remove(index);
        }

        for edge in polygon {
            let new_triangle = Triangle::new([edge.vertices[0], edge.vertices[1], point_index], &self.points);
            // Check if triangle is properly oriented, if not, flip it
            let mut vertices = [edge.vertices[0], edge.vertices[1], point_index];
            if new_triangle.jacobian(&self.points) < 0.0 {
                vertices = [edge.vertices[1], edge.vertices[0], point_index];
            }
            let corrected_triangle = Triangle::new(vertices, &self.points);
            self.triangles.push(corrected_triangle);
        }

        Ok(())
    }

    fn remove_super_triangle(&mut self) {
        let original_point_count = self.points.len() - 3;
        self.triangles.retain(|triangle| {
            triangle.vertices.iter().all(|&v| v < original_point_count)
        });
    }

    fn filter_outside_triangles(&mut self, boundary_count: usize) {
        let points = &self.points; // Borrow points outside the closure
        self.triangles.retain(|triangle| {
            // Check if triangle centroid is inside the boundary polygon
            let centroid = Point::new(
                (points[triangle.vertices[0]].x + points[triangle.vertices[1]].x + points[triangle.vertices[2]].x) / 3.0,
                (points[triangle.vertices[0]].y + points[triangle.vertices[1]].y + points[triangle.vertices[2]].y) / 3.0,
            );
            
            // Implement point-in-polygon test directly here to avoid borrow issues
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
        let max_iterations = 50; // Reduced from 1000 to prevent hanging
        let max_points = 10000; // Prevent too many points

        while iteration < max_iterations && self.points.len() < max_points {
            let mut needs_refinement = false;
            let mut bad_triangles = Vec::new();

            // Only refine based on area constraint for now (angle refinement is complex)
            for (i, triangle) in self.triangles.iter().enumerate() {
                let area = triangle.area(&self.points);
                if area > max_area {
                    // Check if triangle is inside the boundary polygon
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

            // Limit the number of triangles to refine per iteration
            let triangles_to_refine = bad_triangles.into_iter().take(5).collect::<Vec<_>>();
            let mut new_points = Vec::new();

            for &triangle_index in &triangles_to_refine {
                if triangle_index < self.triangles.len() {
                    let triangle = &self.triangles[triangle_index];
                    let centroid = Point::new(
                        (self.points[triangle.vertices[0]].x + self.points[triangle.vertices[1]].x + self.points[triangle.vertices[2]].x) / 3.0,
                        (self.points[triangle.vertices[0]].y + self.points[triangle.vertices[1]].y + self.points[triangle.vertices[2]].y) / 3.0,
                    );
                    
                    // Only add centroid if it's inside the boundary
                    if self.is_point_inside_polygon(&centroid, boundary_count) {
                        new_points.push(centroid);
                    }
                }
            }

            if new_points.is_empty() {
                break;
            }

            // Add new points one by one
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


    // Check if a point is inside a polygon using ray casting algorithm
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
}

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
        
        // Simple paving algorithm: create a structured grid inside the boundary
        let bounds = self.calculate_bounds();
        let (min_x, max_x, min_y, max_y) = bounds;
        
        // Calculate grid spacing based on target size
        let grid_size = (target_size.sqrt() * 0.8).max(1.0);
        
        // Generate grid points
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
        
        // Add grid points to our point collection
        let boundary_count = self.points.len();
        self.points.extend(grid_points);
        
        // Create quads from grid structure
        let cols = ((max_x - min_x) / grid_size).ceil() as usize;
        let rows = ((max_y - min_y) / grid_size).ceil() as usize;
        
        for row in 0..rows-1 {
            for col in 0..cols-1 {
                let base_idx = boundary_count + row * cols + col;
                if base_idx + cols + 1 < self.points.len() {
                    // Check if all four points exist and are inside
                    if self.point_exists_and_inside(base_idx) &&
                       self.point_exists_and_inside(base_idx + 1) &&
                       self.point_exists_and_inside(base_idx + cols) &&
                       self.point_exists_and_inside(base_idx + cols + 1) {
                        
                        // Create quad with proper counter-clockwise orientation
                        let mut vertices = [
                            base_idx,
                            base_idx + 1,
                            base_idx + cols + 1,
                            base_idx + cols,
                        ];
                        
                        let quad = Quad::new(vertices);
                        
                        // Check if quad has positive Jacobian, if not, reorder vertices
                        if quad.min_jacobian(&self.points) <= 0.0 {
                            // Try different ordering
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
        
        // Fill remaining areas with triangles if needed
        self.fill_boundary_with_triangles(boundary_count);
        
        let mesh = Mesh::new_with_quads(self.points.clone(), self.triangles.clone(), self.quads.clone());
        
        // Validate Jacobians
        if let Err(e) = mesh.validate_jacobians() {
            return Err(format!("Paving mesh validation failed: {}", e));
        }
        
        let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
        log::info!("Paving mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", min_jac, max_jac, avg_jac);
        
        Ok(mesh)
    }
    
    fn calculate_bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

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
        let boundary_count = self.points.len().min(self.points.len()); // Use original boundary
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
        // Simple triangulation of boundary - this is a placeholder
        // In a full implementation, you'd do proper boundary triangulation
        if boundary_count >= 3 {
            for i in 1..boundary_count-1 {
                let mut vertices = [0, i, i+1];
                let triangle = Triangle::new(vertices, &self.points);
                
                // Ensure proper orientation
                if triangle.jacobian(&self.points) < 0.0 {
                    vertices = [0, i+1, i];
                }
                
                let corrected_triangle = Triangle::new(vertices, &self.points);
                self.triangles.push(corrected_triangle);
            }
        }
    }
}

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
        
        // Step 1: Generate boundary points (refine existing boundary)
        self.refine_boundary_points(target_area)?;
        log::info!("ANNEALING - Refined boundary to {} points", self.boundary_points.len());
        
        // Step 2: Generate internal grid
        self.generate_internal_grid(target_area)?;
        log::info!("ANNEALING - Generated internal grid with {} points", self.internal_points.len());
        
        // Step 3: Create initial triangulation
        self.create_initial_triangulation()?;
        log::info!("ANNEALING - Created initial triangulation with {} triangles", self.triangles.len());
        
        // Step 4: Optimize with simulated annealing
        self.optimize_with_annealing()?;
        log::info!("ANNEALING - Optimization complete");
        
        // Step 5: Convert to mesh
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
        // Combine boundary and internal points
        let mut all_points = self.boundary_points.clone();
        all_points.extend(self.internal_points.clone());
        
        // Create Delaunay triangulation
        let mut triangulator = DelaunayTriangulator::new(all_points);
        let mesh = triangulator.triangulate()?;
        
        // Extract triangles
        self.triangles = mesh.triangle_indices.iter().map(|&vertices| {
            Triangle::new(vertices, &mesh.vertices)
        }).collect();
        
        // Update points to match mesh vertices
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
            
            // Try to move a random internal point
            if !self.internal_points.is_empty() {
                let point_idx = self.rng.gen_range(0..self.internal_points.len());
                let old_point = self.internal_points[point_idx].clone();
                
                // Generate small random perturbation
                let perturbation_radius = temperature * 0.1;
                let dx = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                let dy = self.rng.gen_range(-perturbation_radius..perturbation_radius);
                
                let new_point = Point::new(old_point.x + dx, old_point.y + dy);
                
                // Check if new point is still inside polygon
                if self.is_point_inside_polygon(&new_point) {
                    // Apply perturbation
                    self.internal_points[point_idx] = new_point;
                    
                    // Recalculate affected triangles
                    self.update_triangulation_after_move(point_idx + self.boundary_points.len())?;
                    
                    let new_quality = self.calculate_mesh_quality();
                    let quality_improvement = new_quality - current_quality;
                    
                    // Accept or reject based on simulated annealing criteria
                    if quality_improvement > 0.0 || 
                       self.rng.gen::<f64>() < (quality_improvement / temperature).exp() {
                        // Accept the move
                        if iterations % 1000 == 0 {
                            log::info!("ANNEALING - Iteration {}: quality={:.4}, temp={:.2}", 
                                      iterations, new_quality, temperature);
                        }
                    } else {
                        // Reject the move
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
            
            // Quality metric: combines minimum angle and jacobian
            // Good triangles have large minimum angles and positive jacobians
            if jacobian > 0.0 {
                let angle_quality = min_angle / 60.0; // Normalize to equilateral triangle
                let jacobian_quality = jacobian.min(1.0); // Cap at 1.0
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
    
    fn update_triangulation_after_move(&mut self, moved_point_idx: usize) -> Result<(), String> {
        // For simplicity, we'll recreate the triangulation
        // In a more sophisticated implementation, we'd only update affected triangles
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
        
        // Validate the mesh
        if let Err(e) = mesh.validate_jacobians() {
            return Err(format!("Annealing mesh validation failed: {}", e));
        }
        
        let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
        log::info!("Annealing mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", 
                  min_jac, max_jac, avg_jac);
        
        Ok(mesh)
    }
    
    fn calculate_bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

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

// Standalone function to refine boundary edges before triangulation
fn refine_boundary_edges(boundary_points: Vec<Point>, max_area: f64) -> Result<Vec<Point>, String> {
    // Calculate target edge length based on max area
    // For a triangle with area A, if it's roughly equilateral, edge length ≈ sqrt(4A/sqrt(3))
    let target_edge_length = (4.0 * max_area / 3.0_f64.sqrt()).sqrt();
    
    let mut refined_points = Vec::new();
    
    // Process each boundary edge
    for i in 0..boundary_points.len() {
        let next_i = (i + 1) % boundary_points.len();
        let p1 = &boundary_points[i];
        let p2 = &boundary_points[next_i];
        
        // Always add the current point
        refined_points.push(p1.clone());
        
        let edge_length = p1.distance_to(p2);
        
        // If edge is too long, subdivide it
        if edge_length > target_edge_length {
            let num_subdivisions = (edge_length / target_edge_length).ceil() as usize;
            
            // Add intermediate points along the edge
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
    
    Ok(refined_points)
}

pub fn generate_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 3 {
        return Err("Need at least 3 points to generate mesh".to_string());
    }

    let algorithm = request.algorithm.clone().unwrap_or_else(|| "delaunay".to_string());
    
    match algorithm.as_str() {
        "paving" => generate_paving_mesh(request),
        "annealing" => generate_annealing_mesh(request),
        "delaunay" | _ => generate_delaunay_mesh(request),
    }
}

fn generate_annealing_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 3 {
        return Err("Need at least 3 points for annealing mesh".to_string());
    }
    
    let target_area = request.max_area.unwrap_or(100.0);
    let quality_threshold = request.min_angle.unwrap_or(20.0) / 60.0; // Normalize to [0,1]
    
    let mut annealing_generator = SimulatedAnnealingMeshGenerator::new(
        request.geometry.points, 
        quality_threshold
    );
    annealing_generator.generate_mesh(target_area)
}

fn generate_paving_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 4 {
        return Err("Need at least 4 points for paving mesh".to_string());
    }
    
    let target_size = request.max_area.unwrap_or(100.0);
    let mut paving_generator = PavingMeshGenerator::new(request.geometry.points);
    paving_generator.generate_mesh(target_size)
}

fn generate_delaunay_mesh(request: MeshRequest) -> Result<Mesh, String> {
    let mut boundary_points = request.geometry.points;
    
    // Apply boundary refinement if area constraint is specified
    if let Some(max_area) = request.max_area {
        boundary_points = refine_boundary_edges(boundary_points, max_area)?;
    }
    
    let refined_boundary_count = boundary_points.len();
    
    // Create triangulator with the refined boundary
    let mut triangulator = DelaunayTriangulator::new(boundary_points);
    
    // Add all boundary points (including refined ones)
    for i in 0..refined_boundary_count {
        triangulator.add_point(i)?;
    }
    
    // Apply interior refinement if area or angle constraints are specified
    if let (Some(max_area), Some(min_angle)) = (request.max_area, request.min_angle) {
        triangulator.refine_interior(max_area, min_angle, refined_boundary_count)?;
    }
    
    // Remove super triangle and filter outside triangles
    triangulator.remove_super_triangle();
    triangulator.filter_outside_triangles(refined_boundary_count);
    
    let vertices: Vec<Point> = triangulator.points[..triangulator.points.len() - 3].to_vec();
    let mesh = Mesh::new(vertices, triangulator.triangles.clone());
    
    // Validate Jacobians
    if let Err(e) = mesh.validate_jacobians() {
        return Err(format!("Mesh validation failed: {}", e));
    }
    
    let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
    log::info!("Delaunay mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", min_jac, max_jac, avg_jac);
    
    Ok(mesh)
}

pub fn export_to_csv(geometry: &Geometry, mesh: Option<&Mesh>) -> Result<String, String> {
    let mut csv_content = String::new();
    csv_content.push_str("Type,Index,X,Y,Additional_Info\n");

    for (i, point) in geometry.points.iter().enumerate() {
        csv_content.push_str(&format!("Point,{},{},{},Boundary_Point_{}\n", i, point.x, point.y, i));
    }

    if let Some(mesh) = mesh {
        for (i, vertex) in mesh.vertices.iter().enumerate() {
            csv_content.push_str(&format!("Mesh_Vertex,{},{},{},Mesh_Node\n", i, vertex.x, vertex.y));
        }

        for (i, triangle) in mesh.triangle_indices.iter().enumerate() {
            csv_content.push_str(&format!("Triangle,{},{},{},Triangle_Nodes_{}\n", i, triangle[0], triangle[1], triangle[2]));
        }
    }

    Ok(csv_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(1.0, 2.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert!(point.id.is_none());
    }

    #[test]
    fn test_triangle_creation() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.5, 1.0),
        ];
        let triangle = Triangle::new([0, 1, 2], &points);
        assert_eq!(triangle.vertices, [0, 1, 2]);
    }

    #[test]
    fn test_simple_triangulation() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.5, 1.0),
        ];
        let geometry = Geometry::new(points);
        let request = MeshRequest::new(geometry);
        let mesh = generate_mesh(request).unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangle_indices.len(), 1);
    }
}