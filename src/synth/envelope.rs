use crate::constants::DEFAULT_SAMPLE_RATE;

use super::voice::VoiceOpts;

#[derive(Clone, Copy)]
pub struct AdsrEnvelope {
    /// Duration of a sample in seconds.
    inv_sample_rate: f32,
    /// Attack rate in inverse seconds.
    inv_attack: f32,
    /// Decay rate in inverse seconds.
    inv_decay: f32,
    /// Inverted sustain level between 1 and 0.
    sustain: f32,
    /// Release rate in inverse seconds.
    inv_release: f32,
    /// The current envelope state.
    state: AdsrState,
    /// The current amplitude.0
    amp: f32,
}

#[derive(Clone, Copy)]
enum AdsrState {
    Attack {
        /// The amplitude the attack phase started at
        start: f32,
        /// The progress of the attack phase between 0 and 1.
        t: f32,
    },
    Decay {
        /// The progress of the decay phase between 0 and 1.
        t: f32,
    },
    Sustain,
    Release {
        /// The amplitude the release phase started at
        start: f32,
        /// The progress of the release phase between 0 and 1.
        t: f32,
    },
    Inactive,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdsrPhase {
    Active,
    Released,
    Inactive,
}

impl AdsrEnvelope {
    pub fn new(opts: VoiceOpts) -> Self {
        Self {
            inv_sample_rate: 1.0 / (DEFAULT_SAMPLE_RATE as f32),
            inv_attack: opts.attack.max(0.0001).recip(),
            inv_decay: opts.decay.max(0.0001).recip(),
            sustain: opts.sustain.clamp(0.0, 1.0),
            inv_release: opts.release.max(0.0001).recip(),
            state: AdsrState::Inactive,
            amp: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inv_sample_rate = (sample_rate as f32).recip();
    }

    pub fn phase(&self) -> AdsrPhase {
        match self.state {
            AdsrState::Attack { .. } => AdsrPhase::Active,
            AdsrState::Decay { .. } => AdsrPhase::Active,
            AdsrState::Sustain => AdsrPhase::Active,
            AdsrState::Release { .. } => AdsrPhase::Released,
            AdsrState::Inactive => AdsrPhase::Inactive,
        }
    }

    pub fn trigger(&mut self) {
        self.state = AdsrState::Attack {
            start: self.amp,
            t: 0.0,
        };
    }

    pub fn release(&mut self) {
        self.state = AdsrState::Release {
            start: self.amp,
            t: 0.0,
        };
    }

    pub fn process(&mut self) -> f32 {
        use AdsrState::*;
        match self.state {
            Attack { start, mut t } => {
                self.amp = start + (1.0 - start) * t;
                t += self.inv_attack * self.inv_sample_rate;
                if t < 1.0 {
                    self.state = Attack { start, t };
                } else {
                    self.state = Decay { t: t - 1.0 };
                }
            }
            Decay { mut t } => {
                self.amp = 1.0 - t * (1.0 - self.sustain);
                t += self.inv_decay * self.inv_sample_rate;
                if t < 1.0 {
                    self.state = Decay { t };
                } else {
                    self.state = Sustain;
                }
            }
            Sustain => self.amp = self.sustain,
            Release { start, mut t } => {
                self.amp = start * (1.0 - t);
                t += self.inv_release * self.inv_sample_rate;
                if t < 1.0 {
                    self.state = Release { start, t };
                } else {
                    self.state = Inactive;
                }
            }
            Inactive => self.amp = 0.0,
        }

        self.amp
    }
}