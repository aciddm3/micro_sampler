// sampler/mod.rs
use std::sync::Arc;

use parking_lot::RwLock;

use crate::sampler::adsr::AdsrParams;
use crate::sampler::decoded_audio::DecodedAudio;
use crate::sampler::params::{MAX_POLYPHONY, SamplerParams};
use crate::sampler::sample_player::SampleTimeMarks;
use crate::sampler::voices::VoiceManager;
use crate::sampler::voices::note::TransposeInfo;

pub mod adsr;
pub mod decoded_audio;
pub mod params;
pub mod sample_player;
pub mod voices;

#[derive(Debug)]
pub struct Sampler {
    pub params: Arc<SamplerParams>,
    //
    pub vm: VoiceManager,
    //
    pub decoded_audio: Arc<RwLock<DecodedAudio>>,
    //
    pub plugin_samplerate: f32,
}

impl Default for Sampler {
    fn default() -> Self {
        let time_marks = SampleTimeMarks::new(0.0, 1.0, 0.0, 0.5, true);
        let decoded_audio = Arc::new(RwLock::new(DecodedAudio::default()));
        let adsr_params = AdsrParams {
            attack: 0.1,
            decay: 0.0,
            sustain: 1.0,
            release: 0.1,
        };
        let adsr_params = Arc::new(RwLock::new(adsr_params));

        let plugin_samplerate = 48000.0;

        Self {
            params: Arc::new(SamplerParams::default()),
            vm: VoiceManager::new(
                adsr_params,
                decoded_audio.clone(),
                time_marks,
                MAX_POLYPHONY as usize,
                true,
                plugin_samplerate,
                1.0,
                TransposeInfo {
                    octaves: 0,
                    semitones: 0,
                    fine_tuning: 0.0,
                },
            ),
            decoded_audio: decoded_audio.clone(),
            plugin_samplerate,
        }
    }
}
