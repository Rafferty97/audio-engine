#[derive(Clone)]
pub struct RingBuffer {
    buffer: Box<[f32]>,
    read_idx: usize,
    write_idx: usize,
}

impl RingBuffer {
    /// Creates a ring buffer.
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size].into_boxed_slice(),
            read_idx: 0,
            write_idx: 0,
        }
    }

    /// Gets the number of samples held by this ring buffer.
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Gets the delay of the read head relative to the write head in samples.
    pub fn delay(&self) -> usize {
        let offset = self.write_idx as isize - self.read_idx as isize;
        offset.rem_euclid(self.buffer.len() as isize) as usize
    }

    /// Sets the delay of the read head to be the given number of sample behind the write head.
    pub fn seek(&mut self, delay: usize) {
        let read_idx = self.write_idx as isize - delay as isize;
        self.read_idx = read_idx.rem_euclid(self.buffer.len() as isize) as usize;
    }

    /// Reads samples from the ring buffer into `samples`, and advances the read position.
    pub fn read(&mut self, samples: &mut [f32]) {
        let buf = &self.buffer;
        let i = self.read_idx;
        let j = self.read_idx + samples.len();
        let len = buf.len();

        if j > len {
            // The slice being read wraps around the end of the ring buffer
            samples[..(len - i)].copy_from_slice(&buf[i..]);
            samples[(len - i)..].copy_from_slice(&buf[..(j - len)]);
            self.read_idx = j - len;
        } else {
            // The slice being read is contiguous in the ring buffer
            samples.copy_from_slice(&buf[i..j]);
            self.read_idx = j;
        }
    }

    /// Write samples from `samples` into the ring buffer, and advances the write position.
    pub fn write(&mut self, samples: &[f32]) {
        let buf = &mut self.buffer;
        let i = self.write_idx;
        let j = self.write_idx + samples.len();
        let len = buf.len();

        if j > len {
            // The slice being written wraps around the end of the ring buffer
            buf[i..].copy_from_slice(&samples[..(len - i)]);
            buf[..(j - len)].copy_from_slice(&samples[(len - i)..]);
            self.write_idx = j - len;
        } else {
            // The slice being written is contiguous in the ring buffer
            buf[i..j].copy_from_slice(samples);
            self.write_idx = j;
        }
    }
}
