/// Interleaves the two channels of a stereo signal.
pub fn interleave_stereo(left: &[f32], right: &[f32], output: &mut [f32]) {
    let lr = left.iter().zip(right.iter());
    for (i, (&ls, &rs)) in lr.enumerate() {
        output[2 * i] = ls;
        output[2 * i + 1] = rs;
    }
}

/// Uninterleaves the two channels of a stereo signal.
pub fn uninterleave_stereo(input: &[f32], left: &mut [f32], right: &mut [f32]) {
    for (i, samples) in input.chunks_exact(2).enumerate() {
        left[i] = samples[0];
        right[i] = samples[1];
    }
}

/// Converts a LR signal to a MS signal
pub fn leftright_to_midside(left: &[f32], right: &[f32], mid: &mut [f32], side: &mut [f32]) {
    let lr = left.iter().zip(right.iter());
    let ms = mid.iter_mut().zip(side.iter_mut());
    for ((&l, &r), (m, s)) in lr.zip(ms) {
        *m = 0.5 * (l + r);
        *s = 0.5 * (l - r);
    }
}

/// Converts a LR signal to a mono signal
pub fn leftright_to_mono(left: &[f32], right: &[f32], mono: &mut [f32]) {
    let lr = left.iter().zip(right.iter());
    for ((&l, &r), m) in lr.zip(mono.iter_mut()) {
        *m = 0.5 * (l + r);
    }
}

/// Converts a MS signal to a LR signal
pub fn midside_to_leftright(mid: &[f32], side: &[f32], left: &mut [f32], right: &mut [f32]) {
    let ms = mid.iter().zip(side.iter());
    let lr = left.iter_mut().zip(right.iter_mut());
    for ((&m, &s), (l, r)) in ms.zip(lr) {
        *l = m + s;
        *r = m - s;
    }
}

/// Converts a mono signal to a LR signal
pub fn mono_to_leftright(mono: &[f32], left: &mut [f32], right: &mut [f32]) {
    left.copy_from_slice(mono);
    right.copy_from_slice(mono);
}
