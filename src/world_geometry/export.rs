use serde_json::json;
use std::{fs::File, io::Write, path::Path};
use crate::mesh::Mesh;

/// Write mesh data to binary format
pub fn write_binary_data(mesh: &Mesh, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut binary_data = Vec::new();
    
    // Write vertices (3 floats per vertex, 4 bytes per float)
    for vertex in &mesh.vertices {
        binary_data.extend_from_slice(&vertex[0].to_le_bytes());
        binary_data.extend_from_slice(&vertex[1].to_le_bytes());
        binary_data.extend_from_slice(&vertex[2].to_le_bytes());
    }
    
    // Write colors (3 floats per color, 4 bytes per float)
    for color in &mesh.colors {
        binary_data.extend_from_slice(&color[0].to_le_bytes());
        binary_data.extend_from_slice(&color[1].to_le_bytes());
        binary_data.extend_from_slice(&color[2].to_le_bytes());
    }
    
    // Write indices (4 bytes per index)
    for &index in &mesh.indices {
        binary_data.extend_from_slice(&(index as u32).to_le_bytes());
    }
    
    let mut file = File::create(filename)?;
    file.write_all(&binary_data)?;
    Ok(())
}

/// Export mesh as GLTF format
pub fn export_gltf(mesh: &Mesh, gltf_filename: &str, binary_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Write binary data first
    write_binary_data(mesh, binary_filename)?;
    
    // Calculate buffer sizes
    let vertex_buffer_size = mesh.vertices.len() * 12; // 3 floats * 4 bytes
    let color_buffer_size = mesh.colors.len() * 12; // 3 floats * 4 bytes
    let index_buffer_size = mesh.indices.len() * 4; // 1 u32 * 4 bytes
    let total_buffer_size = vertex_buffer_size + color_buffer_size + index_buffer_size;
    
    // Only keep the file name portion for the URI stored in GLTF
    let binary_uri = Path::new(binary_filename)
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or("Invalid binary filename")?;
    
    // Create GLTF JSON
    let gltf = json!({
        "asset": {
            "version": "2.0",
            "generator": "DTD_WorldGenerator"
        },
        "scene": 0,
        "scenes": [
            {
                "name": "H3_Scene",
                "nodes": [0]
            }
        ],
        "nodes": [
            {
                "name": "H3_Polyhedron_Node",
                "mesh": 0
            }
        ],
        "meshes": [
            {
                "name": "H3_Polyhedron",
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0,
                            "COLOR_0": 1
                        },
                        "indices": 2,
                        "mode": 4
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": mesh.vertices.len(),
                "type": "VEC3",
                "byteOffset": 0
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": mesh.colors.len(),
                "type": "VEC3",
                "byteOffset": 0
            },
            {
                "bufferView": 2,
                "componentType": 5125,
                "count": mesh.indices.len(),
                "type": "SCALAR",
                "byteOffset": 0
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": vertex_buffer_size,
                "byteStride": 12,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": vertex_buffer_size,
                "byteLength": color_buffer_size,
                "byteStride": 12,
                "target": 34962
            },
            {
                "buffer": 0,
                "byteOffset": vertex_buffer_size + color_buffer_size,
                "byteLength": index_buffer_size,
                "target": 34963
            }
        ],
        "buffers": [
            {
                "byteLength": total_buffer_size,
                "uri": binary_uri
            }
        ]
    });
    
    // Write GLTF file
    let gltf_string = serde_json::to_string_pretty(&gltf)?;
    std::fs::write(gltf_filename, gltf_string)?;
    
    Ok(())
} 