use super::Processor;
use crate::convert::{interleave_stereo, uninterleave_stereo};
use basedrop::Handle;
use cpal::{traits::DeviceTrait, Device, Stream, StreamConfig};
use ringbuf_basedrop as ringbuf;
use std::sync::mpsc;

pub struct AudioOutput {
    channel: ringbuf::Producer<f32>,
    buffer: Vec<f32>,
    notify: mpsc::Receiver<()>,
}

impl AudioOutput {
    pub fn new(
        device: Device,
        config: &StreamConfig,
        buffer_size: usize,
        handle: &Handle,
    ) -> (Self, Stream) {
        let (tx, mut rx) = ringbuf::RingBuffer::new(buffer_size).split(handle);
        let (tx2, rx2) = mpsc::sync_channel(0);

        let stream = device
            .build_output_stream(
                config,
                move |data, _| {
                    rx.pop_slice(data);
                    tx2.try_send(()).ok();
                },
                move |err| {
                    eprintln!("an error occurred on stream: {}", err);
                },
                None,
            )
            .unwrap();

        (
            Self {
                channel: tx,
                buffer: vec![],
                notify: rx2,
            },
            stream,
        )
    }
}

impl Processor for AudioOutput {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 2,
            max_audio_ins: 2,
            num_audio_outs: 0,
        }
    }

    fn set_sample_rate(&mut self, _sample_rate: u32) {
        // Doesn't do anything
    }

    fn process(&mut self, data: super::ProcessorData) {
        let [left, right, ..] = data.audio_in else {
            panic!("Expected at least two input audio buffers");
        };

        self.buffer.resize(left.len() + right.len(), 0.0);

        interleave_stereo(left, right, &mut self.buffer[..]);

        while self.channel.remaining() < self.buffer.len() {
            self.notify.recv().unwrap();
        }
        self.channel.push_slice(&self.buffer);
    }
}

pub struct AudioInput {
    channel: ringbuf::Consumer<f32>,
    buffer: Vec<f32>,
}

impl AudioInput {
    pub fn new(
        device: Device,
        config: &StreamConfig,
        buffer_size: usize,
        handle: &Handle,
    ) -> (Self, Stream) {
        let (mut tx, rx) = ringbuf::RingBuffer::new(buffer_size).split(handle);

        let stream = device
            .build_input_stream(
                config,
                move |data, _| {
                    tx.push_slice(data);
                },
                move |err| {
                    eprintln!("an error occurred on stream: {}", err);
                },
                None,
            )
            .unwrap();

        (
            Self {
                channel: rx,
                buffer: vec![],
            },
            stream,
        )
    }
}

impl Processor for AudioInput {
    fn description(&self) -> super::ProcessorDescription {
        super::ProcessorDescription {
            min_audio_ins: 0,
            max_audio_ins: 0,
            num_audio_outs: 2,
        }
    }

    fn set_sample_rate(&mut self, _sample_rate: u32) {
        // Doesn't do anything
    }

    fn process(&mut self, data: super::ProcessorData) {
        let [left, right, ..] = data.audio_out else {
            panic!("Expected at least two output audio buffers");
        };

        self.buffer.resize(left.len() + right.len(), 0.0);

        let read = self.channel.pop_slice(&mut self.buffer);
        self.buffer[read..].fill(0.0);

        uninterleave_stereo(&self.buffer, left, right);
    }
}
