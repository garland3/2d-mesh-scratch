use serde::{Deserialize, Serialize};

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