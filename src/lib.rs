use serde::{Deserialize, Serialize};
use std::f64;

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
}

impl MeshRequest {
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            max_area: Some(0.1),
            min_angle: Some(20.0),
        }
    }

    pub fn with_constraints(geometry: Geometry, max_area: f64, min_angle: f64) -> Self {
        Self {
            geometry,
            max_area: Some(max_area),
            min_angle: Some(min_angle),
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
pub struct Mesh {
    pub vertices: Vec<Point>,
    pub triangles: Vec<Vec<Point>>,
    pub triangle_indices: Vec<[usize; 3]>,
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
        
        triangulator.triangles.push(Triangle::new(
            [triangulator.points.len() - 3, triangulator.points.len() - 2, triangulator.points.len() - 1],
            &triangulator.points,
        ));
        
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
            self.triangles.push(new_triangle);
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

    pub fn refine_mesh(&mut self, max_area: f64, _min_angle: f64) -> Result<(), String> {
        let mut iteration = 0;
        let max_iterations = 50; // Reduced from 1000 to prevent hanging
        let max_points = 10000; // Prevent too many points
        let original_boundary_count = self.points.len() - 3; // Exclude super triangle

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
                    
                    if self.is_point_inside_polygon(&centroid, original_boundary_count) {
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
                    if self.is_point_inside_polygon(&centroid, original_boundary_count) {
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

pub fn generate_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 3 {
        return Err("Need at least 3 points to generate mesh".to_string());
    }

    let original_point_count = request.geometry.points.len();
    
    // Apply refinement if area or angle constraints are specified
    if let (Some(max_area), Some(min_angle)) = (request.max_area, request.min_angle) {
        // Create triangulator with refinement
        let mut triangulator = DelaunayTriangulator::new(request.geometry.points);
        
        // Add original points
        for i in 0..original_point_count {
            triangulator.add_point(i)?;
        }
        
        // Apply refinement
        triangulator.refine_mesh(max_area, min_angle)?;
        
        // Remove super triangle and filter outside triangles
        triangulator.remove_super_triangle();
        triangulator.filter_outside_triangles(original_point_count);
        
        let vertices: Vec<Point> = triangulator.points[..triangulator.points.len() - 3].to_vec();
        let mesh = Mesh::new(vertices, triangulator.triangles.clone());
        Ok(mesh)
    } else {
        // No refinement - use basic triangulation but still filter outside triangles
        let mut triangulator = DelaunayTriangulator::new(request.geometry.points);
        
        // Add original points
        for i in 0..original_point_count {
            triangulator.add_point(i)?;
        }
        
        // Remove super triangle and filter outside triangles
        triangulator.remove_super_triangle();
        triangulator.filter_outside_triangles(original_point_count);
        
        let vertices: Vec<Point> = triangulator.points[..triangulator.points.len() - 3].to_vec();
        let mesh = Mesh::new(vertices, triangulator.triangles.clone());
        Ok(mesh)
    }
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