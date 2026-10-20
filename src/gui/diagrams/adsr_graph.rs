use itertools::Itertools;
use nih_plug_egui::egui::{self, Color32, Stroke, Ui, Vec2, pos2};

use crate::sampler::adsr::AdsrParams;

const GRAPH_HEIGHT: f32 = 100.0;
const GRAPH_WIDTH: f32 = 300.0;

pub fn draw_adsr_graph(ui: &mut Ui, params: AdsrParams) {
    let fractions: [f32; 5] = [0.0, params.attack, params.decay, 1.0, params.release];
    let all_fractions: f32 = fractions.iter().sum();
    let fractions = fractions.iter().scan(0.0, |acc, &x| {
        *acc += x;
        Some(*acc)
    });
    let phase_end_values = [0.0, 1.0, params.sustain, params.sustain, 0.0];
    let colors = [Color32::RED, Color32::GREEN, Color32::PURPLE, Color32::BLUE];
    let (rect, _) = ui.allocate_exact_size(
        Vec2 {
            x: GRAPH_WIDTH,
            y: GRAPH_HEIGHT,
        },
        nih_plug_egui::egui::Sense::empty(),
    );

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
    for (index, pos) in fractions
        .into_iter()
        .zip(phase_end_values.into_iter())
        .map(|(x, y)| {
            pos2(
                rect.left() + GRAPH_WIDTH * x / all_fractions,
                rect.bottom() - y * GRAPH_HEIGHT,
            )
        })
        .array_windows::<2>()
        .enumerate()
    {
        painter.line_segment(pos, Stroke::new(1.5, colors[index]));
    }
}
