use serde::{Deserialize, Serialize};

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