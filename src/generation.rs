use crate::geometry::{Point, Geometry, MeshRequest};
use crate::mesh::Mesh;
use crate::delaunay::DelaunayTriangulator;
use crate::paving::PavingMeshGenerator;
use crate::annealing::SimulatedAnnealingMeshGenerator;

fn refine_boundary_edges(boundary_points: Vec<Point>, max_area: f64) -> Result<Vec<Point>, String> {
    let target_edge_length = (4.0 * max_area / 3.0_f64.sqrt()).sqrt();
    
    let mut refined_points = Vec::new();
    
    for i in 0..boundary_points.len() {
        let next_i = (i + 1) % boundary_points.len();
        let p1 = &boundary_points[i];
        let p2 = &boundary_points[next_i];
        
        refined_points.push(p1.clone());
        
        let edge_length = p1.distance_to(p2);
        
        if edge_length > target_edge_length {
            let num_subdivisions = (edge_length / target_edge_length).ceil() as usize;
            
            for j in 1..num_subdivisions {
                let t = j as f64 / num_subdivisions as f64;
                let new_point = Point::new(
                    p1.x + t * (p2.x - p1.x),
                    p1.y + t * (p2.y - p1.y),
                );
                refined_points.push(new_point);
            }
        }
    }
    
    Ok(refined_points)
}

pub fn generate_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 3 {
        return Err("Need at least 3 points to generate mesh".to_string());
    }

    let algorithm = request.algorithm.clone().unwrap_or_else(|| "delaunay".to_string());
    
    match algorithm.as_str() {
        "paving" => generate_paving_mesh(request),
        "annealing" => generate_annealing_mesh(request),
        "delaunay" | _ => generate_delaunay_mesh(request),
    }
}

fn generate_annealing_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 3 {
        return Err("Need at least 3 points for annealing mesh".to_string());
    }
    
    let target_area = request.max_area.unwrap_or(100.0);
    let quality_threshold = request.min_angle.unwrap_or(20.0) / 60.0;
    
    let mut annealing_generator = SimulatedAnnealingMeshGenerator::new(
        request.geometry.points, 
        quality_threshold
    );
    annealing_generator.generate_mesh(target_area)
}

fn generate_paving_mesh(request: MeshRequest) -> Result<Mesh, String> {
    if request.geometry.points.len() < 4 {
        return Err("Need at least 4 points for paving mesh".to_string());
    }
    
    let target_size = request.max_area.unwrap_or(100.0);
    let mut paving_generator = PavingMeshGenerator::new(request.geometry.points);
    paving_generator.generate_mesh(target_size)
}

fn generate_delaunay_mesh(request: MeshRequest) -> Result<Mesh, String> {
    let mut boundary_points = request.geometry.points;
    
    if let Some(max_area) = request.max_area {
        boundary_points = refine_boundary_edges(boundary_points, max_area)?;
    }
    
    let refined_boundary_count = boundary_points.len();
    
    let mut triangulator = DelaunayTriangulator::new(boundary_points);
    
    for i in 0..refined_boundary_count {
        triangulator.add_point(i)?;
    }
    
    if let (Some(max_area), Some(min_angle)) = (request.max_area, request.min_angle) {
        triangulator.refine_interior(max_area, min_angle, refined_boundary_count)?;
    }
    
    triangulator.remove_super_triangle();
    triangulator.filter_outside_triangles(refined_boundary_count);
    
    let vertices: Vec<Point> = triangulator.points[..triangulator.points.len() - 3].to_vec();
    let mesh = Mesh::new(vertices, triangulator.triangles.clone());
    
    if let Err(e) = mesh.validate_jacobians() {
        return Err(format!("Mesh validation failed: {}", e));
    }
    
    let (min_jac, max_jac, avg_jac) = mesh.get_jacobian_stats();
    log::info!("Delaunay mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", min_jac, max_jac, avg_jac);
    
    Ok(mesh)
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