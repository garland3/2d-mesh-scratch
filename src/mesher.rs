use crate::geometry::{Point, Triangle};
use std::collections::{HashMap, HashSet};

pub struct MeshCore {
    pub points: Vec<Point>,
    pub triangles: Vec<Triangle>,
    pub boundary_points: HashSet<usize>,
}

impl MeshCore {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            triangles: Vec::new(),
            boundary_points: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.triangles.clear();
        self.boundary_points.clear();
    }

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

    pub fn add_polygon_from_points(&mut self, polygon_points: &[Point]) {
        self.clear();
        for point in polygon_points {
            self.points.push(*point);
            self.boundary_points.insert(self.points.len() - 1);
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
                }
            }
        }
        
        self.points = new_points;
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

        let delaunay_points: Vec<delaunator::Point> = self.points.iter()
            .map(|p| delaunator::Point { x: p.x, y: p.y })
            .collect();

        let triangulation = delaunator::triangulate(&delaunay_points);
        
        self.triangles.clear();
        
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

    pub fn smooth_mesh(&mut self, iterations: usize) -> bool {
        let polygon: Vec<Point> = self.boundary_points.iter()
            .map(|&i| self.points[i])
            .collect();

        for _ in 0..iterations {
            let mut neighbors: HashMap<usize, Vec<usize>> = HashMap::new();
            
            for triangle in &self.triangles {
                for i in 0..3 {
                    let curr = triangle.indices[i];
                    let next1 = triangle.indices[(i + 1) % 3];
                    let next2 = triangle.indices[(i + 2) % 3];
                    
                    neighbors.entry(curr).or_insert_with(Vec::new).push(next1);
                    neighbors.entry(curr).or_insert_with(Vec::new).push(next2);
                }
            }

            let mut new_points = self.points.clone();
            let mut moved_count = 0;

            for (i, _point) in self.points.iter().enumerate() {
                if self.boundary_points.contains(&i) {
                    continue;
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
}