use crate::processor::{Processor, ProcessorData};

/// An audio processor pipeline
pub struct Pipeline {
    components: Vec<Box<dyn Processor + Send>>,
    buffer: Vec<f32>,
}

impl Pipeline {
    pub fn new(components: impl IntoIterator<Item = Box<dyn Processor + Send>>) -> Self {
        Self {
            components: components.into_iter().collect(),
            buffer: vec![],
        }
    }
}

impl Processor for Pipeline {
    fn set_sample_rate(&mut self, sample_rate: u32) {
        for component in &mut self.components {
            component.set_sample_rate(sample_rate);
        }
    }

    fn process(&mut self, data: ProcessorData) {
        // Ensure buffer is large enough
        let len = data.audio_out[0].len();
        self.buffer.resize(4 * len, 0.0);

        // Split buffer into two stereo pairs for double buffering
        let (mut buffer_a, mut buffer_b) = self.buffer.split_at_mut(2 * len);

        // Initialize buffer_a from input
        copy_or_clear(data.audio_in.get(0).copied(), &mut buffer_a[0..len]);
        copy_or_clear(data.audio_in.get(1).copied(), &mut buffer_a[len..2 * len]);

        // Setup buffers for MIDI
        let mut midi_current = data.midi_in.to_vec();
        let mut midi_next = Vec::new();

        // Process each component in the pipeline
        for component in &mut self.components {
            let (current_left, current_right) = buffer_a.split_at_mut(len);
            let (next_left, next_right) = buffer_b.split_at_mut(len);

            component.process(ProcessorData {
                midi_in: &midi_current,
                midi_out: &mut midi_next,
                samples: len,
                audio_in: &[current_left, current_right],
                audio_out: &mut [next_left, next_right],
            });

            // Swap buffers and MIDI vectors
            std::mem::swap(&mut buffer_a, &mut buffer_b);
            std::mem::swap(&mut midi_current, &mut midi_next);
            midi_next.clear();
        }

        // Copy results from buffer_a to output
        data.audio_out[0].copy_from_slice(&buffer_a[0..len]);
        data.audio_out[1].copy_from_slice(&buffer_a[len..2 * len]);
        data.midi_out.extend(midi_current.iter().cloned());
    }
}

fn copy_or_clear(src: Option<&[f32]>, dst: &mut [f32]) {
    if let Some(buffer) = src {
        dst.copy_from_slice(buffer);
    } else {
        dst.fill(0.0);
    }
}
