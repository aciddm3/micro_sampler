// sampler/adsr.rs

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdsrPhase {
    /// Attack and remain time is f32
    Attack(f32),
    /// Decay and remain time is f32
    Decay(f32),
    /// Sustain
    Sustain,
    /// Release contains remain time and release start value
    Release(f32, f32),
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

    #[inline]
    fn morph_exp(x: f32) -> f32 {
        (20f32.powf(x) - 1.0) / 19.0
    }

    pub fn step(&mut self, step: f32) {
        let guard = self.params.read();
        if self.is_gated {
            match self.phase {
                AdsrPhase::Idle => {
                    self.current_val = 0.0;
                    self.phase = if guard.attack > step {
                        AdsrPhase::Attack(guard.attack - step)
                    } else if guard.decay > step {
                        self.current_val = 1.0;
                        AdsrPhase::Decay(guard.decay - step)
                    } else {
                        self.current_val = guard.sustain;
                        AdsrPhase::Sustain
                    };
                }
                AdsrPhase::Attack(rem) => {
                    if rem <= step {
                        self.phase = if guard.decay > step {
                            AdsrPhase::Decay(guard.decay + rem - step)
                        } else {
                            self.current_val = guard.sustain;
                            AdsrPhase::Sustain
                        }
                    } else {
                        self.phase = AdsrPhase::Attack(rem - step);
                    }
                    self.current_val = Self::morph_exp(1.0 - rem / guard.attack.max(0.0001));
                }
                AdsrPhase::Decay(rem) => {
                    if rem <= step {
                        self.phase = AdsrPhase::Sustain
                    } else {
                        self.phase = AdsrPhase::Decay(rem - step);
                    }
                    self.current_val = guard.sustain + (1.0 - guard.sustain) * Self::morph_exp(rem / guard.decay.max(0.0001));
                }
                AdsrPhase::Sustain => self.current_val = guard.sustain,
                _ => self.phase = AdsrPhase::Idle,
            }
        } else {
            // self.is_gated() == false
            match self.phase {
                AdsrPhase::Attack(_) | AdsrPhase::Decay(_) | AdsrPhase::Sustain => {
                    if guard.release <= step {
                        self.current_val = 0.0;
                        self.phase = AdsrPhase::Idle;
                    } else {
                        self.phase = AdsrPhase::Release(guard.release - step, self.current_val)
                    }
                }
                AdsrPhase::Release(rem, val) => {
                    if rem <= step {
                        self.phase = AdsrPhase::Idle
                    } else {
                        self.phase = AdsrPhase::Release(rem - step, val);
                    }
                    self.current_val = val * Self::morph_exp(rem / guard.release.max(0.0001));
                }
                _ => {
                    self.phase = AdsrPhase::Idle;
                    self.current_val = 0.0;
                }
            }
        };
        self.current_val = self.current_val.clamp(0.0, 1.0);
    }

    pub fn get_value(&self) -> f32 {
        self.current_val
    }

    pub fn get_phase(&self) -> AdsrPhase {
        self.phase
    }
}
