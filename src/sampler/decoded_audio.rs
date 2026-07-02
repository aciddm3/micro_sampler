#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub samples: Vec<Vec<f32>>,
    sample_rate: f32,
    current_channel : usize,
    sample_length_in_seconds : f32,
}

impl DecodedAudio {
    pub fn new (samples : Vec<Vec<f32>>, sample_rate : f32) -> Self {
        let current_channel = 0;
        let sample_length_in_seconds = samples[current_channel].len() as f32 / sample_rate;
        Self {
            samples,
            sample_rate,
            current_channel,
            sample_length_in_seconds,
        }
    }

    pub fn get_channel_count (&self) -> usize {
        self.samples.len()
    }
    
    pub fn get_current_channel (&self) -> usize {
        self.current_channel
    }

    pub fn get_length_in_seconds (&self) -> f32 {
        self.sample_length_in_seconds
    }

    pub fn set_audio_channel (&mut self, channel_number : usize) {
        self.current_channel = channel_number.clamp(0, self.samples.len()-1);
        self.sample_length_in_seconds = self.samples[self.current_channel].len() as f32 / self.sample_rate;
    }
}

impl Default for DecodedAudio {
    fn default() -> Self {
        Self {
            samples : vec![vec![-1.0, 1.0]],
            sample_rate : 440.0,
            current_channel : 0,
            sample_length_in_seconds : 1.0 / 440.0,
        }
    }
}