use super::voice::VoiceOpts;

#[derive(Clone, Copy)]
pub struct AdsrEnvelope {
    /// Duration of a sample in seconds.
    inv_sample_rate: f32,
    /// Attack time in seconds.
    attack: f32,
    /// Decay time in seconds.
    decay: f32,
    /// Inverted sustain level between 1 and 0.
    sustain: f32,
    /// Release time in seconds.
    release: f32,
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
            inv_sample_rate: 1.0 / (opts.sample_rate as f32),
            attack: opts.attack.max(0.0),
            decay: opts.decay.max(0.0),
            sustain: opts.sustain.clamp(0.0, 1.0),
            release: opts.release.max(0.0),
            state: AdsrState::Inactive,
            amp: 0.0,
        }
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
                t += self.inv_sample_rate;
                if t < 1.0 {
                    self.state = Attack { start, t };
                } else {
                    self.state = Decay { t: t - 1.0 };
                }
            }
            Decay { mut t } => {
                self.amp = 1.0 - t * (1.0 - self.sustain);
                t += self.inv_sample_rate;
                if t < 1.0 {
                    self.state = Decay { t };
                } else {
                    self.state = Sustain;
                }
            }
            Sustain => self.amp = self.sustain,
            Release { start, mut t } => {
                self.amp = start * (1.0 - t);
                t += self.inv_sample_rate;
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
