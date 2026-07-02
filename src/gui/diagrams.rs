use std::sync::Arc;

use itertools::Itertools;
use nih_plug_egui::egui::{self, Color32, Pos2, Stroke, Ui, Vec2};
use parking_lot::RwLock;

use crate::sampler::{
    decoded_audio::DecodedAudio, params::SamplerParams, sample_player::table_value,
};

const GRAPH_HEIGHT: f32 = 100.0;
const HALF_GRAPH_HEIGHT: f32 = GRAPH_HEIGHT / 2.0;

pub fn draw_sample_graph(
    ui: &mut Ui,
    decoded_audio_arc: &Arc<RwLock<DecodedAudio>>,
    params: &SamplerParams,
) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        Vec2 {
            x: width,
            y: GRAPH_HEIGHT,
        },
        nih_plug_egui::egui::Sense::empty(),
    );

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
    let decoded_audio_guard = decoded_audio_arc.clone();
    for (p_n, p_n1) in (0..(width as usize))
        .map(|x| Pos2 {
            x: x as f32 + rect.left(),
            y: -table_value(x as f32 / width, &decoded_audio_guard.read()) * HALF_GRAPH_HEIGHT
                + HALF_GRAPH_HEIGHT
                + rect.top(),
        })
        .tuple_windows::<(Pos2, Pos2)>()
        .into_iter()
    {
        painter.circle(p_n1, 1.0, Color32::LIGHT_YELLOW, Stroke::NONE);
        painter.line_segment([p_n, p_n1], Stroke::new(0.25, Color32::YELLOW));
    }
    //
    let (play_start, play_end, loop_start, loop_end, loop_enabled) = (
        params.play_start.value() * width + rect.left(),
        params.play_end.value() * width + rect.left(),
        params.loop_start.value() * width + rect.left(),
        params.loop_end.value() * width + rect.left(),
        params.loop_enable.value(),
    );
    //
    painter.line(
        vec![
            Pos2::new(play_start, rect.top()),
            Pos2::new(play_start, GRAPH_HEIGHT + rect.top()),
        ],
        Stroke::new(3.0, Color32::WHITE),
    );
    painter.line(
        vec![
            Pos2::new(play_start + 5.0, GRAPH_HEIGHT + rect.top()),
            Pos2::new(play_start, GRAPH_HEIGHT + rect.top()),
        ],
        Stroke::new(5.0, Color32::WHITE),
    );
    painter.line(
        vec![
            Pos2::new(play_start + 5.0, rect.top()),
            Pos2::new(play_start, rect.top()),
        ],
        Stroke::new(5.0, Color32::WHITE),
    );
    //
    painter.line(
        vec![
            Pos2::new(play_end, rect.top()),
            Pos2::new(play_end, GRAPH_HEIGHT + rect.top()),
        ],
        Stroke::new(3.0, Color32::WHITE),
    );
    painter.line(
        vec![
            Pos2::new(play_end - 5.0, GRAPH_HEIGHT + rect.top()),
            Pos2::new(play_end, GRAPH_HEIGHT + rect.top()),
        ],
        Stroke::new(5.0, Color32::WHITE),
    );
    painter.line(
        vec![
            Pos2::new(play_end - 5.0, rect.top()),
            Pos2::new(play_end, rect.top()),
        ],
        Stroke::new(5.0, Color32::WHITE),
    );
    //
    if loop_enabled {
        painter.line(
            vec![
                Pos2::new(loop_start, rect.top()),
                Pos2::new(loop_start, GRAPH_HEIGHT + rect.top()),
            ],
            Stroke::new(3.0, Color32::BLUE),
        );
        painter.line(
            vec![
                Pos2::new(loop_start + 5.0, GRAPH_HEIGHT + rect.top()),
                Pos2::new(loop_start, GRAPH_HEIGHT + rect.top()),
            ],
            Stroke::new(5.0, Color32::BLUE),
        );
        painter.line(
            vec![
                Pos2::new(loop_start + 5.0, rect.top()),
                Pos2::new(loop_start, rect.top()),
            ],
            Stroke::new(5.0, Color32::BLUE),
        );
        //
        painter.line(
            vec![
                Pos2::new(loop_end, rect.top()),
                Pos2::new(loop_end, GRAPH_HEIGHT + rect.top()),
            ],
            Stroke::new(3.0, Color32::BLUE),
        );
        painter.line(
            vec![
                Pos2::new(loop_end - 5.0, GRAPH_HEIGHT + rect.top()),
                Pos2::new(loop_end, GRAPH_HEIGHT + rect.top()),
            ],
            Stroke::new(5.0, Color32::BLUE),
        );
        painter.line(
            vec![
                Pos2::new(loop_end - 5.0, rect.top()),
                Pos2::new(loop_end, rect.top()),
            ],
            Stroke::new(5.0, Color32::BLUE),
        );
    }
}
