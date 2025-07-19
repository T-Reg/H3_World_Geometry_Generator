

/// Mesh data structure containing vertices, colors, and indices
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub uvs0: Vec<[f32; 2]>,
    pub indices: Vec<usize>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex(&mut self, vertex: [f32; 3], uv: [f32; 2]) -> usize {
        let index = self.vertices.len();
        self.vertices.push(vertex);

        self.uvs0.push(uv);

        index
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