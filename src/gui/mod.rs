use nih_plug::{context::gui::AsyncExecutor, editor::Editor, params::Param};
use nih_plug_egui::{
    EguiState, create_egui_editor,
    egui::{self, Color32, Vec2, vec2},
};

use crate::{
    background_task::BackgroundTask,
    gui::{bool_param_switch::BoolParamSwitcher, param_slider::ParamSlider},
    sampler::{Sampler, params::MAX_POLYPHONY},
};

mod bool_param_switch;
mod diagrams;
mod float_param_knob;
mod int_param_knob;
mod param_slider;

use float_param_knob::ParamKnobFloat;
use int_param_knob::ParamKnobInt;

const KNOB_SIZE: Vec2 = vec2(29.0, 29.0);

pub trait SamplerGUI {
    fn create_gui(&mut self, async_executor: AsyncExecutor<Sampler>) -> Option<Box<dyn Editor>>;
}

impl SamplerGUI for Sampler {
    fn create_gui(&mut self, async_executor: AsyncExecutor<Sampler>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let decoded_audio = self.decoded_audio.clone();
        create_egui_editor(
            EguiState::from_size(600, 420),
            (),
            |_ctx, _data| {},
            move |egui_ctx, setter, _data| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.group(|ui| {
                            diagrams::draw_sample_graph(ui, &decoded_audio, &params);
                            ui.horizontal_top(|ui| {
                                let channel_last_index;
                                let current_channel;
                                {
                                    let decoded_audio_guard = decoded_audio.read();
                                    channel_last_index =
                                        decoded_audio_guard.get_channel_count() - 1;
                                    current_channel = decoded_audio_guard.get_current_channel();
                                }
                                ui.label(format!(
                                    "Channel : {}/{}",
                                    current_channel, channel_last_index
                                ));
                                if ui.button("-").clicked() {
                                    decoded_audio.write().set_audio_channel(
                                        (current_channel - 1).clamp(0, channel_last_index),
                                    );
                                }
                                if ui.button("+").clicked() {
                                    decoded_audio.write().set_audio_channel(
                                        (current_channel + 1).clamp(0, channel_last_index),
                                    );
                                }
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        vec2(20.0, 10.0),
                                        BoolParamSwitcher {
                                            param: &params.retrigger,
                                            setter,
                                        },
                                    );
                                    ui.label("retrig");
                                });
                                ui.horizontal(|ui| {
                                    ui.add_sized(
                                        vec2(20.0, 10.0),
                                        BoolParamSwitcher {
                                            param: &params.loop_enable,
                                            setter,
                                        },
                                    );
                                    ui.label("loop");
                                });
                            });
                            ui.horizontal_top(|ui| {
                                if ui.button("Load File").clicked() {
                                    async_executor
                                        .execute_background(BackgroundTask::LoadSampleWithDialog);
                                }
                                if let Some(p) = &*params.file_path.read() {
                                    ui.label(format!("Loaded: {}", p));
                                }
                            });
                            ui.add(
                                ParamSlider::for_param(&params.play_start, setter, Color32::WHITE)
                                    .with_width(ui.available_width()),
                            );
                            ui.add(
                                ParamSlider::for_param(&params.play_end, setter, Color32::WHITE)
                                    .with_width(ui.available_width()),
                            );
                            ui.add(
                                ParamSlider::for_param(&params.loop_start, setter, Color32::BLUE)
                                    .with_width(ui.available_width()),
                            );
                            ui.add(
                                ParamSlider::for_param(&params.loop_end, setter, Color32::BLUE)
                                    .with_width(ui.available_width()),
                            );
                        });
                    });
                    //
                    ui.horizontal(|ui| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Master Gain");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobFloat {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.master_gain,
                                    },
                                );
                                ui.label(format!(
                                    "{:.3}{}",
                                    params.master_gain.value(),
                                    params.master_gain.unit()
                                ))
                            });
                            ui.vertical(|ui| {
                                ui.label("Panorama");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobFloat {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.panorama,
                                    },
                                );
                                ui.label(format!("{:.3}", params.panorama.value()))
                            });
                            ui.vertical(|ui| {
                                ui.label("Vel Skew");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobFloat {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.velocity_skew,
                                    },
                                );
                                ui.label(format!("{:.3}", params.velocity_skew.value()))
                            });
                            ui.vertical(|ui| {
                                ui.label("Polyphony");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobInt {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.polyphony,
                                    },
                                );
                                ui.label(format!("{}/{}", params.polyphony.value(), MAX_POLYPHONY))
                            });

                            ui.vertical(|ui| {
                                ui.label("Oct transpose");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobInt {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.transpose_oct,
                                    },
                                );
                                ui.label(format!("{}", params.transpose_oct.value()))
                            });
                            ui.vertical(|ui| {
                                ui.label("ST transpose");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobInt {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.transpose_semitones,
                                    },
                                );
                                ui.label(format!("{}", params.transpose_semitones.value()))
                            });
                            ui.vertical(|ui| {
                                ui.label("Fine tune");
                                ui.add_sized(
                                    KNOB_SIZE,
                                    ParamKnobFloat {
                                        setter,
                                        color: Color32::RED,
                                        param: &params.fine_tune,
                                    },
                                );
                                ui.label(format!(
                                    "{:.3}{}",
                                    params.fine_tune.value(),
                                    params.fine_tune.unit()
                                ))
                            });
                        });
                    });
                    //
                    ui.horizontal_centered(|ui| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Attack");
                                    ui.add(nih_plug_egui::widgets::ParamSlider::for_param(
                                        &params.attack,
                                        setter,
                                    ))
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Decay");
                                    ui.add(nih_plug_egui::widgets::ParamSlider::for_param(
                                        &params.decay,
                                        setter,
                                    ))
                                });
                            });
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Sustain");
                                    ui.add(nih_plug_egui::widgets::ParamSlider::for_param(
                                        &params.sustain,
                                        setter,
                                    ))
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Release");
                                    ui.add(nih_plug_egui::widgets::ParamSlider::for_param(
                                        &params.release,
                                        setter,
                                    ))
                                });
                            });
                        });
                    });
                });
            },
        )
    }
}
