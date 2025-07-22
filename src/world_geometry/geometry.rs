/// Convert latitude/longitude to 3D coordinates on a unit sphere
pub fn lat_lng_to_3d(lat_deg: f64, lng_deg: f64, radius: f64) -> [f32; 3] {
    let lat_rad = lat_deg.to_radians();
    let lng_rad = lng_deg.to_radians();

    let x = radius * lat_rad.cos() * lng_rad.cos();
    let y = radius * lat_rad.sin(); // +Y is “up” in Godot
    let z = radius * lat_rad.cos() * lng_rad.sin();

    [x as f32, y as f32, z as f32]
}