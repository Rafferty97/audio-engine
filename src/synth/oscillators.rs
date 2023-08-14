use std::f32::consts::PI;

pub fn sine(phase: f32) -> f32 {
    (2.0 * PI * phase).sin()
}

pub fn square(phase: f32) -> f32 {
    if phase > 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn tri(phase: f32) -> f32 {
    (4.0 * phase - 2.0).abs() + 1.0
}

pub fn saw(phase: f32) -> f32 {
    2.0 * phase - 1.0
}
