use rand::Rng;

/// Generate a random bright color
pub fn generate_random_color() -> [f32; 3] {
    let mut rng = rand::rng();
    [
        rng.random_range(0.3..1.0), // Red
        rng.random_range(0.3..1.0), // Green  
        rng.random_range(0.3..1.0), // Blue
    ]
} 