#[inline]
pub fn scale_samples(buffer: &mut [f32], scale: f32) {
    for sample in buffer.iter_mut() {
        *sample *= scale;
    }
}

#[inline]
pub fn clip_samples(buffer: &mut [f32], min: f32, max: f32) {
    for sample in buffer.iter_mut() {
        *sample = sample.clamp(min, max);
    }
}

#[inline]
pub fn copy_samples(buffer: &mut [f32], other: &[f32]) {
    buffer.copy_from_slice(other);
}

#[inline]
pub fn copy_scaled_samples(buffer: &mut [f32], other: &[f32], scale: f32) {
    assert!(buffer.len() == other.len());
    for (sample, other) in buffer.iter_mut().zip(other.iter()) {
        *sample = scale * other;
    }
}

#[inline]
pub fn add_samples(buffer: &mut [f32], other: &[f32]) {
    assert!(buffer.len() == other.len());
    for (sample, other) in buffer.iter_mut().zip(other.iter()) {
        *sample += other;
    }
}

#[inline]
pub fn add_scaled_samples(buffer: &mut [f32], other: &[f32], scale: f32) {
    assert!(buffer.len() == other.len());
    for (sample, other) in buffer.iter_mut().zip(other.iter()) {
        *sample += scale * other;
    }
}
