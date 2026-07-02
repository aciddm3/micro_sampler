// sampler/sample_player.rs
use parking_lot::RwLock;
use std::{f32::consts::FRAC_PI_2, sync::Arc};

use crate::sampler::decoded_audio::DecodedAudio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleTimeMarks {
    play_start: f32,
    play_direction_sight: f32,
    play_end: f32,

    loop_enabled: bool,
    loop_start: f32,
    loop_direction_sight: f32,
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
            play_direction_sight: (play_end - play_start).signum(),
            loop_enabled,
            loop_start,
            loop_direction_sight: (loop_end - loop_start).signum(),
            loop_end,
        }
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
        }
    }

    pub fn get_status(&self) -> SamplePlayerStatus {
        self.status
    }

    pub fn reset(&mut self, time_marks: SampleTimeMarks) {
        self.fade_out_angle = FRAC_PI_2;
        self.fade_out_position = self.position;
        self.position = time_marks.play_start;
        let decoded_audio_guard = self.decoded_audio.read();
        self.value = table_value(self.position, &decoded_audio_guard);
        self.status = SamplePlayerStatus::PlayingNormal;
    }

    pub fn process(&mut self, step: f32, time_marks: SampleTimeMarks) -> (f32, SamplePlayerStatus) {
        let decoded_audio_guard = self.decoded_audio.read();
        let step_normalized = step / decoded_audio_guard.get_length_in_seconds();
        if self.status == SamplePlayerStatus::PlayingNormal {
            // если проигрывается вне цикла
            self.position += time_marks.play_direction_sight * step_normalized;
            self.fade_out_position += time_marks.play_direction_sight * step_normalized;
            self.position = self.position.clamp(0.0, 1.0);
            if self.position * time_marks.play_direction_sight // тут используется свойство x > y <=> -x < -y
                >= time_marks.play_direction_sight * time_marks.loop_start
                && time_marks.loop_enabled
            {
                self.status = SamplePlayerStatus::PlayingLoop;
            }

            if self.position * time_marks.play_direction_sight
                >= time_marks.play_direction_sight * time_marks.play_end
            {
                self.status = SamplePlayerStatus::PlayEnd;
            }
        } else if self.status == SamplePlayerStatus::PlayingLoop {
            // если проигрывается цикл
            self.position += time_marks.loop_direction_sight * step_normalized;
            self.position = self.position.clamp(0.0, 1.0);

            if self.position * time_marks.loop_direction_sight
                >= time_marks.loop_direction_sight * time_marks.loop_end
            {
                self.position = time_marks.loop_start
            }
        }
        // вычисление и возвращение значения

        let (prev, curr) = self.fade_out_angle.sin_cos();
        
        self.value = curr * table_value(self.position, &decoded_audio_guard) + prev * table_value(self.fade_out_position, &decoded_audio_guard);
        if self.fade_out_angle > 0.0 {
            self.fade_out_angle -= 0.0030674; // = (pi/2)/256
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
            let res = sp.process(0.01, time_marks);
            println!("({:?}\t:{})", res.1, res.0,);
        }

        //this test oriented to debug of the SamplePlayer
        assert!(true)
    }
}
