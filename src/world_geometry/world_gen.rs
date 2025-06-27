use h3o::{CellIndex, LatLng};
use crate::{mesh::{Mesh, triangulate_pentagon}, geometry::lat_lng_to_3d};
use std::path::Path;
use super::export::export_gltf;

/// Statistics about H3 processing
#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub pentagon_count: usize,
    pub hexagon_count: usize,
    pub invalid_coords: usize,
    pub cells_processed: usize,
}

/// Generate a world geometry mesh
pub fn gen_world_geometry(
    sphere_radius: f64,
    resolution: u8,
) -> Result<(Mesh, ProcessingStats), Box<dyn std::error::Error>> {
    let res_enum = h3o::Resolution::try_from(resolution)
        .map_err(|_| format!("Invalid H3 resolution: {} (must be 0..=15)", resolution))?;

    // Collect all cell indices for the requested resolution.
    let cells: Vec<CellIndex> = if resolution == 0 {
        CellIndex::base_cells().collect()
    } else {
        // For higher resolutions, gather children of each base cell.
        CellIndex::base_cells()
            .flat_map(|base| base.children(res_enum))
            .collect()
    };

    println!(
        "Generating geometry for {} cells at resolution {}",
        cells.len(), resolution
    );

    let mut mesh = Mesh::new();
    let mut stats = ProcessingStats::default();

    println!("Processing H3 cells...");

    // Iterate over the H3 cells
    for (i, cell) in cells.iter().enumerate() {
        if let Err(e) = process_single_cell(&mut mesh, &mut stats, *cell, i, sphere_radius) {
            eprintln!("Error processing cell {}: {}", i, e);
            continue;
        }
        stats.cells_processed += 1;
    }

    eprintln!("Pentagons: {}, Hexagons: {}", stats.pentagon_count, stats.hexagon_count);
    eprintln!("Invalid coordinates: {}", stats.invalid_coords);

    let mesh_stats = mesh.stats();
    eprintln!(
        "Generated {} vertices and {} triangles",
        mesh_stats.vertex_count, mesh_stats.triangle_count
    );

    Ok((mesh, stats))
}

/// Process a single H3 cell and add it to the mesh
pub fn process_single_cell(
    mesh: &mut Mesh, 
    stats: &mut ProcessingStats, 
    cell: CellIndex, 
    cell_index: usize, 
    sphere_radius: f64
) -> Result<(), Box<dyn std::error::Error>> {
    if cell.is_pentagon() {
        stats.pentagon_count += 1;
    } else {
        stats.hexagon_count += 1;
    }
    
    let center_latlng = LatLng::from(cell);
    let center_3d = lat_lng_to_3d(center_latlng.lat(), center_latlng.lng(), sphere_radius);
    
    // Check for invalid coordinates
    if !center_3d.iter().all(|&x| x.is_finite()) {
        eprintln!("Invalid center coordinate for cell {}: {:?} (lat: {}, lng: {})", 
                 cell_index, center_3d, center_latlng.lat(), center_latlng.lng());
        stats.invalid_coords += 1;
        return Err("Invalid center coordinates".into());
    }
    
    let center_idx = mesh.add_vertex(center_3d, [1.0; 3]);
    
    // Get the boundary vertices
    let vertex_indices: Vec<_> = cell.vertexes().collect();
    let mut boundary_indices = Vec::new();
    
    for (j, vertex_index) in vertex_indices.iter().enumerate() {
        let vertex_latlng = LatLng::from(*vertex_index);
        let vertex_3d = lat_lng_to_3d(vertex_latlng.lat(), vertex_latlng.lng(), sphere_radius);
        
        // Check for invalid coordinates
        if !vertex_3d.iter().all(|&x| x.is_finite()) {
            eprintln!("Invalid vertex coordinate for cell {} vertex {}: {:?} (lat: {}, lng: {})", 
                     cell_index, j, vertex_3d, vertex_latlng.lat(), vertex_latlng.lng());
            stats.invalid_coords += 1;
            continue;
        }
        
        let vertex_idx = mesh.add_vertex(vertex_3d, [1.0; 3]);
        boundary_indices.push(vertex_idx);
    }
    
    // Only triangulate if we have valid boundary vertices
    if boundary_indices.len() >= 3 {
        // Triangulate the pentagon/hexagon
        let triangles = triangulate_pentagon(center_idx, &boundary_indices);
        for triangle in triangles {
            mesh.add_triangle(triangle);
        }
    } else {
        eprintln!("Cell {} has insufficient boundary vertices: {} vertices", cell_index, boundary_indices.len());
        return Err("Insufficient boundary vertices".into());
    }
    
    Ok(())
}

