use crate::processor::{
    AudioInput, AudioOutput, Delay, Mixer, Processor, ProcessorData, Saturator,
};
use bumpalo::Bump;
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};
use std::{
    collections::HashMap,
    hash::Hash,
    slice::{from_raw_parts, from_raw_parts_mut},
};

new_key_type! {
    pub struct DeviceId;
}

pub struct AudioEngine {
    sample_rate: u32,
    devices: SlotMap<DeviceId, Box<dyn Processor>>,
    inputs: SecondaryMap<DeviceId, Vec<(DeviceId, usize)>>,
    buffer: Vec<f32>,
    buffer_map: HashMap<(DeviceId, usize), usize>, // FIXME
    device_order: Vec<DeviceId>,                   // FIXME
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            sample_rate: 0,
            devices: SlotMap::with_key(),
            inputs: SecondaryMap::new(),
            buffer: vec![],
            buffer_map: HashMap::new(),
            device_order: vec![],
        }
    }

    pub fn test(&mut self, audio_in: AudioInput, audio_out: AudioOutput) {
        let audio_in = self.add_device(Box::new(audio_in));

        let mut delay = Delay::new();
        delay.set_delay(0.25);
        delay.set_feedback(0.5);
        delay.set_ping_pong(true);
        let delay = self.add_device(Box::new(delay));

        let saturator = Saturator::new(|s| s.clamp(-1.0, 1.0));
        let saturator = self.add_device(Box::new(saturator));

        let mixer = Mixer::new();
        let mixer = self.add_device(Box::new(mixer));

        let audio_out = self.add_device(Box::new(audio_out));

        for i in 0..2 {
            self.set_input(audio_in, i, delay, i);
            self.set_input(delay, i, saturator, i);
            self.set_input(saturator, i, mixer, i);
            self.set_input(audio_in, i, mixer, i + 2);
            self.set_input(mixer, i, audio_out, i);
        }

        self.device_order = vec![audio_in, delay, saturator, mixer, audio_out];
        self.buffer_map.clear();
        self.buffer_map.insert((audio_in, 0), 1);
        self.buffer_map.insert((audio_in, 1), 2);
        self.buffer_map.insert((delay, 0), 3);
        self.buffer_map.insert((delay, 1), 4);
        self.buffer_map.insert((saturator, 0), 5);
        self.buffer_map.insert((saturator, 1), 6);
        self.buffer_map.insert((mixer, 0), 7);
        self.buffer_map.insert((mixer, 1), 8);
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        for device in self.devices.values_mut() {
            device.set_sample_rate(sample_rate);
        }
    }

    pub fn add_device(&mut self, mut device: Box<dyn Processor>) -> DeviceId {
        if self.sample_rate > 0 {
            device.set_sample_rate(self.sample_rate);
        }
        self.devices.insert(device)
    }

    pub fn remove_device(&mut self, device_id: DeviceId) {
        self.devices.remove(device_id);
    }

    pub fn set_input(
        &mut self,
        src_device: DeviceId,
        src_channel: usize,
        dst_device: DeviceId,
        dst_channel: usize,
    ) {
        let input_map = self
            .inputs
            .entry(dst_device)
            .expect("Destination device was removed")
            .or_insert(vec![]);
        if dst_channel >= input_map.len() {
            input_map.resize(dst_channel + 1, (DeviceId::null(), 0));
        }
        input_map[dst_channel] = (src_device, src_channel);
    }

    pub fn remove_input(&mut self, dst_device: DeviceId, dst_channel: usize) {
        let input_map = self
            .inputs
            .entry(dst_device)
            .expect("Destination device was removed")
            .or_insert(vec![]);
        if let Some(slot) = input_map.get_mut(dst_channel) {
            *slot = (DeviceId::null(), 0);
        }
    }

    pub fn process(&mut self, len: usize) {
        if self.sample_rate == 0 {
            panic!("Sample rate has not been set.");
        }

        let mut bump = Bump::new();

        let num_buffers = 10; // FIXME

        self.buffer.resize(num_buffers * len, 0.0);
        self.buffer[..len].fill(0.0);

        let mut midi_out = vec![];

        for &device_id in self.device_order.iter() {
            bump.reset();

            let Some(device) = self.devices.get_mut(device_id) else {
                // FIXME: Fill outputs with silence?
                continue;
            };
            let descr = device.description();

            // Prepare audio buffers
            let inputs = self.inputs.get(device_id).map(|i| &i[..]).unwrap_or(&[]);
            let num_inputs = inputs.len().clamp(descr.min_audio_ins, descr.max_audio_ins);
            let num_outputs = descr.num_audio_outs;
            let (audio_in, audio_out) = borrow_buffers(
                &mut self.buffer,
                len,
                (0..num_inputs).map(|ch| {
                    inputs
                        .get(ch)
                        .and_then(|i| self.buffer_map.get(i))
                        .copied()
                        .unwrap_or(0)
                }),
                (0..num_outputs)
                    .map(|ch| self.buffer_map.get(&(device_id, ch)).copied().unwrap_or(0)),
                &bump,
            );

            // Prepare MIDI output
            midi_out.clear();

            device.process(ProcessorData {
                midi_in: &[],
                midi_out: &mut midi_out,
                samples: len,
                audio_in,
                audio_out,
            })
        }
    }
}

