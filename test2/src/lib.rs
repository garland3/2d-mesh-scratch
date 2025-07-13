use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    pub indices: [usize; 3],
}

impl Triangle {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        Self { indices: [a, b, c] }
    }

    pub fn get_points<'a>(&self, points: &'a [Point]) -> [&'a Point; 3] {
        [
            &points[self.indices[0]],
            &points[self.indices[1]],
            &points[self.indices[2]],
        ]
    }

    pub fn center(&self, points: &[Point]) -> Point {
        let pts = self.get_points(points);
        Point::new(
            (pts[0].x + pts[1].x + pts[2].x) / 3.0,
            (pts[0].y + pts[1].y + pts[2].y) / 3.0,
        )
    }

    pub fn circumcenter(&self, points: &[Point]) -> Option<Point> {
        let pts = self.get_points(points);
        let p1 = pts[0];
        let p2 = pts[1];
        let p3 = pts[2];

        let d = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));
        if d.abs() < 1e-9 {
            return None;
        }

        let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y)
            + (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y)
            + (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y))
            / d;
        let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x)
            + (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x)
            + (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x))
            / d;

        Some(Point::new(ux, uy))
    }

    pub fn min_angle(&self, points: &[Point]) -> f64 {
        let pts = self.get_points(points);
        let a = pts[1].distance_to(pts[2]);
        let b = pts[0].distance_to(pts[2]);
        let c = pts[0].distance_to(pts[1]);

        if b * c == 0.0 || a * c == 0.0 || a * b == 0.0 {
            return 0.0;
        }

        let angle_a = ((b * b + c * c - a * a) / (2.0 * b * c)).acos();
        let angle_b = ((a * a + c * c - b * b) / (2.0 * a * c)).acos();
        let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b)).acos();

        angle_a.min(angle_b).min(angle_c).to_degrees()
    }

    pub fn aspect_ratio(&self, points: &[Point]) -> f64 {
        let pts = self.get_points(points);
        let a = pts[1].distance_to(pts[2]);
        let b = pts[0].distance_to(pts[2]);
        let c = pts[0].distance_to(pts[1]);

        let s = (a + b + c) / 2.0;
        let area = (s * (s - a) * (s - b) * (s - c)).max(0.0).sqrt();
        
        if area < 1e-9 {
            return f64::INFINITY;
        }

        let circumradius = (a * b * c) / (4.0 * area);
        let inradius = area / s;
        
        if inradius < 1e-9 {
            return f64::INFINITY;
        }

        circumradius / (2.0 * inradius)
    }
}

#[wasm_bindgen]
pub struct Mesher {
    points: Vec<Point>,
    triangles: Vec<Triangle>,
    boundary_points: HashSet<usize>,
}

