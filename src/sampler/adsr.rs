// sampler/adsr.rs

use nih_plug::util::db_to_gain;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct AdsrParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(Debug, Clone)]
pub struct Adsr {
    pub params: Arc<RwLock<AdsrParams>>,
    pub current_val: f32,
    pub is_gated: bool,
    pub phase: AdsrPhase,
}

impl AdsrParams {
    pub fn set(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack;
        self.decay = decay;
        self.sustain = sustain;
        self.release = release;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdsrPhase {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

impl Adsr {
    pub fn new(params: Arc<RwLock<AdsrParams>>) -> Self {
        Self {
            params,
            current_val: 0.0,
            is_gated: false,
            phase: AdsrPhase::Idle,
        }
    }

    pub fn gate_on(&mut self) {
        self.is_gated = true;
    }

    pub fn gate_off(&mut self) {
        self.is_gated = false;
    }

    pub fn reset(&mut self) {
        self.current_val = 0.0;
        self.phase = AdsrPhase::Idle;
    }

    pub fn step(&mut self, step: f32) {
        let guard = self.params.read();
        if self.is_gated {
            match self.phase {
                AdsrPhase::Idle => {
                    self.phase = AdsrPhase::Attack;
                }
                AdsrPhase::Attack => {
                    if guard.attack < f32::EPSILON {
                        self.current_val = 1.0;
                    } else {
                        self.current_val += step / guard.attack;
                    }

                    if self.current_val >= 1.0 {
                        self.phase = AdsrPhase::Decay;
                    }
                }
                AdsrPhase::Decay => {
                    let der = guard.decay * (1.0 - guard.sustain);
                    if der < f32::EPSILON {
                        self.current_val = guard.sustain;
                    } else {
                        self.current_val -= step / der;
                    }

                    if self.current_val <= guard.sustain {
                        self.phase = AdsrPhase::Sustain;
                        self.current_val = guard.sustain;
                    }
                }
                AdsrPhase::Sustain => self.current_val = guard.sustain,
                _ => self.phase = AdsrPhase::Idle,
            }
        } else {
            // self.is_gated() == false
            if self.current_val > 0.0 {
                self.phase = AdsrPhase::Release;
                let der = guard.release * guard.sustain;
                if der < f32::EPSILON {
                    self.current_val = 0.0;
                } else {
                    self.current_val -= step / guard.release;
                }
            } else {
                self.phase = AdsrPhase::Idle;
                self.current_val = 0.0;
            }
        };
        self.current_val = self.current_val.clamp(0.0, 1.0);
    }

    pub fn get_value(&self) -> f32 {
        db_to_gain(100.0 * (self.current_val - 1.0))
    }

    pub fn get_phase(&self) -> AdsrPhase {
        self.phase
    }
}
