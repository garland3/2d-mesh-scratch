use mesh_generator::*;
use std::env;
use std::fs;
use log::{info, error};

fn main() {
    // Initialize logging to stderr
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{} [RUST] [{}] {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Stderr)
        .init();

    let args: Vec<String> = env::args().collect();
    
    info!("RUST_BINARY_START - Args: {:?}", args);
    
    if args.len() < 2 {
        error!("RUST_BINARY_ERROR - Insufficient arguments");
        println!("Usage: {} <mode> [options]", args[0]);
        println!("Modes:");
        println!("  test - Run with example data");
        println!("  json <file> - Load geometry from JSON file");
        println!("  json-stdin - Read JSON from stdin and output JSON");
        println!("  csv-stdin - Read JSON from stdin and output CSV");
        println!("  interactive - Interactive point entry");
        return;
    }

    match args[1].as_str() {
        "test" => {
            info!("RUST_BINARY_MODE - Running test mode");
            run_test();
        }
        "json" => {
            if args.len() < 3 {
                error!("RUST_BINARY_ERROR - JSON mode requires file argument");
                println!("Usage: {} json <file>", args[0]);
                return;
            }
            info!("RUST_BINARY_MODE - Running JSON file mode with file: {}", args[2]);
            run_from_json(&args[2]);
        }
        "json-stdin" => {
            info!("RUST_BINARY_MODE - Running JSON stdin mode");
            run_from_stdin();
        }
        "csv-stdin" => {
            info!("RUST_BINARY_MODE - Running CSV stdin mode");
            run_csv_from_stdin();
        }
        "interactive" => {
            info!("RUST_BINARY_MODE - Running interactive mode");
            run_interactive();
        }
        _ => {
            error!("RUST_BINARY_ERROR - Unknown mode: {}", args[1]);
            println!("Unknown mode: {}", args[1]);
        }
    }
}

