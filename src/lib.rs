// lib.rs
use nih_plug::{prelude::*, util::db_to_gain};

use crate::{
    background_task::{BackgroundTask::LoadSampleWithoutDialog, SamplerTaskExecutor},
    gui::SamplerGUI,
    sampler::{sample_player::SampleTimeMarks, voices::note::Note},
};

mod background_task;
mod gui;
mod sampler;

impl Plugin for sampler::Sampler {
    const NAME: &'static str = "MicroSampler";

    const VENDOR: &'static str = "Gema";

    const URL: &'static str = "https://gema/sampler";

    const EMAIL: &'static str = "None";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION_MAJOR");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_output_channels: NonZeroU32::new(2),
        main_input_channels: NonZeroU32::new(0),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = ();

    type BackgroundTask = background_task::BackgroundTask;

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        context.execute(LoadSampleWithoutDialog);
        self.plugin_samplerate = buffer_config.sample_rate;
        self.vm.init_from_default(
            self.decoded_audio.clone(),
            self.plugin_samplerate,
            SampleTimeMarks::new(0.0, 1.0, 0.0, 1.0, false),
        );
        true
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        self.create_gui(async_executor)
    }

    fn params(&self) -> std::sync::Arc<dyn nih_plug::prelude::Params> {
        self.params.clone()
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        SamplerTaskExecutor::task_executor(self)
    }

    fn process(
        &mut self,
        buffer: &mut nih_plug::prelude::Buffer,
        _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
        context: &mut impl nih_plug::prelude::ProcessContext<Self>,
    ) -> nih_plug::prelude::ProcessStatus {
        self.vm.set_polyphony(
            self.params.polyphony.value(),
            SampleTimeMarks::new(
                self.params.play_start.value(),
                self.params.play_end.value(),
                self.params.loop_start.value(),
                self.params.loop_end.value(),
                self.params.loop_enable.value(),
            ),
        );
        self.vm.retrigger = self.params.retrigger.value();
        self.vm.transpose_info.set(
            self.params.transpose_oct.value(),
            self.params.transpose_semitones.value(),
            self.params.fine_tune.value(),
        );
        self.vm.velocity_skew = self.params.velocity_skew.value();
        //
        let mut next_event = context.next_event();
        let sample_count = buffer.samples();
        //
        self.vm.adsr_params.write().set(
            self.params.attack.value(),
            self.params.decay.value(),
            db_to_gain(self.params.sustain.value()),
            self.params.release.value(),
        );
        //
        let mut time_marks;
        for sample_idx in 0..sample_count {
            time_marks = SampleTimeMarks::new(
                self.params.play_start.smoothed.next(),
                self.params.play_end.smoothed.next(),
                self.params.loop_start.smoothed.next(),
                self.params.loop_end.smoothed.next(),
                self.params.loop_enable.value(),
            );

            while let Some(event) = next_event {
                if event.timing() > sample_idx as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        if velocity > 0.0 {
                            self.vm.insert_note(
                                Note {
                                    number: note,
                                    velocity,
                                },
                                time_marks,
                            );
                        } else {
                            self.vm.remove_note(Note {
                                number: note,
                                ..Default::default()
                            });
                        }
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        self.vm.remove_note(Note {
                            number: note,
                            ..Default::default()
                        });
                    }
                    _ => (),
                }

                next_event = context.next_event();
            }

            let pan_angle = std::f32::consts::FRAC_PI_2 * self.params.panorama.smoothed.next();
            let (r_gain, l_gain) = pan_angle.sin_cos();

            let process = self.vm.process(time_marks);
            let output = process * 10f32.powf(self.params.master_gain.smoothed.next() / 20.0);
            for channel in buffer.as_slice().chunks_mut(2) {
                channel[0][sample_idx] = output * l_gain;
                channel[1][sample_idx] = output * r_gain;
            }
        }
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for sampler::Sampler {
    const VST3_CLASS_ID: [u8; 16] = [
        151, 65, 37, 235, 58, 129, 242, 200, 41, 28, 245, 82, 161, 50, 250, 90,
    ]; // !generated randomly!

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Sampler];
}

nih_export_vst3!(sampler::Sampler);
