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
    pub fn delay(&mut self) -> usize {
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
            buf[i..j].copy_from_slice(&samples);
            self.write_idx = j;
        }
    }

    // /// Returns immutable slices corresponding to a wrapped range [i, j) relative to the read head.
    // /// The method returns two slices that together represent the desired wrapped range.
    // pub fn slice_read(&self, i: isize, j: isize) -> (&[f32], &[f32]) {
    //     assert!(j - i >= 0 && j - i < self.buffer.len() as isize);
    //     let i = self.wrap_index(self.read_idx, i);
    //     let j = self.wrap_index(self.read_idx, j);
    //     if i < j {
    //         (&self.buffer[i..j], &self.buffer[j..j])
    //     } else {
    //         (&self.buffer[i..], &self.buffer[..j])
    //     }
    // }

    // /// Returns mutable slices corresponding to a wrapped range [i, j) relative to the write head.
    // /// The method returns two slices that together represent the desired wrapped range.
    // pub fn slice_write(&mut self, i: isize, j: isize) -> (&mut [f32], &mut [f32]) {
    //     assert!(j - i >= 0 && j - i < self.buffer.len() as isize);
    //     let i = self.wrap_index(self.write_idx, i);
    //     let j = self.wrap_index(self.write_idx, j);
    //     if i <= j {
    //         (&mut self.buffer[i..j], &mut [])
    //     } else {
    //         let (a, b) = self.buffer.split_at_mut(i);
    //         (b, &mut a[..j])
    //     }
    // }

    // pub fn seek_read(&mut self, offset: isize) {
    //     self.read_idx = self.wrap_index(self.read_idx, offset);
    // }

    // pub fn seek_write(&mut self, offset: isize) {
    //     self.write_idx = self.wrap_index(self.write_idx, offset);
    // }

    // pub fn sync_read_to_write(&mut self, delay: usize) {
    //     self.read_idx = self.wrap_index(self.write_idx, -(delay as isize));
    // }

    // /// Reads samples from the end of the circular buffer and advances the read head.
    // pub fn read(&mut self, samples: &mut [f32]) {
    //     let len = samples.len() as isize;
    //     let (a, b) = self.slice_read(0, len);
    //     samples[..a.len()].copy_from_slice(a);
    //     samples[a.len()..].copy_from_slice(b);
    //     self.seek_read(len);
    // }

    // /// Appends samples to the end of the circular buffer and advances the write head.
    // pub fn write(&mut self, samples: &[f32]) {
    //     let len = samples.len() as isize;
    //     let (a, b) = self.slice_write(0, len);
    //     a.copy_from_slice(&samples[..a.len()]);
    //     b.copy_from_slice(&samples[a.len()..]);
    //     self.seek_write(len);
    // }

    // pub fn mutate(&mut self, len: usize, mut f: impl FnMut(usize, f32) -> f32) {
    //     let (a, b) = self.slice_write(0, len as isize);
    //     for (idx, sample) in a.iter_mut().enumerate() {
    //         *sample = f(idx, *sample);
    //     }
    //     for (idx, sample) in b.iter_mut().enumerate() {
    //         *sample = f(idx + a.len(), *sample);
    //     }
    //     self.seek_write(len as isize);
    // }

    // fn wrap_index(&self, base: usize, offset: isize) -> usize {
    //     let len = self.buffer.len() as isize;
    //     ((base as isize + offset).rem_euclid(len)) as usize
    // }
}
