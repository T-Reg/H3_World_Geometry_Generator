use h3_world_geometry_generator::gen_world_chunks;
use std::env;

fn main() {
    match run() {
        Ok(_) => println!("Successfully created H3 chunks!"),
        Err(e) => eprintln!("Error creating mesh: {}", e),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    const SPHERE_RADIUS: f64 = 10.0;

    let (world_res, chunk_res, output_prefix) = parse_cli_args();

    println!("World resolution: {}", world_res);
    println!("Chunk resolution: {}", chunk_res);
    println!("Output prefix: {}", output_prefix);

    let stats = gen_world_chunks(SPHERE_RADIUS, world_res, chunk_res, &output_prefix)?;

    println!("\nProcessing completed:");
    println!("  - Cells processed: {}", stats.cells_processed);
    println!("  - Pentagons: {}", stats.pentagon_count);
    println!("  - Hexagons: {}", stats.hexagon_count);
    println!("  - Invalid coordinates: {}", stats.invalid_coords);

    Ok(())
}

/// Parse CLI arguments.
/// 
/// Returns `(world_resolution, chunk_resolution, output_prefix)`
/// * `world_resolution` – H3 grid resolution for geometry (defaults to 0)
/// * `chunk_resolution` – resolution used to split geometry into chunks (defaults to 0)
/// * `output_prefix` – filename prefix for exported files (defaults to "output")
fn parse_cli_args() -> (u8, u8, String) {
    let args: Vec<String> = env::args().collect();

    // First optional arg: world resolution
    let world_resolution = if args.len() > 1 {
        args[1].parse::<u8>().unwrap_or(0)
    } else {
        0
    };

    // Second optional arg: chunk resolution
    let chunk_resolution = if args.len() > 2 {
        args[2].parse::<u8>().unwrap_or(0)
    } else {
        0
    };

    // Third optional arg: output filename prefix
    let output_prefix = if args.len() > 3 {
        args[3].clone()
    } else {
        String::from("output")
    };

    (world_resolution, chunk_resolution, output_prefix)
}