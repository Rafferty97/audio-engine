/// Adapts a function generating fixed-sized blocks (`N`) of samples
/// into one that outputs variable-sized blocks by using an internal buffer.
pub struct FixedOutputAdapter<const N: usize> {
    /// Sample buffer
    buffer: [f32; N],
    /// Index of the first unread sample
    idx: usize,
}

impl<const N: usize> Default for FixedOutputAdapter<N> {
    fn default() -> Self {
        // An idx of `N` signifies an empty buffer
        Self {
            buffer: [0.0; N],
            idx: N,
        }
    }
}

impl<const N: usize> FixedOutputAdapter<N> {
    /// Creates a new [`FixedOutputAdapter`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Fills `output` by repeatedly calling `factory` and buffering as needed.
    ///
    /// # Parameters
    /// - `output` - The output buffer to write samples to, which can be of any length.
    /// - `factory` - The function which produces samples, which is passed a buffer of length `N` to be filled.
    pub fn fill(&mut self, mut output: &mut [f32], mut factory: impl FnMut(&mut [f32])) {
        loop {
            // Write samples from the internal buffer to the output buffer
            let written = self.read(output);
            output = &mut output[written..];

            // Break when the output buffer is full
            if output.is_empty() {
                break;
            }

            // Fill the internal buffer using the provided function
            (factory)(&mut self.buffer);
            self.idx = 0;
        }
    }

    /// Copies samples from `buffer` into `output` and returns number written.
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        if self.idx >= N {
            return 0;
        }

        let idx = self.idx;
        let next_idx = usize::min(self.idx + output.len(), N);
        let len = next_idx - self.idx;
        output[..len].copy_from_slice(&self.buffer[idx..next_idx]);
        self.idx = next_idx;
        len
    }

    /// Fills the internal buffer with silence
    pub fn write_silence(&mut self) {
        self.buffer.fill(0.0);
        self.idx = 0;
    }
}
