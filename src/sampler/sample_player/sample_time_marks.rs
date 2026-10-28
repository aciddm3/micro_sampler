#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleTimeMarks {
    play_start: f32,
    is_reverse_play_direction: bool,
    play_end: f32,
    //
    pub loop_enabled: bool,
    //
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
            is_reverse_play_direction: play_end < play_start,
            loop_enabled,
            loop_start,
            is_reverse_loop_direction: loop_end < loop_start,
            loop_end,
        }
    }
    #[inline]
    pub fn get_loop_length(&self) -> f32 {
        (self.loop_end - self.loop_start).abs()
    }

    #[inline]
    pub fn get_play_start(&self) -> f32 {
        self.play_start
    }

    #[inline]
    pub fn get_play_end(&self) -> f32 {
        self.play_end
    }

    #[inline]
    pub fn get_loop_start(&self) -> f32 {
        self.loop_start
    }

	#[inline]
    pub fn get_loop_end(&self) -> f32 {
        self.loop_end
    }

    #[inline]
    pub fn has_reverse_loop_direction(&self) -> bool {
        self.is_reverse_loop_direction
    }

    #[inline]
    pub fn has_reverse_play_direction(&self) -> bool {
        self.is_reverse_play_direction
    }
}