/// Borrows slices from a "master" buffer for audio input and output based on specified indices.
///
/// # Parameters
///
/// - `master`: The master buffer serving as the source for input audio and the destination for output audio.
/// - `len`: The number of samples being processed by the audio processor in one cycle.
/// - `audio_in`: An array of indices specifying which slices to borrow for audio input.
/// - `audio_out`: An array of indices specifying which slices to borrow for audio output.
/// - `bump`: A bump allocator for allocating the slices.
///
/// # Returns
///
/// A tuple containing two arrays of slices:
/// - The first element is an array of borrowed slices for audio input. These slices can be overlapping.
/// - The second element is an array of mutable borrowed slices for audio output. These slices are guaranteed to be non-overlapping.
///
/// # Panics
///
/// - If any index is out of bounds based on the master buffer size and `len`.
/// - If an attempt is made to mutably borrow the same slice more than once.
fn borrow_buffers<'a>(
    master: &'a mut [f32],
    len: usize,
    audio_in_indices: impl Iterator<Item = usize> + ExactSizeIterator,
    audio_out_indices: impl Iterator<Item = usize> + ExactSizeIterator,
    bump: &'a Bump,
) -> (&'a [&'a [f32]], &'a mut [&'a mut [f32]]) {
    // Get a mutable pointer to the start of the master buffer
    let base_ptr = master.as_mut_ptr();

    // Calculate the number of possible buffers of given length `len`
    let max_buffers = (master.len() / len).min(64);

    // Initialize a bit-mask to keep track of borrowed slices
    let mut borrow_mask = 0u64;

    // Create audio input slices
    let audio_in_slices = bump.alloc_slice_fill_iter(audio_in_indices.map(|idx| {
        assert_index_valid(idx, max_buffers, &mut borrow_mask, false);
        // Borrow the slice safely, as assured by the mask and index validation
        unsafe { from_raw_parts(base_ptr.add(len * idx), len) }
    }));

    // Create audio output slices
    let audio_out_slices = bump.alloc_slice_fill_iter(audio_out_indices.map(|idx| {
        assert_index_valid(idx, max_buffers, &mut borrow_mask, true);
        // Borrow the slice safely, as assured by the mask and index validation
        unsafe { from_raw_parts_mut(base_ptr.add(len * idx), len) }
    }));

    (audio_in_slices, audio_out_slices)
}

/// Asserts that the given index is valid and updates the borrow mask.
fn assert_index_valid(idx: usize, max_buffers: usize, borrow_mask: &mut u64, mutable: bool) {
    if idx >= max_buffers {
        panic!(
            "Buffer index {} is out of bounds; max is {}",
            idx, max_buffers
        );
    }
    if mutable && (*borrow_mask & (1 << idx) != 0) {
        panic!("Buffer at index {} is already borrowed", idx);
    }
    // Mark this buffer as borrowed
    *borrow_mask |= 1 << idx;
}
