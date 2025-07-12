use std::f64;
use crate::geometry::Point;
use crate::elements::{Triangle, Edge};
use crate::mesh::Mesh;

pub struct DelaunayTriangulator {
    pub points: Vec<Point>,
    pub triangles: Vec<Triangle>,
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

    pub fn add_point(&mut self, point_index: usize) -> Result<(), String> {
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
            let mut vertices = [edge.vertices[0], edge.vertices[1], point_index];
            if new_triangle.jacobian(&self.points) < 0.0 {
                vertices = [edge.vertices[1], edge.vertices[0], point_index];
            }
            let corrected_triangle = Triangle::new(vertices, &self.points);
            self.triangles.push(corrected_triangle);
        }

        Ok(())
    }

    pub fn remove_super_triangle(&mut self) {
        let original_point_count = self.points.len() - 3;
        self.triangles.retain(|triangle| {
            triangle.vertices.iter().all(|&v| v < original_point_count)
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
}