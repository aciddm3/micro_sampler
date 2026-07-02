use std::f32::consts::FRAC_PI_2;

use nih_plug::{
    params::{FloatParam, Param},
    prelude::ParamSetter,
};
use nih_plug_egui::egui::{Color32, Shape, Stroke, epaint::PathStroke, pos2};

pub struct ParamKnobFloat<'a> {
    pub setter: &'a ParamSetter<'a>,
    pub color: Color32,
    pub param: &'a FloatParam,
}

impl<'a> nih_plug_egui::egui::Widget for ParamKnobFloat<'a> {
    fn ui(self, ui: &mut nih_plug_egui::egui::Ui) -> nih_plug_egui::egui::Response {
        let desired_size = ui.available_size_before_wrap();
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, nih_plug_egui::egui::Sense::click_and_drag());

        let mut uv_value = self.param.modulated_normalized_value();
        let mut value_changed = false;

        if response.drag_started() {
            self.setter.begin_set_parameter(self.param);
        }

        if response.double_clicked() {
            uv_value = self.param.default_normalized_value();
            value_changed = true;
        }

        if response.hovered() {
            use nih_plug_egui::egui::CursorIcon::ResizeVertical;
            ui.ctx().set_cursor_icon(ResizeVertical);

            let scroll_delta = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.0 {
                self.setter.begin_set_parameter(self.param);
                uv_value = (uv_value + scroll_delta * 0.01).clamp(0.0, 1.0);
                value_changed = true;
                self.setter.end_set_parameter(self.param); // Сразу завершаем для скролла
            }
        }

        if response.dragged() {
            let delta_y = response.drag_delta().y;
            
            uv_value = (uv_value - delta_y * 0.01).clamp(0.0, 1.0);
            value_changed = true;
        }

        if value_changed {
            let plain_value = self.param.preview_plain(uv_value);
            self.setter.set_parameter(self.param, plain_value);
            response.mark_changed();
        }

        if response.drag_stopped() {
            self.setter.end_set_parameter(self.param);
        }

        if ui.is_rect_visible(rect) {
            let knob_radius = rect.width().min(rect.height()) / 2.0;
            let center = rect.center();

            let draw_color = if response.hovered() || response.dragged() {
                // Делаем цвет чуть ярче, не мутируя self (так как self сейчас исчезнет)
                let mut c = self.color;
                c[0] = c[0].saturating_add(40);
                c[1] = c[1].saturating_add(40);
                c[2] = c[2].saturating_add(40);
                c
            } else {
                self.color
            };

            let ang = (0.875 - uv_value) * 4.19;
            let stick_points = vec![
                pos2(
                    knob_radius * ang.cos() + center.x,
                    -knob_radius * ang.sin() + center.y,
                ),
                pos2(
                    0.1 * knob_radius * (ang + FRAC_PI_2).cos() + center.x,
                    -0.1 * knob_radius * (ang + FRAC_PI_2).sin() + center.y,
                ),
                pos2(
                    0.1 * knob_radius * (ang - FRAC_PI_2).cos() + center.x,
                    -0.1 * knob_radius * (ang - FRAC_PI_2).sin() + center.y,
                ),
            ];

            ui.painter()
                .circle_stroke(center, 0.9 * knob_radius, Stroke::new(0.6, self.color));
            ui.painter()
                .add(Shape::Path(nih_plug_egui::egui::epaint::PathShape {
                    points: stick_points,
                    closed: true,
                    fill: draw_color,
                    stroke: PathStroke::NONE,
                }));
        }
        response
    }
}