fn run_from_stdin() {
    use std::io::{self, Read};
    
    info!("RUST_STDIN_JSON - Reading input from stdin");
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input) {
        Ok(bytes_read) => {
            info!("RUST_STDIN_JSON - Read {} bytes from stdin", bytes_read);
            match serde_json::from_str::<MeshRequest>(&input) {
                Ok(request) => {
                    info!("RUST_STDIN_JSON - Parsed request with {} points, algorithm: {:?}", 
                          request.geometry.points.len(), request.algorithm);
                    match generate_mesh(request) {
                        Ok(mesh) => {
                            info!("RUST_STDIN_JSON - Generated mesh with {} vertices, {} triangles", 
                                  mesh.vertices.len(), mesh.triangle_indices.len());
                            match serde_json::to_string(&mesh) {
                                Ok(json_output) => {
                                    info!("RUST_STDIN_JSON - Serialized mesh to JSON ({} chars)", json_output.len());
                                    println!("{}", json_output);
                                }
                                Err(e) => {
                                    error!("RUST_STDIN_JSON - Error serializing mesh to JSON: {}", e);
                                    eprintln!("Error serializing mesh to JSON: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            error!("RUST_STDIN_JSON - Error generating mesh: {}", e);
                            eprintln!("Error generating mesh: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    error!("RUST_STDIN_JSON - Error parsing JSON input: {}", e);
                    eprintln!("Error parsing JSON input: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("RUST_STDIN_JSON - Error reading from stdin: {}", e);
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_csv_from_stdin() {
    use std::io::{self, Read};
    
    info!("RUST_STDIN_CSV - Reading input from stdin");
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input) {
        Ok(bytes_read) => {
            info!("RUST_STDIN_CSV - Read {} bytes from stdin", bytes_read);
            match serde_json::from_str::<MeshRequest>(&input) {
                Ok(request) => {
                    info!("RUST_STDIN_CSV - Parsed request with {} points", request.geometry.points.len());
                    match generate_mesh(request.clone()) {
                        Ok(mesh) => {
                            info!("RUST_STDIN_CSV - Generated mesh with {} vertices, {} triangles", 
                                  mesh.vertices.len(), mesh.triangle_indices.len());
                            match export_to_csv(&request.geometry, Some(&mesh)) {
                                Ok(csv_content) => {
                                    info!("RUST_STDIN_CSV - Generated CSV with {} characters", csv_content.len());
                                    print!("{}", csv_content);
                                }
                                Err(e) => {
                                    error!("RUST_STDIN_CSV - Error generating CSV: {}", e);
                                    eprintln!("Error generating CSV: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            error!("RUST_STDIN_CSV - Error generating mesh: {}", e);
                            eprintln!("Error generating mesh: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    error!("RUST_STDIN_CSV - Error parsing JSON input: {}", e);
                    eprintln!("Error parsing JSON input: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            error!("RUST_STDIN_CSV - Error reading from stdin: {}", e);
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_test() {
    println!("Running test with example data...");
    
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(4.0, 0.0),
        Point::new(4.0, 3.0),
        Point::new(0.0, 3.0),
    ];
    
    let geometry = Geometry::new(points);
    let request = MeshRequest::with_constraints(geometry, 0.5, 25.0);
    
    match generate_mesh(request) {
        Ok(mesh) => {
            println!("Mesh generated successfully!");
            println!("Vertices: {}", mesh.vertices.len());
            println!("Triangles: {}", mesh.triangle_indices.len());
            
            println!("\nVertices:");
            for (i, vertex) in mesh.vertices.iter().enumerate() {
                println!("  {}: ({:.2}, {:.2})", i, vertex.x, vertex.y);
            }
            
            println!("\nTriangles:");
            for (i, triangle) in mesh.triangle_indices.iter().enumerate() {
                println!("  {}: [{}, {}, {}]", i, triangle[0], triangle[1], triangle[2]);
            }

            let geometry_for_export = Geometry::new(mesh.vertices.clone());
            match export_to_csv(&geometry_for_export, Some(&mesh)) {
                Ok(csv_content) => {
                    match fs::write("test_mesh.csv", csv_content) {
                        Ok(_) => println!("\nCSV exported to test_mesh.csv"),
                        Err(e) => println!("Error writing CSV: {}", e),
                    }
                }
                Err(e) => println!("Error generating CSV: {}", e),
            }
        }
        Err(e) => println!("Error generating mesh: {}", e),
    }
}

fn run_from_json(filename: &str) {
    println!("Loading geometry from {}...", filename);
    
    match fs::read_to_string(filename) {
        Ok(content) => {
            match serde_json::from_str::<MeshRequest>(&content) {
                Ok(request) => {
                    match generate_mesh(request) {
                        Ok(mesh) => {
                            println!("Mesh generated successfully!");
                            println!("Vertices: {}", mesh.vertices.len());
                            println!("Triangles: {}", mesh.triangle_indices.len());
                            
                            let geometry_for_export = Geometry::new(mesh.vertices.clone());
                            match export_to_csv(&geometry_for_export, Some(&mesh)) {
                                Ok(csv_content) => {
                                    let output_file = format!("{}_mesh.csv", filename.replace(".json", ""));
                                    match fs::write(&output_file, csv_content) {
                                        Ok(_) => println!("CSV exported to {}", output_file),
                                        Err(e) => println!("Error writing CSV: {}", e),
                                    }
                                }
                                Err(e) => println!("Error generating CSV: {}", e),
                            }
                        }
                        Err(e) => println!("Error generating mesh: {}", e),
                    }
                }
                Err(e) => println!("Error parsing JSON: {}", e),
            }
        }
        Err(e) => println!("Error reading file: {}", e),
    }
}

fn run_interactive() {
    println!("Interactive mode - Enter points (x, y). Type 'done' when finished.");
    
    let mut points = Vec::new();
    let mut input = String::new();
    
    loop {
        input.clear();
        println!("Enter point {} (x y) or 'done':", points.len() + 1);
        
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let line = input.trim();
                if line == "done" {
                    break;
                }
                
                let coords: Vec<&str> = line.split_whitespace().collect();
                if coords.len() == 2 {
                    match (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                        (Ok(x), Ok(y)) => {
                            points.push(Point::new(x, y));
                            println!("Added point: ({}, {})", x, y);
                        }
                        _ => println!("Invalid coordinates. Please enter two numbers."),
                    }
                } else {
                    println!("Please enter two numbers separated by space.");
                }
            }
            Err(e) => {
                println!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    if points.len() < 3 {
        println!("Need at least 3 points to generate a mesh.");
        return;
    }
    
    println!("Enter max area (default 0.1):");
    input.clear();
    let max_area = match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let line = input.trim();
            if line.is_empty() {
                0.1
            } else {
                line.parse::<f64>().unwrap_or(0.1)
            }
        }
        Err(_) => 0.1,
    };
    
    println!("Enter min angle (default 20.0):");
    input.clear();
    let min_angle = match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            let line = input.trim();
            if line.is_empty() {
                20.0
            } else {
                line.parse::<f64>().unwrap_or(20.0)
            }
        }
        Err(_) => 20.0,
    };
    
    let geometry = Geometry::new(points);
    let request = MeshRequest::with_constraints(geometry, max_area, min_angle);
    
    match generate_mesh(request) {
        Ok(mesh) => {
            println!("Mesh generated successfully!");
            println!("Vertices: {}", mesh.vertices.len());
            println!("Triangles: {}", mesh.triangle_indices.len());
            
            let geometry_for_export = Geometry::new(mesh.vertices.clone());
            match export_to_csv(&geometry_for_export, Some(&mesh)) {
                Ok(csv_content) => {
                    match fs::write("interactive_mesh.csv", csv_content) {
                        Ok(_) => println!("CSV exported to interactive_mesh.csv"),
                        Err(e) => println!("Error writing CSV: {}", e),
                    }
                }
                Err(e) => println!("Error generating CSV: {}", e),
            }
        }
        Err(e) => println!("Error generating mesh: {}", e),
    }
}