use crate::sampler::decoded_audio::DecodedAudio;
// sampler/voices/mod.rs
use crate::sampler::voices::note::TransposeInfo;

use super::sample_player::SampleTimeMarks;
use super::voices::note::Note;
use super::{
    adsr::AdsrParams,
    params::MAX_POLYPHONY,
    voices::voice::{Voice, VoiceStatus},
};

use parking_lot::RwLock;
use std::sync::Arc;

pub mod note;
mod voice;

///Round Robin scheme polyphony
#[derive(Debug)]
pub struct VoiceManager {
    pub transpose_info: TransposeInfo,
    pub adsr_params: Arc<RwLock<AdsrParams>>,
    pub time_marks: SampleTimeMarks,
    current_polyphony: usize,
    pub retrigger: bool,

    last_voice_index: usize,
    voices: Vec<Voice>,
    pub velocity_skew: f32,
    //
    process_step: f32,
    //
    value: f32,
}

impl VoiceManager {
    pub fn new(
        adsr_params: Arc<RwLock<AdsrParams>>,
        decoded_audio: Arc<RwLock<DecodedAudio>>,
        time_marks: SampleTimeMarks,
        current_polyphony: usize,
        retrigger: bool,
        plugin_samplerate: f32,
        velocity_skew: f32,
        transpose_info: TransposeInfo,
    ) -> Self {
        let voices =
            vec![Voice::new(&adsr_params, &decoded_audio, time_marks); MAX_POLYPHONY as usize];
        Self {
            voices,
            last_voice_index: 0,
            current_polyphony,
            retrigger,
            adsr_params,
            value: 0.0,
            velocity_skew,
            process_step: 1.0 / plugin_samplerate,
            transpose_info,
            time_marks,
        }
    }

    /// Вставляет ноту в список воспроизведения
    pub fn insert_note(&mut self, note: Note, time_marks: SampleTimeMarks) {
        if let Some(voice) = self
            .voices
            .iter_mut()
            .take(self.current_polyphony)
            .find(|s| s.status != VoiceStatus::Inactive && s.get_note().has_same_number(note))
        {
        match voice.status {
            VoiceStatus::Playing => {
                if self.retrigger {
                    voice.reset(time_marks);
                    
                    voice.reset_adsr();
                    voice.gate_on();
                    
                    // Обновляем velocity, если он изменился
                    voice.set_note(note, self.velocity_skew);
                }
            }
            VoiceStatus::Releasing | VoiceStatus::Inactive => {
                voice.reset(time_marks);
                voice.set_note(note, self.velocity_skew);
                voice.gate_on();
            }
        }
    } else {
        self.last_voice_index = (self.last_voice_index + 1) % self.current_polyphony;
        let stolen = &mut self.voices[self.last_voice_index];
        stolen.reset(time_marks);
        stolen.set_note(note, self.velocity_skew);
        stolen.gate_on();
    }
    }

    /// Убирает ноту из списка воспроизведения
    pub fn remove_note(&mut self, note: Note) {
        for voice in self
            .voices
            .iter_mut()
            .take(self.current_polyphony)
            .filter(|s| s.get_note().has_same_number(note))
        {
            voice.gate_off();
        }
    }

    pub fn process(&mut self, time_marks: SampleTimeMarks) -> f32 {
        self.value = 0.0;
        self.time_marks = time_marks;
        for val in self
            .voices
            .iter_mut()
            .take(self.current_polyphony)
            .filter(|p| p.status != VoiceStatus::Inactive)
        {
            self.value += val.process(
                self.process_step,
                self.process_step * val.self_note_to_step(self.transpose_info),
                time_marks,
            );
        }
        self.value
    }

    pub fn set_polyphony(&mut self, polyphony: i32, time_marks: SampleTimeMarks) {
        self.time_marks = time_marks;
        let polyphony = polyphony.clamp(1, MAX_POLYPHONY) as usize;
        if self.current_polyphony != polyphony {
            if self.current_polyphony > polyphony {
                for i in polyphony..self.current_polyphony {
                    self.voices[i].reset(time_marks);
                }
            }
            if self.last_voice_index >= polyphony {
                self.last_voice_index = 0;
            }
            self.current_polyphony = polyphony;
        }
    }

    pub fn init_from_default(
        &mut self,
        decoded_audio: Arc<RwLock<DecodedAudio>>,
        plugin_samplerate: f32,
        time_marks: SampleTimeMarks,
    ) {
        self.time_marks = time_marks;
        self.voices =
            vec![Voice::new(&self.adsr_params, &decoded_audio, time_marks); MAX_POLYPHONY as usize];
        self.process_step = 1.0 / plugin_samplerate;
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn common_test() {
        let time_marks = SampleTimeMarks::new(0.0, 1.0, 0.0, 1.0, true);
        let mut vm = VoiceManager::new(
            Arc::new(RwLock::new(AdsrParams {
                attack: 0.0,
                decay: 0.0,
                sustain: 1.0,
                release: 0.0,
            })),
            Arc::new(RwLock::new(DecodedAudio::new(vec![vec![1.0, -1.0]], 2.0))),
            time_marks,
            4,
            true,
            100.0,
            0.0,
            TransposeInfo {
                octaves: 0,
                semitones: 0,
                fine_tuning: 0.0,
            },
        );
        vm.retrigger = true;
        vm.insert_note(
            Note {
                number: 69,
                velocity: 1.0,
            },
            time_marks,
        );

        for _ in 0..100 {
            print!("{:3.4} \t\t", vm.process(time_marks))
        }
        println!()
    }
}
