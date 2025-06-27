use std::collections::HashMap;

/// Mesh data structure containing vertices, colors, and indices
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub indices: Vec<usize>,
    vertex_map: HashMap<[i32; 3], usize>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex with deduplication based on position
    pub fn add_vertex(&mut self, vertex: [f32; 3], color: [f32; 3]) -> usize {
        // Convert to fixed precision for hashing (to handle floating point precision issues)
        let key = [
            (vertex[0] * 1000000.0) as i32,
            (vertex[1] * 1000000.0) as i32,
            (vertex[2] * 1000000.0) as i32,
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