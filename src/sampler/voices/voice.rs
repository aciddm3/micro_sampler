// sampler/voices/voice.rs
//
use crate::sampler::decoded_audio::DecodedAudio;
use crate::sampler::sample_player::SampleTimeMarks;
use crate::sampler::voices::note::TransposeInfo;
use crate::sampler::{
    adsr::{Adsr, AdsrParams, AdsrPhase},
    sample_player::{SamplePlayer, SamplePlayerStatus},
    voices::note::Note,
};

use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoiceStatus {
    /// Этот статус должен быть, если голос не играет
    Inactive,
    /// Этот статус должен быть, если ADSR голоса в фазе Attack, Decay либо Sustain
    Playing,
    /// Этот статус должен быть, если ADSR голоса в фазе Release
    Releasing,
}

#[derive(Debug, Clone)]
pub struct Voice {
    pub status: VoiceStatus,
    note: Note,
    gain_by_velocity: f32,
    adsr: Adsr,
    sp: SamplePlayer,
    value: f32,
}

impl Voice {
    pub fn new(
        adsr_params: &Arc<RwLock<AdsrParams>>,
        decoded_audio: &Arc<RwLock<DecodedAudio>>,
        time_marks: SampleTimeMarks,
    ) -> Self {
        Self {
            status: VoiceStatus::Inactive,
            note: Note {
                number: 64,
                velocity: 0.0,
            },
            gain_by_velocity: 0.0,
            adsr: Adsr::new(adsr_params.clone()),
            sp: SamplePlayer::new(decoded_audio, time_marks),
            value: 0.0,
        }
    }

    pub fn reset_adsr(&mut self) {
        self.adsr.reset();
    }

    pub fn process(&mut self, adsr_step: f32, sp_step: f32, time_marks: SampleTimeMarks) -> f32 {
        self.adsr.step(adsr_step);
        self.value = self.adsr.get_value() * self.sp.process(sp_step, time_marks).0 * self.gain_by_velocity;
        match self.adsr.get_phase() {
            AdsrPhase::Attack | AdsrPhase::Decay | AdsrPhase::Sustain => {
                self.status = VoiceStatus::Playing
            }
            AdsrPhase::Release => self.status = VoiceStatus::Releasing,
            AdsrPhase::Idle => {
                self.status = VoiceStatus::Inactive;
                self.adsr.gate_off();
                self.value = 0.0;
            }
        }
        if matches!(self.sp.get_status(), SamplePlayerStatus::PlayEnd) {
            self.reset(time_marks);
            self.value = 0.0;
        }

        self.value
    }

    pub fn get_note(&self) -> Note {
        self.note
    }

    pub fn set_note(&mut self, note: Note, velocity_skew: f32) {
        self.note = note;
        self.gain_by_velocity = velocity_to_gain(note.velocity, velocity_skew);
    }

    #[inline]
    pub fn reset(&mut self, time_marks: SampleTimeMarks) {
        self.adsr.reset();
        self.sp.reset(time_marks);
        self.status = VoiceStatus::Inactive;
    }

    pub fn gate_on(&mut self) {
        self.adsr.gate_on();
        self.status = VoiceStatus::Playing;
    }

    pub fn gate_off(&mut self) {
        self.adsr.gate_off();
    }

    #[inline]
    pub fn self_note_to_step(&self, transpose: TransposeInfo) -> f32 {
        note_to_step(self.note.number, transpose)
    }
}

/// переводит MIDI ноту в множитель плейрэйта для сэмплера
#[inline]
pub fn note_to_step(
    note_midi_number: u8,
    TransposeInfo {
        octaves,
        semitones,
        fine_tuning,
    }: TransposeInfo,
) -> f32 {
    2f32.powf(
        (note_midi_number as f32 - 69.0 + 12.0 * octaves as f32 + semitones as f32 + fine_tuning)
            / 12.0,
    )
}

/// переводит velocity в коэффициент усиления
/// этот коэфициент зависит также от параметра skew
/// параметр skew клампится:
///
/// ``skew = skew.clamp(-1.0, 1.0);``
///
/// Параметр skew определяет функцию зависимости громкости от velocity
#[inline]
pub fn velocity_to_gain(velocity: f32, mut skew: f32) -> f32 {
    skew = skew.clamp(-1.0, 1.0);
    if skew <= -0.999 {
        0.0
    } else if skew >= 0.999 {
        1.0
    } else {
        velocity.powf((1.0 - skew) / (1.0 + skew))
    }
}
