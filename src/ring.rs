pub struct RingBuffer {
    buf: Vec<f32>,
    read_idx: usize,
    write_idx: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity],
            read_idx: 0,
            write_idx: 0,
        }
    }

    /// Returns immutable slices corresponding to a wrapped range [i, j) relative to the read head.
    /// The method returns two slices that together represent the desired wrapped range.
    pub fn slice_read(&self, i: isize, j: isize) -> (&[f32], &[f32]) {
        assert!(j - i >= 0 && j - i < self.buf.len() as isize);
        let i = self.wrap_index(self.read_idx, i);
        let j = self.wrap_index(self.read_idx, j);
        if i < j {
            (&self.buf[i..j], &self.buf[j..j])
        } else {
            (&self.buf[i..], &self.buf[..j])
        }
    }

    /// Returns mutable slices corresponding to a wrapped range [i, j) relative to the write head.
    /// The method returns two slices that together represent the desired wrapped range.
    pub fn slice_write(&mut self, i: isize, j: isize) -> (&mut [f32], &mut [f32]) {
        assert!(j - i >= 0 && j - i < self.buf.len() as isize);
        let i = self.wrap_index(self.write_idx, i);
        let j = self.wrap_index(self.write_idx, j);
        if i <= j {
            (&mut self.buf[i..j], &mut [])
        } else {
            let (a, b) = self.buf.split_at_mut(i);
            (b, &mut a[..j])
        }
    }

    pub fn seek_read(&mut self, offset: isize) {
        self.read_idx = self.wrap_index(self.read_idx, offset);
    }

    pub fn seek_write(&mut self, offset: isize) {
        self.write_idx = self.wrap_index(self.write_idx, offset);
    }

    pub fn sync_read_to_write(&mut self, delay: usize) {
        self.read_idx = self.wrap_index(self.write_idx, -(delay as isize));
    }

    /// Reads samples from the end of the circular buffer and advances the read head.
    pub fn read(&mut self, samples: &mut [f32]) {
        let len = samples.len() as isize;
        let (a, b) = self.slice_read(0, len);
        samples[..a.len()].copy_from_slice(a);
        samples[a.len()..].copy_from_slice(b);
        self.seek_read(len);
    }

    /// Appends samples to the end of the circular buffer and advances the write head.
    pub fn write(&mut self, samples: &[f32]) {
        let len = samples.len() as isize;
        let (a, b) = self.slice_write(0, len);
        a.copy_from_slice(&samples[..a.len()]);
        b.copy_from_slice(&samples[a.len()..]);
        self.seek_write(len);
    }

    pub fn mutate(&mut self, len: usize, mut f: impl FnMut(usize, f32) -> f32) {
        let (a, b) = self.slice_write(0, len as isize);
        for (idx, sample) in a.iter_mut().enumerate() {
            *sample = f(idx, *sample);
        }
        for (idx, sample) in b.iter_mut().enumerate() {
            *sample = f(idx + a.len(), *sample);
        }
        self.seek_write(len as isize);
    }

    fn wrap_index(&self, base: usize, offset: isize) -> usize {
        let len = self.buf.len() as isize;
        ((base as isize + offset).rem_euclid(len)) as usize
    }
}
