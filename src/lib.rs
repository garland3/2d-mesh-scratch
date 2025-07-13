use wasm_bindgen::prelude::*;
use serde_json;

pub mod geometry;
pub mod mesher;

use geometry::{Point, Triangle};
use mesher::MeshCore;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct Mesher {
    core: MeshCore,
}

#[wasm_bindgen]
impl Mesher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: MeshCore::new(),
        }
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.core.clear();
    }

    #[wasm_bindgen]
    pub fn add_polygon(&mut self, polygon_points: &[f64]) {
        self.core.add_polygon(polygon_points);
    }

    #[wasm_bindgen]
    pub fn generate_mesh(&mut self, density: f64) -> bool {
        self.core.generate_mesh(density)
    }

    #[wasm_bindgen]
    pub fn refine_mesh(&mut self, metric: &str, threshold: f64, max_iterations: usize) -> usize {
        self.core.refine_mesh(metric, threshold, max_iterations)
    }

    #[wasm_bindgen]
    pub fn smooth_mesh(&mut self, iterations: usize) -> bool {
        self.core.smooth_mesh(iterations)
    }

    #[wasm_bindgen]
    pub fn get_mesh_data(&self) -> String {
        let triangles_data: Vec<[usize; 3]> = self.core.triangles.iter()
            .map(|t| t.indices)
            .collect();

        let mesh_data = serde_json::json!({
            "points": self.core.points,
            "elements": triangles_data
        });

        mesh_data.to_string()
    }

    #[wasm_bindgen]
    pub fn get_triangle_count(&self) -> usize {
        self.core.triangles.len()
    }

    #[wasm_bindgen]
    pub fn get_point_count(&self) -> usize {
        self.core.points.len()
    }

    #[wasm_bindgen]
    pub fn get_average_quality(&self, metric: &str) -> f64 {
        self.core.get_average_quality(metric)
    }

    #[wasm_bindgen]
    pub fn get_triangles_for_drawing(&self) -> Vec<f64> {
        let mut result = Vec::new();
        
        for triangle in &self.core.triangles {
            for &idx in &triangle.indices {
                if idx < self.core.points.len() {
                    result.push(self.core.points[idx].x);
                    result.push(self.core.points[idx].y);
                }
            }
        }
        
        result
    }

    #[wasm_bindgen]
    pub fn get_boundary_points_for_drawing(&self) -> Vec<f64> {
        let mut result = Vec::new();
        
        let boundary_vec: Vec<usize> = self.core.boundary_points.iter().cloned().collect();
        for &idx in &boundary_vec {
            if idx < self.core.points.len() {
                result.push(self.core.points[idx].x);
                result.push(self.core.points[idx].y);
            }
        }
        
        result
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Rust mesher loaded!");
}