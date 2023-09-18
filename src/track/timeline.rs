pub struct Timeline {
    sample_rate: f64,
    tempo_map: Vec<TempoChange>,
    curr_sample: usize,
    curr_time: f64,
}

struct TempoChange {
    time: f64,
    sample: usize,
    samples_per_beat: f64,
}

impl Timeline {
    pub fn advance(&mut self, offset: usize) {
        self.curr_sample += offset;
        self.curr_time = self.sample_to_time(self.curr_sample);
    }

    pub fn set_tempo(&mut self, bpm: f64) {
        let samples_per_beat = self.sample_rate * 60.0 / bpm;
        if let Some(idx) = self.tempo_map.iter().position(|m| m.sample >= self.curr_sample) {
            self.tempo_map.truncate(idx);
        }
        self.tempo_map.push(TempoChange {
            time: self.curr_time,
            sample: self.curr_sample,
            samples_per_beat,
        });
    }

    pub fn curr_sample(&self) -> usize {
        self.curr_sample
    }

    pub fn time_to_sample(&self, time: f64) -> usize {
        let map = self.tempo_map.iter().take_while(|m| m.time <= time).last().unwrap();
        map.sample + ((time - map.time) * map.samples_per_beat) as usize
    }

    pub fn sample_to_time(&self, sample: usize) -> f64 {
        let map = self.tempo_map.iter().take_while(|m| m.sample <= sample).last().unwrap();
        map.time + ((sample - map.sample) as f64 / map.samples_per_beat)
    }
}
