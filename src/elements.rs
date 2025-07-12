use std::f64;
use serde::{Deserialize, Serialize};
use crate::geometry::Point;

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
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];
        let p4 = &points[self.vertices[3]];
        
        let dx_dxi = 0.25 * (-p1.x + p2.x + p3.x - p4.x);
        let dx_deta = 0.25 * (-p1.x - p2.x + p3.x + p4.x);
        let dy_dxi = 0.25 * (-p1.y + p2.y + p3.y - p4.y);
        let dy_deta = 0.25 * (-p1.y - p2.y + p3.y + p4.y);
        
        dx_dxi * dy_deta - dx_deta * dy_dxi
    }

    pub fn min_jacobian(&self, points: &[Point]) -> f64 {
        let p1 = &points[self.vertices[0]];
        let p2 = &points[self.vertices[1]];
        let p3 = &points[self.vertices[2]];
        let p4 = &points[self.vertices[3]];
        
        let mut min_jac = f64::INFINITY;
        
        let gauss_point = 1.0 / 3.0_f64.sqrt();
        let xi_vals = [-gauss_point, gauss_point, gauss_point, -gauss_point];
        let eta_vals = [-gauss_point, -gauss_point, gauss_point, gauss_point];
        
        for i in 0..4 {
            let xi = xi_vals[i];
            let eta = eta_vals[i];
            
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