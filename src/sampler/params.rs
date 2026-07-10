// sampler/params.rs
use nih_plug::prelude::*;
use parking_lot::RwLock;

pub const MAX_POLYPHONY: i32 = 16;

#[derive(Params, Debug)]
pub struct SamplerParams {
    #[id = "polyphony"]
    pub polyphony: IntParam,
    // число полифонии (см. полифонические синты в Caustic 3, там можно менять полифонию самому.
    // Также работает SurgeXT)
    #[id = "retrigger"]
    pub retrigger: BoolParam,
    // Если ретриггер выключен, то слитные ноты будут воспроизводится как одна
    // в противном случае, вызов каждой будет "перезапускать" голос
    // см. ADSR огибающую в CardinalSynth.
    #[id = "play_start"]
    pub play_start: FloatParam,
    // Нормализованное время начала проигрывания
    #[id = "play_end"]
    pub play_end: FloatParam,
    // Нормализованное время окончания проигрывания
    //
    #[id = "loop_enable"]
    pub loop_enable: BoolParam,
    // включен ли цикл
    #[id = "loop_start"]
    pub loop_start: FloatParam,
    // Нормализованное время начала цикла
    #[id = "loop_end"]
    pub loop_end: FloatParam,
    // Нормализованное время окончания цикла
    //
    #[id = "transpose_oct"]
    pub transpose_oct: IntParam,
    // транспозиция сэмпла в октавах
    #[id = "transpose_semitones"]
    pub transpose_semitones: IntParam,
    // транспозиция сэмпла в полутонах
    #[id = "fine_tune"]
    pub fine_tune: FloatParam,
    // Изменение плейрейта сэмпла в пределах одного полутона
    //
    #[id = "attack"]
    pub attack: FloatParam,
    // attack ADSR огибающей амплитуды
    #[id = "decay"]
    pub decay: FloatParam,
    // decay ADSR огибающей амплитуды
    #[id = "sustain"]
    pub sustain: FloatParam,
    // sustain ADSR огибающей амплитуды
    #[id = "release"]
    pub release: FloatParam,
    // release ADSR огибающей амплитуды
    //
    #[id = "velocity_skew"]
    pub velocity_skew: FloatParam,
    // влияние velocity на громкость воспроизведения
    //
    #[id = "panorama"]
    pub panorama: FloatParam,
    // панорама звука на выходе
    #[id = "master_gain"]
    pub master_gain: FloatParam,
    // усиление звука на выходе
    //
    #[persist = "file_path"]
    pub file_path: RwLock<Option<String>>,
    // путь к фалу с сэмплами
    #[persist = "file_audio_channel"]
    pub file_audio_channel_number: RwLock<usize>,
    // выбор канала аудио файла для воспроизведения
}

impl Default for SamplerParams {
    fn default() -> Self {
        Self {
            polyphony: IntParam::new(
                "Polyphony",
                1,
                IntRange::Linear {
                    min: 1,
                    max: MAX_POLYPHONY,
                },
            ),

            retrigger: BoolParam::new("Retrigger", true),
            transpose_oct: IntParam::new("Octave", 0, IntRange::Linear { min: -5, max: 5 }),
            transpose_semitones: IntParam::new("Semiton", 0, IntRange::Linear { min: -6, max: 6 }),
            fine_tune: FloatParam::new(
                "Fine Tune",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            ).with_unit("cents"),

            play_start: FloatParam::new("Start", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            play_end: FloatParam::new("End", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),

            loop_enable: BoolParam::new("Loop Enable", true),
            loop_start: FloatParam::new(
                "Loop Start",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),
            loop_end: FloatParam::new("Loop End", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),

            attack: FloatParam::new(
                "Attack",
                0.25,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 8.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            ).with_unit("s").with_value_to_string(formatters::v2s_f32_rounded(3)),
            decay: FloatParam::new(
                "Decay",
                0.25,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 8.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            ).with_unit("s").with_value_to_string(formatters::v2s_f32_rounded(3)),
            sustain: FloatParam::new(
                "Sustain",
                1.0,
                FloatRange::Linear {
                    min: -90.0,
                    max: 0.0,
                },
            )
            .with_unit("dB")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            release: FloatParam::new(
                "Release",
                0.25,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 8.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit("s").with_value_to_string(formatters::v2s_f32_rounded(3)),
            panorama: FloatParam::new("Panorama", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(3)),
            master_gain: FloatParam::new(
                "Master Gain",
                0.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 20.0,
                },
            )
            .with_unit("dB")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            velocity_skew: FloatParam::new(
                "Velocity Skew",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            //
            file_path: RwLock::new(None),
            file_audio_channel_number: RwLock::new(0),
        }
    }
}
