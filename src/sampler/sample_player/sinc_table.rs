// sampler/sample_player/sinc_table.rs

pub const PHASE_COUNT: usize = 1024;
pub const HANN_CORE_SIZE: usize = 16;

const _: () = {
    if PHASE_COUNT < 2 {
        panic!("Compilation error: PHASE_COUNT must be not less than 2");
    }
    if HANN_CORE_SIZE % 2 != 0 || HANN_CORE_SIZE == 0 {
        panic!("Compilation error: HANN_CORE_SIZE must be a positive even number")
    }
};

pub const PHASE_COUNT_F: f32 = PHASE_COUNT as f32;
pub const PHASE_COUNT_MINUS_ONE: usize = PHASE_COUNT - 1;
pub const HALF_HANN_CORE_SIZE_ISIZE: isize = (HANN_CORE_SIZE / 2) as isize;

use std::f32::consts::{PI, TAU};

#[derive(Debug, Clone)]
pub struct SincTable {
    pub phases: Vec<Vec<f32>>,
}

impl SincTable {
    pub fn new() -> Self {
        let mut phases = vec![vec![0.0; HANN_CORE_SIZE]; PHASE_COUNT];

        for p in 0..PHASE_COUNT {
            let phase_offset = p as f32 / PHASE_COUNT as f32;

            for t in 0..HANN_CORE_SIZE {
                let x = (t as f32 - (HANN_CORE_SIZE as f32 / 2.0) + 1.0) - phase_offset;

                let sinc = if x.abs() < 1e-5 {
                    1.0 // sin(pi*x)/pi*x -> 1 as x -> 0 --- the first wonderful limit
                } else {
                    (x * PI).sin() / (x * PI)
                };

                let window = 0.5 - (TAU * t as f32 / (HANN_CORE_SIZE as f32 - 1.0)).cos() / 2.0;

                phases[p][t] = sinc * window;
            }

            let sum: f32 = phases[p].iter().sum();

            if sum.abs() > 1e-4 {
                let inv_sum = 1.0 / sum;
                for val in &mut phases[p] {
                    *val *= inv_sum;
                }
            }
        }

        Self { phases }
    }
}
