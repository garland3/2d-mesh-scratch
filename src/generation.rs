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
    
    let max_iterations = request.annealing_options.as_ref()
        .and_then(|opts| opts.max_iterations)
        .unwrap_or(10000);
    
    let mut annealing_generator = if let Some(ref annealing_options) = request.annealing_options {
        let temperature = annealing_options.temperature.unwrap_or(1000.0);
        let cooling_rate = annealing_options.cooling_rate.unwrap_or(0.995);
        let quality_threshold = annealing_options.quality_threshold.unwrap_or(quality_threshold);
        
        SimulatedAnnealingMeshGenerator::with_options(
            request.geometry.points, 
            temperature,
            cooling_rate,
            quality_threshold
        )
    } else {
        SimulatedAnnealingMeshGenerator::new(
            request.geometry.points, 
            quality_threshold
        )
    };
    
    annealing_generator.generate_mesh_with_iterations(target_area, max_iterations)
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
    let boundary_points = request.geometry.points;
    
    // Use hexagonal grid approach for better quality triangulation
    let points = if let Some(max_area) = request.max_area {
        let target_edge_length = (4.0 * max_area / (3.0_f64).sqrt()).sqrt();
        DelaunayTriangulator::generate_hexagonal_grid(&boundary_points, target_edge_length)
    } else {
        // Fallback to boundary points only
        boundary_points.clone()
    };
    
    let mut triangulator = DelaunayTriangulator::new(points);
    let mesh = triangulator.triangulate()?;
    
    // Filter triangles outside boundary
    let filtered_mesh = filter_mesh_outside_boundary(mesh, &boundary_points)?;
    
    if let Err(e) = filtered_mesh.validate_jacobians() {
        return Err(format!("Mesh validation failed: {}", e));
    }
    
    let (min_jac, max_jac, avg_jac) = filtered_mesh.get_jacobian_stats();
    log::info!("Delaunay mesh Jacobian stats - Min: {:.6}, Max: {:.6}, Avg: {:.6}", min_jac, max_jac, avg_jac);
    
    Ok(filtered_mesh)
}

fn filter_mesh_outside_boundary(mut mesh: Mesh, boundary: &[Point]) -> Result<Mesh, String> {
    // Filter triangles whose centroids are outside the boundary
    let mut filtered_triangles = Vec::new();
    let mut filtered_indices = Vec::new();
    
    for (i, triangle_indices) in mesh.triangle_indices.iter().enumerate() {
        let p1 = &mesh.vertices[triangle_indices[0]];
        let p2 = &mesh.vertices[triangle_indices[1]];
        let p3 = &mesh.vertices[triangle_indices[2]];
        
        let centroid = Point::new(
            (p1.x + p2.x + p3.x) / 3.0,
            (p1.y + p2.y + p3.y) / 3.0,
        );
        
        if is_point_inside_polygon(&centroid, boundary) {
            filtered_triangles.push(mesh.triangles[i].clone());
            filtered_indices.push(*triangle_indices);
        }
    }
    
    Ok(Mesh {
        vertices: mesh.vertices,
        triangles: filtered_triangles,
        triangle_indices: filtered_indices,
        quads: None,
        quad_indices: None,
    })
}

fn is_point_inside_polygon(point: &Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = &polygon[i];
        let pj = &polygon[j];
        
        if ((pi.y > point.y) != (pj.y > point.y)) &&
           (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
            inside = !inside;
        }
        j = i;
    }
    
    inside
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