#[wasm_bindgen]
impl Mesher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            triangles: Vec::new(),
            boundary_points: HashSet::new(),
        }
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.points.clear();
        self.triangles.clear();
        self.boundary_points.clear();
    }

    #[wasm_bindgen]
    pub fn add_polygon(&mut self, polygon_points: &[f64]) {
        self.clear();
        
        for i in (0..polygon_points.len()).step_by(2) {
            if i + 1 < polygon_points.len() {
                let point = Point::new(polygon_points[i], polygon_points[i + 1]);
                self.points.push(point);
                self.boundary_points.insert(self.points.len() - 1);
            }
        }
    }

    fn is_point_in_polygon(&self, point: &Point, polygon: &[Point]) -> bool {
        let x = point.x;
        let y = point.y;
        let mut inside = false;

        let mut j = polygon.len() - 1;
        for i in 0..polygon.len() {
            let xi = polygon[i].x;
            let yi = polygon[i].y;
            let xj = polygon[j].x;
            let yj = polygon[j].y;

            if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    #[wasm_bindgen]
    pub fn generate_mesh(&mut self, density: f64) -> bool {
        if self.points.len() < 3 {
            return false;
        }

        let polygon = self.points.clone();
        self.densify_boundary(density);
        self.add_interior_points(density, &polygon);
        self.triangulate(&polygon);

        true
    }

    fn densify_boundary(&mut self, density: f64) {
        let original_count = self.points.len();
        let mut new_points = Vec::new();
        
        for i in 0..original_count {
            let current = self.points[i];
            let next = self.points[(i + 1) % original_count];
            
            let edge_length = current.distance_to(&next);
            let num_segments = (edge_length / density).ceil() as usize;
            
            new_points.push(current);
            
            if num_segments > 1 {
                for j in 1..num_segments {
                    let t = j as f64 / num_segments as f64;
                    let x = current.x + t * (next.x - current.x);
                    let y = current.y + t * (next.y - current.y);
                    new_points.push(Point::new(x, y));
                    self.boundary_points.insert(self.points.len() + new_points.len() - 1);
                }
            }
        }
        
        self.points = new_points;
        // Update boundary points indices
        self.boundary_points.clear();
        for i in 0..self.points.len() {
            self.boundary_points.insert(i);
        }
    }

    fn add_interior_points(&mut self, density: f64, polygon: &[Point]) {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for point in polygon {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        let mut x = min_x;
        while x < max_x {
            let mut y = min_y;
            while y < max_y {
                let point = Point::new(x, y);
                if self.is_point_in_polygon(&point, polygon) {
                    self.points.push(point);
                }
                y += density;
            }
            x += density;
        }
    }

    fn triangulate(&mut self, polygon: &[Point]) {
        if self.points.len() < 3 {
            return;
        }

        // Use delaunator for Delaunay triangulation
        let delaunay_points: Vec<delaunator::Point> = self.points.iter()
            .map(|p| delaunator::Point { x: p.x, y: p.y })
            .collect();

        let triangulation = delaunator::triangulate(&delaunay_points);
        
        self.triangles.clear();
        
        // Filter triangles to only include those inside the polygon
        for i in (0..triangulation.triangles.len()).step_by(3) {
            let tri = Triangle::new(
                triangulation.triangles[i],
                triangulation.triangles[i + 1],
                triangulation.triangles[i + 2],
            );
            
            let center = tri.center(&self.points);
            if self.is_point_in_polygon(&center, polygon) {
                self.triangles.push(tri);
            }
        }
    }

    #[wasm_bindgen]
    pub fn refine_mesh(&mut self, metric: &str, threshold: f64, max_iterations: usize) -> usize {
        let polygon: Vec<Point> = self.boundary_points.iter()
            .map(|&i| self.points[i])
            .collect();

        let mut iterations = 0;
        
        for _ in 0..max_iterations {
            let worst_triangle = self.find_worst_triangle(metric, threshold);
            
            if worst_triangle.is_none() {
                break;
            }
            
            let triangle = worst_triangle.unwrap();
            if let Some(circumcenter) = triangle.circumcenter(&self.points) {
                if self.is_point_in_polygon(&circumcenter, &polygon) {
                    self.points.push(circumcenter);
                    self.triangulate(&polygon);
                    iterations += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        iterations
    }

    fn find_worst_triangle(&self, metric: &str, threshold: f64) -> Option<Triangle> {
        let mut worst_triangle = None;
        let mut worst_quality = if metric == "angle" { 180.0 } else { 0.0 };

        for triangle in &self.triangles {
            let quality = match metric {
                "angle" => triangle.min_angle(&self.points),
                "aspect" => triangle.aspect_ratio(&self.points),
                _ => continue,
            };

            let is_bad = match metric {
                "angle" => quality < threshold,
                "aspect" => quality > threshold,
                _ => false,
            };

            if is_bad {
                let is_worse = match metric {
                    "angle" => quality < worst_quality,
                    "aspect" => quality > worst_quality,
                    _ => false,
                };

                if is_worse {
                    worst_quality = quality;
                    worst_triangle = Some(triangle.clone());
                }
            }
        }

        worst_triangle
    }

    #[wasm_bindgen]
    pub fn smooth_mesh(&mut self, iterations: usize) -> bool {
        let polygon: Vec<Point> = self.boundary_points.iter()
            .map(|&i| self.points[i])
            .collect();

        for _ in 0..iterations {
            let mut neighbors: HashMap<usize, Vec<usize>> = HashMap::new();
            
            // Build neighbor relationships
            for triangle in &self.triangles {
                for i in 0..3 {
                    let curr = triangle.indices[i];
                    let next1 = triangle.indices[(i + 1) % 3];
                    let next2 = triangle.indices[(i + 2) % 3];
                    
                    neighbors.entry(curr).or_insert_with(Vec::new).push(next1);
                    neighbors.entry(curr).or_insert_with(Vec::new).push(next2);
                }
            }

            // Smooth interior points using Laplacian smoothing
            let mut new_points = self.points.clone();
            let mut moved_count = 0;

            for (i, _point) in self.points.iter().enumerate() {
                if self.boundary_points.contains(&i) {
                    continue; // Don't move boundary points
                }

                if let Some(neighbor_indices) = neighbors.get(&i) {
                    let mut avg_x = 0.0;
                    let mut avg_y = 0.0;
                    let count = neighbor_indices.len() as f64;

                    for &neighbor_idx in neighbor_indices {
                        if neighbor_idx < self.points.len() {
                            avg_x += self.points[neighbor_idx].x;
                            avg_y += self.points[neighbor_idx].y;
                        }
                    }

                    if count > 0.0 {
                        new_points[i] = Point::new(avg_x / count, avg_y / count);
                        moved_count += 1;
                    }
                }
            }

            self.points = new_points;
            self.triangulate(&polygon);

            if moved_count == 0 {
                break;
            }
        }

        true
    }

    #[wasm_bindgen]
    pub fn get_mesh_data(&self) -> String {
        let triangles_data: Vec<[usize; 3]> = self.triangles.iter()
            .map(|t| t.indices)
            .collect();

        let mesh_data = serde_json::json!({
            "points": self.points,
            "elements": triangles_data
        });

        mesh_data.to_string()
    }

    #[wasm_bindgen]
    pub fn get_triangle_count(&self) -> usize {
        self.triangles.len()
    }

    #[wasm_bindgen]
    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    #[wasm_bindgen]
    pub fn get_average_quality(&self, metric: &str) -> f64 {
        if self.triangles.is_empty() {
            return 0.0;
        }

        let total: f64 = self.triangles.iter()
            .map(|t| match metric {
                "angle" => t.min_angle(&self.points),
                "aspect" => t.aspect_ratio(&self.points),
                _ => 0.0,
            })
            .sum();

        total / self.triangles.len() as f64
    }

    #[wasm_bindgen]
    pub fn get_triangles_for_drawing(&self) -> Vec<f64> {
        let mut result = Vec::new();
        
        for triangle in &self.triangles {
            for &idx in &triangle.indices {
                if idx < self.points.len() {
                    result.push(self.points[idx].x);
                    result.push(self.points[idx].y);
                }
            }
        }
        
        result
    }

    #[wasm_bindgen]
    pub fn get_boundary_points_for_drawing(&self) -> Vec<f64> {
        let mut result = Vec::new();
        
        let boundary_vec: Vec<usize> = self.boundary_points.iter().cloned().collect();
        for &idx in &boundary_vec {
            if idx < self.points.len() {
                result.push(self.points[idx].x);
                result.push(self.points[idx].y);
            }
        }
        
        result
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Rust mesher loaded!");
}