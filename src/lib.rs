pub mod geometry;
pub mod elements;
pub mod mesh;
pub mod delaunay;
pub mod paving;
pub mod annealing;
pub mod generation;

pub use geometry::{Point, Geometry, MeshRequest};
pub use elements::{Triangle, Edge, Quad};
pub use mesh::Mesh;
pub use delaunay::DelaunayTriangulator;
pub use paving::PavingMeshGenerator;
pub use annealing::SimulatedAnnealingMeshGenerator;
pub use generation::{generate_mesh, export_to_csv};

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