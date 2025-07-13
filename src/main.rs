use std::env;
use std::io::{self, Read};
use serde_json;

mod geometry;
mod mesher;

use geometry::Point;
use mesher::MeshCore;

#[derive(serde::Deserialize)]
struct CliInput {
    geometry: Geometry,
    density: Option<f64>,
    refine_metric: Option<String>,
    refine_threshold: Option<f64>,
    refine_iterations: Option<usize>,
    smooth_iterations: Option<usize>,
}

#[derive(serde::Deserialize)]
struct Geometry {
    points: Vec<Point>,
    name: Option<String>,
}

#[derive(serde::Serialize)]
struct MeshOutput {
    points: Vec<Point>,
    triangles: Vec<[usize; 3]>,
    stats: MeshStats,
}

#[derive(serde::Serialize)]
struct MeshStats {
    point_count: usize,
    triangle_count: usize,
    avg_angle_quality: f64,
    avg_aspect_quality: f64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "test" => run_test(),
        "json" => {
            if args.len() < 3 {
                eprintln!("Error: json mode requires a filename");
                print_usage();
                return;
            }
            run_json_file(&args[2]);
        }
        "json-stdin" => run_json_stdin(),
        "interactive" => run_interactive(),
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Mesh Generator CLI Tool");
    println!("Usage:");
    println!("  mesher test                    - Run with example data");
    println!("  mesher json <file>             - Process JSON from file");
    println!("  mesher json-stdin              - Process JSON from stdin");
    println!("  mesher interactive             - Interactive mode");
    println!();
    println!("JSON Input Format:");
    println!("{{");
    println!("  \"geometry\": {{");
    println!("    \"points\": [{{\"x\": 0, \"y\": 0}}, {{\"x\": 1, \"y\": 0}}, ...],");
    println!("    \"name\": \"rectangle\"");
    println!("  }},");
    println!("  \"density\": 0.1,");
    println!("  \"refine_metric\": \"angle\",");
    println!("  \"refine_threshold\": 20.0,");
    println!("  \"refine_iterations\": 100,");
    println!("  \"smooth_iterations\": 5");
    println!("}}");
}

fn run_test() {
    println!("Running mesh generator test...");
    
    let test_points = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(0.0, 1.0),
    ];
    
    let mut mesher = MeshCore::new();
    mesher.add_polygon_from_points(&test_points);
    
    if mesher.generate_mesh(0.1) {
        println!("Mesh generated successfully!");
        
        mesher.refine_mesh("angle", 20.0, 50);
        mesher.smooth_mesh(3);
        
        let stats = MeshStats {
            point_count: mesher.points.len(),
            triangle_count: mesher.triangles.len(),
            avg_angle_quality: mesher.get_average_quality("angle"),
            avg_aspect_quality: mesher.get_average_quality("aspect"),
        };
        
        println!("Mesh Statistics:");
        println!("  Points: {}", stats.point_count);
        println!("  Triangles: {}", stats.triangle_count);
        println!("  Avg Min Angle: {:.2}°", stats.avg_angle_quality);
        println!("  Avg Aspect Ratio: {:.2}", stats.avg_aspect_quality);
    } else {
        eprintln!("Failed to generate mesh");
    }
}

fn run_json_file(filename: &str) {
    match std::fs::read_to_string(filename) {
        Ok(content) => process_json_input(&content),
        Err(e) => eprintln!("Error reading file '{}': {}", filename, e),
    }
}

fn run_json_stdin() {
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input) {
        Ok(_) => process_json_input(&input.trim()),
        Err(e) => eprintln!("Error reading from stdin: {}", e),
    }
}

fn process_json_input(input: &str) {
    let cli_input: CliInput = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            return;
        }
    };
    
    let mut mesher = MeshCore::new();
    mesher.add_polygon_from_points(&cli_input.geometry.points);
    
    let density = cli_input.density.unwrap_or(0.1);
    
    if !mesher.generate_mesh(density) {
        eprintln!("Failed to generate mesh");
        return;
    }
    
    if let Some(metric) = cli_input.refine_metric {
        let threshold = cli_input.refine_threshold.unwrap_or(if metric == "angle" { 20.0 } else { 2.0 });
        let iterations = cli_input.refine_iterations.unwrap_or(100);
        mesher.refine_mesh(&metric, threshold, iterations);
    }
    
    if let Some(smooth_iters) = cli_input.smooth_iterations {
        mesher.smooth_mesh(smooth_iters);
    }
    
    let triangles_data: Vec<[usize; 3]> = mesher.triangles.iter()
        .map(|t| t.indices)
        .collect();
    
    let output = MeshOutput {
        points: mesher.points.clone(),
        triangles: triangles_data,
        stats: MeshStats {
            point_count: mesher.points.len(),
            triangle_count: mesher.triangles.len(),
            avg_angle_quality: mesher.get_average_quality("angle"),
            avg_aspect_quality: mesher.get_average_quality("aspect"),
        },
    };
    
    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing output: {}", e),
    }
}

fn run_interactive() {
    println!("Interactive Mesh Generator");
    println!("Enter polygon points (x y) on separate lines, or 'done' to finish:");
    
    let mut points = Vec::new();
    
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input == "done" {
                    break;
                }
                
                let coords: Vec<&str> = input.split_whitespace().collect();
                if coords.len() == 2 {
                    match (coords[0].parse::<f64>(), coords[1].parse::<f64>()) {
                        (Ok(x), Ok(y)) => {
                            points.push(Point::new(x, y));
                            println!("Added point ({}, {})", x, y);
                        }
                        _ => println!("Invalid coordinates. Please enter two numbers."),
                    }
                } else {
                    println!("Please enter x y coordinates or 'done'");
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    if points.len() < 3 {
        eprintln!("Need at least 3 points to create a mesh");
        return;
    }
    
    println!("Enter mesh density (default 0.1):");
    let mut input = String::new();
    let density = match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().parse().unwrap_or(0.1),
        Err(_) => 0.1,
    };
    
    let mut mesher = MeshCore::new();
    mesher.add_polygon_from_points(&points);
    
    if mesher.generate_mesh(density) {
        println!("Mesh generated!");
        println!("Points: {}, Triangles: {}", mesher.points.len(), mesher.triangles.len());
        
        let triangles_data: Vec<[usize; 3]> = mesher.triangles.iter()
            .map(|t| t.indices)
            .collect();
        
        let output = MeshOutput {
            points: mesher.points.clone(),
            triangles: triangles_data,
            stats: MeshStats {
                point_count: mesher.points.len(),
                triangle_count: mesher.triangles.len(),
                avg_angle_quality: mesher.get_average_quality("angle"),
                avg_aspect_quality: mesher.get_average_quality("aspect"),
            },
        };
        
        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing output: {}", e),
        }
    } else {
        eprintln!("Failed to generate mesh");
    }
}