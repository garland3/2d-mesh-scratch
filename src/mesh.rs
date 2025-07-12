use std::f64;
use serde::{Deserialize, Serialize};
use crate::geometry::Point;
use crate::elements::{Triangle, Quad};

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
        for (i, triangle_verts) in self.triangle_indices.iter().enumerate() {
            let triangle = Triangle::new(*triangle_verts, &self.vertices);
            let jacobian = triangle.jacobian(&self.vertices);
            if jacobian <= 0.0 {
                return Err(format!("Triangle {} has non-positive Jacobian: {}", i, jacobian));
            }
        }

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

        for triangle_verts in &self.triangle_indices {
            let triangle = Triangle::new(*triangle_verts, &self.vertices);
            let jacobian = triangle.jacobian(&self.vertices);
            min_jac = min_jac.min(jacobian);
            max_jac = max_jac.max(jacobian);
            sum_jac += jacobian;
            count += 1;
        }

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