/// Generate the world geometry split into chunks and write each chunk to disk.
///
/// * `sphere_radius` – radius of the sphere used when projecting vertices.
/// * `world_res` – resolution at which geometry is generated.
/// * `chunk_res` – coarser resolution used to split work into chunks (must be < world_res).
/// * `output_prefix` – folder and filename prefix for exported files. Results are written to `output/<output_prefix>/`.
///
/// The function prints progress to stdout and returns aggregated statistics on success.
pub fn gen_world_chunks(
    sphere_radius: f64,
    world_res: u8,
    chunk_res: u8,
    output_prefix: &str,
) -> Result<ProcessingStats, Box<dyn std::error::Error>> {
    if chunk_res >= world_res {
        return Err(format!(
            "Chunk resolution ({}) must be lower than world resolution ({})",
            chunk_res, world_res
        ).into());
    }

    let chunk_res_enum = h3o::Resolution::try_from(chunk_res)
        .map_err(|_| format!("Invalid chunk resolution: {} (must be 0..=15)", chunk_res))?;
    let world_res_enum = h3o::Resolution::try_from(world_res)
        .map_err(|_| format!("Invalid world resolution: {} (must be 0..=15)", world_res))?;

    // Gather chunk cells at `chunk_res`.
    let chunk_cells: Vec<CellIndex> = if chunk_res == 0 {
        CellIndex::base_cells().collect()
    } else {
        CellIndex::base_cells()
            .flat_map(|base| base.children(chunk_res_enum))
            .collect()
    };

    let total_chunks = chunk_cells.len();

    // Count total cells across all chunks (for progress).
    let total_cells: usize = chunk_cells
        .iter()
        .map(|c| c.children(world_res_enum).count())
        .sum();

    println!(
        "Preparing to process {} chunks (res {} -> {}) containing {} cells",
        total_chunks, chunk_res, world_res, total_cells
    );

    // Always write generated assets into the top-level `output/<prefix>` folder so that the whole
    // directory can be safely ignored by version control (e.g. via .gitignore) while still grouping
    // different runs (or prefixes) into separate sub-folders.
    let output_dir = Path::new("output").join(output_prefix);
    std::fs::create_dir_all(&output_dir)?;

    let mut processed_cells_total = 0usize;
    let mut global_stats = ProcessingStats::default();

    for (chunk_idx, chunk_cell) in chunk_cells.iter().enumerate() {
        let children: Vec<CellIndex> = chunk_cell.children(world_res_enum).collect();
        let total_cells_in_chunk = children.len();

        println!(
            "\n=== Processing chunk {}/{} ({}) ===",
            chunk_idx + 1,
            total_chunks,
            total_cells_in_chunk
        );

        let mut mesh = Mesh::new();
        let mut chunk_stats = ProcessingStats::default();

        for (cell_idx, world_cell) in children.iter().enumerate() {
            if let Err(e) = process_single_cell(&mut mesh, &mut chunk_stats, *world_cell, cell_idx, sphere_radius) {
                eprintln!("Error processing cell {} in chunk {}: {}", cell_idx, chunk_idx, e);
            } else {
                chunk_stats.cells_processed += 1;
            }

            // Print 25% progress intervals within chunk.
            let progress_cells = cell_idx + 1;
            if progress_cells % (total_cells_in_chunk.max(4) / 4) == 0
                || progress_cells == total_cells_in_chunk
            {
                println!(
                    "  - Chunk progress: {}/{} ({:.0}%)",
                    progress_cells,
                    total_cells_in_chunk,
                    100.0 * progress_cells as f32 / total_cells_in_chunk as f32
                );
            }
        }

        // Update global stats
        global_stats.pentagon_count += chunk_stats.pentagon_count;
        global_stats.hexagon_count += chunk_stats.hexagon_count;
        global_stats.invalid_coords += chunk_stats.invalid_coords;
        global_stats.cells_processed += chunk_stats.cells_processed;

        // Export chunk mesh
        let file_stem = format!("{}-chunk{}", output_prefix, chunk_idx + 1);
        let gltf_path = output_dir.join(format!("{}.gltf", file_stem));
        let bin_path = output_dir.join(format!("{}.bin", file_stem));

        export_gltf(&mesh, gltf_path.to_str().unwrap(), bin_path.to_str().unwrap())?;
        println!("  -> Exported {} & {}", gltf_path.display(), bin_path.display());

        processed_cells_total += total_cells_in_chunk;
        println!(
            "Overall progress: {}/{} ({:.2}%)",
            processed_cells_total,
            total_cells,
            100.0 * processed_cells_total as f64 / total_cells as f64
        );
    }

    Ok(global_stats)
} 