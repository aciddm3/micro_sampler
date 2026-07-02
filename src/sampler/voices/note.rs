// sampler/voices/note.rs

#[derive(Debug, Clone, Copy, Default)]
pub struct Note {
    pub number: u8,
    pub velocity: f32,
}

impl Note {
    pub fn has_same_number(self, other: Self) -> bool {
        self.number == other.number
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransposeInfo {
    pub octaves: i32,
    pub semitones: i32,
    pub fine_tuning: f32,
}

impl TransposeInfo {
    pub fn set(&mut self, octaves: i32, semitones: i32, fine_tuning: f32) {
        self.fine_tuning = fine_tuning;
        self.octaves = octaves;
        self.semitones = semitones;
    }
}
