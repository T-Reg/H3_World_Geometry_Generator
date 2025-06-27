use std::collections::HashMap;

/// Mesh data structure containing vertices, colors, and indices
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<usize>,
    vertex_map: HashMap<[i64; 3], usize>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex with deduplication based on position
    pub fn add_vertex(&mut self, vertex: [f32; 3], color: [f32; 3]) -> usize {
        // Scale coordinates to fixed precision before hashing to deduplicate vertices
        // that are very close together. At high H3 resolutions, vertices can be
        // separated by very small distances. i64 provides enough range to handle
        // the scaled coordinates while avoiding floating point precision issues.
        const SCALE: f64 = 1_000_000_000.0; // 1e9 – 1 nanometre on a unit distance

        let key = [
            (vertex[0] as f64 * SCALE) as i64,
            (vertex[1] as f64 * SCALE) as i64,
            (vertex[2] as f64 * SCALE) as i64,
        ];
        
        if let Some(&index) = self.vertex_map.get(&key) {
            index
        } else {
            let index = self.vertices.len();
            self.vertices.push(vertex);
            self.colors.push(color);
            self.vertex_map.insert(key, index);
            index
        }
    }

    /// Add a triangle to the mesh
    pub fn add_triangle(&mut self, triangle: [usize; 3]) {
        self.indices.extend_from_slice(&triangle);
    }

    /// Get mesh statistics
    pub fn stats(&self) -> MeshStats {
        MeshStats {
            vertex_count: self.vertices.len(),
            triangle_count: self.indices.len() / 3,
        }
    }
}

/// Mesh statistics
pub struct MeshStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
}

/// Triangulate a pentagon using fan triangulation from center
pub fn triangulate_pentagon(center_idx: usize, boundary_indices: &[usize]) -> Vec<[usize; 3]> {
    let mut triangles = Vec::new();
    
    for i in 0..boundary_indices.len() {
        let next = (i + 1) % boundary_indices.len();
        triangles.push([center_idx, boundary_indices[i], boundary_indices[next]]);
    }
    
    triangles
} 