// sampler/sample_player.rs
use parking_lot::RwLock;
use std::{f32::consts::FRAC_PI_2, sync::Arc};

use crate::sampler::decoded_audio::DecodedAudio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleTimeMarks {
    play_start: f32,
    is_reverse_play_direction: bool,
    play_end: f32,

    loop_enabled: bool,
    loop_start: f32,
    is_reverse_loop_direction: bool,
    loop_end: f32,
}

impl SampleTimeMarks {
    pub fn new(
        mut play_start: f32,
        mut play_end: f32,
        mut loop_start: f32,
        mut loop_end: f32,
        loop_enabled: bool,
    ) -> Self {
        play_end = play_end.clamp(0.0, 1.0);
        play_start = play_start.clamp(0.0, 1.0);
        loop_end = loop_end.clamp(0.0, 1.0);
        loop_start = loop_start.clamp(0.0, 1.0);

        Self {
            play_start,
            play_end,
            is_reverse_play_direction: (play_end < play_start),
            loop_enabled,
            loop_start,
            is_reverse_loop_direction: (loop_end < loop_start),
            loop_end,
        }
    }
    #[inline]
    pub fn get_loop_length(&self) -> f32 {
        (self.loop_end - self.loop_start).abs()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SamplePlayerStatus {
    #[default]
    PlayingNormal,
    PlayingLoop,
    PlayEnd,
}

#[derive(Debug, Clone)]
pub struct SamplePlayer {
    decoded_audio: Arc<RwLock<DecodedAudio>>,
    position: f32,
    value: f32,
    status: SamplePlayerStatus,
    //
    fade_out_angle: f32,
    fade_out_position: f32,
    fade_out_gain: f32,
}

impl SamplePlayer {
    pub fn new(decoded_audio: &Arc<RwLock<DecodedAudio>>, time_marks: SampleTimeMarks) -> Self {
        let position = time_marks.play_start;
        Self {
            decoded_audio: decoded_audio.clone(),
            position,
            value: 0.0,
            status: SamplePlayerStatus::PlayingNormal,
            fade_out_angle: 0.0,
            fade_out_position: 0.0,
            fade_out_gain: 0.0,
        }
    }

    pub fn get_status(&self) -> SamplePlayerStatus {
        self.status
    }

    pub fn reset(&mut self, time_marks: SampleTimeMarks, adsr_value: f32, velocity_gain: f32) {
        self.fade_out_gain = adsr_value * velocity_gain;
        self.fade_out_angle = FRAC_PI_2;
        self.fade_out_position = self.position;
        self.position = time_marks.play_start;
        let decoded_audio_guard = self.decoded_audio.read();
        self.value = table_value(self.position, &decoded_audio_guard);
        self.status = SamplePlayerStatus::PlayingNormal;
    }

    pub fn process(
        &mut self,
        step_in_seconds: f32,
        time_marks: SampleTimeMarks,
        adsr_value: f32,
        velocity_gain: f32,
    ) -> (f32, SamplePlayerStatus) {
        let decoded_audio_guard = self.decoded_audio.read();
        let step = step_in_seconds / decoded_audio_guard.get_length_in_seconds();

        match self.status {
            SamplePlayerStatus::PlayingNormal => {
                let mut status_changed = false;
                if time_marks.loop_enabled {
                    let movement_to_loop_start = match time_marks.is_reverse_play_direction {
                        true => self.position - time_marks.loop_start,
                        false => time_marks.loop_start - self.position,
                    };
                    if movement_to_loop_start <= step {
                        self.position = time_marks.loop_start
                            + if time_marks.is_reverse_loop_direction {
                                -(step - movement_to_loop_start.abs())
                                    .rem_euclid(time_marks.get_loop_length())
                            } else {
                                (step - movement_to_loop_start.abs())
                                    .rem_euclid(time_marks.get_loop_length())
                            };
                        self.status = SamplePlayerStatus::PlayingLoop;
                        status_changed = true;
                    }
                }
                if !status_changed {
                    let movement_to_end = match time_marks.is_reverse_play_direction {
                        true => self.position - time_marks.play_end,
                        false => time_marks.play_end - self.position,
                    };
                    if movement_to_end <= step {
                        self.position = time_marks.play_end;
                        self.status = SamplePlayerStatus::PlayEnd;
                        status_changed = true;
                    }
                }
                if !status_changed {
                    self.position += match time_marks.is_reverse_play_direction {
                        true => -step,
                        false => step,
                    };
                }
            }
            SamplePlayerStatus::PlayingLoop => {
                let position_at_loop = match time_marks.is_reverse_loop_direction {
                    true => time_marks.loop_start - self.position,
                    false => self.position - time_marks.loop_start,
                };

                self.position = time_marks.loop_start
                    + match time_marks.is_reverse_loop_direction {
                        true => -(position_at_loop + step).rem_euclid(time_marks.get_loop_length()),
                        false => (position_at_loop + step).rem_euclid(time_marks.get_loop_length()),
                    };
            }
            SamplePlayerStatus::PlayEnd => {}
        }

        self.position = self.position.clamp(0.0, 1.0);

        // вычисление и возвращение значения

        let (prev, curr) = self.fade_out_angle.sin_cos();

        self.value = curr
            * table_value(self.position, &decoded_audio_guard)
            * adsr_value
            * velocity_gain
            + prev * table_value(self.fade_out_position, &decoded_audio_guard) * self.fade_out_gain;
        if self.fade_out_angle > 0.0 {
            self.fade_out_angle -= 0.024543692; // = (pi/2)/64
        }
        (self.value, self.status)
    }
}

/// Возвращает значение линейно интерполированного сэмпла на x-ой секунде
#[inline]
pub fn table_value(x: f32, decoded_audio: &DecodedAudio) -> f32 {
    if let Some(table) = decoded_audio
        .samples
        .get(decoded_audio.get_current_channel())
    {
        if x.is_nan() || table.is_empty() {
            return 0.0;
        }

        let x = x.clamp(0.0, 1.0);

        let table_len_minus_1 = table.len() - 1;
        let sample_f = x * table_len_minus_1 as f32;
        let sample_i = sample_f.trunc() as usize;
        let fract = sample_f.fract();
        let idx0 = sample_i;
        let idx1 = (sample_i + 1).min(table_len_minus_1);

        table[idx0] * (1.0 - fract) + table[idx1] * fract
    } else {
        0.0
    }
}

#[cfg(test)]
mod test {
    use crate::sampler::sample_player::*;

    #[test]
    fn table_value_func_limits_values() {
        let table = DecodedAudio::new(vec![vec![0.0, 1.0]], 2.0);
        for val in [-1.0, 0.0, -f32::INFINITY] {
            assert_eq!(table_value(val, &table), table_value(0.0, &table));
        }
        for val in [1.0, 2.0, f32::INFINITY] {
            assert_eq!(table_value(val, &table), table_value(1.0, &table));
        }

        assert_eq!(table_value(f32::NAN, &table), 0.0)
    }

    #[test]
    fn table_value_func() {
        let table = DecodedAudio::new(vec![vec![0.0, 1.0]], 2.0);
        let test_iter = [0.25, 0.5, 0.75].into_iter();
        test_iter.for_each(|s| {
            let abs_diff = (table_value(s, &table) - s).abs();
            dbg!((s, abs_diff));
            assert!(abs_diff <= f32::EPSILON)
        });
    }

    #[test]
    fn sample_player() {
        let table = DecodedAudio::new(vec![vec![0.0, 1.0]], 2.0);
        let time_marks = SampleTimeMarks::new(1.0, 0.0, 0.5, 1.0, true);
        let mut sp = SamplePlayer::new(&Arc::new(RwLock::new(table)), time_marks);

        for _ in 0..200 {
            let res = sp.process(0.01, time_marks, 1.0, 1.0);
            println!("({:?}\t:{})", res.1, res.0,);
        }

        //this test oriented to debug of the SamplePlayer
        assert!(true)
    }
}
