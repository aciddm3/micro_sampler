use nih_plug::{
    context::gui::ParamSetter,
    params::{IntParam, Param, range::IntRange},
};
use nih_plug_egui::egui::{Color32, CursorIcon, Shape, Stroke, Ui, Widget, epaint::PathStroke, pos2};

use std::f32::consts::FRAC_PI_2;
pub struct ParamKnobInt<'a> {
    pub param: &'a IntParam,
    pub color: Color32,
    pub setter: &'a ParamSetter<'a>,
}

impl<'a> Widget for ParamKnobInt<'a> {
    fn ui(self, ui: &mut Ui) -> nih_plug_egui::egui::Response {
        let desired_size = ui.available_size_before_wrap();
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, nih_plug_egui::egui::Sense::click_and_drag());

        let (param_min, param_max) = match self.param.range() {
            IntRange::Linear { min, max } => (min, max),
            _ => (0, 0), 
        };
        let divisor = (param_max - param_min).max(1);

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
            ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
            let scroll_delta = ui.input(|input| input.smooth_scroll_delta.y);
            
            if scroll_delta.abs() > 0.0 {
                self.setter.begin_set_parameter(self.param);
                uv_value = (uv_value + scroll_delta * 0.01).clamp(0.0, 1.0);
                value_changed = true;
                self.setter.end_set_parameter(self.param);
            }
        }

        if response.dragged() {
            let delta_y = response.drag_delta().y;
            uv_value = (uv_value - delta_y * 0.005).clamp(0.0, 1.0);
            value_changed = true;
        }

        if value_changed {
            let plain_val = self.param.preview_plain(uv_value);
            self.setter.set_parameter(self.param, plain_val);
            response.mark_changed();
            
            uv_value = self.param.preview_normalized(plain_val);
        }

        if response.drag_stopped() {
            self.setter.end_set_parameter(self.param);
        }

        if ui.is_rect_visible(rect) {
            let knob_radius = rect.width().min(rect.height()) / 2.0;
            let center = rect.center();

            let divisor_f = divisor as f32;
            for i in 0..=divisor {
                let val = (0.875 - i as f32 / divisor_f) * 4.19;
                ui.painter().line_segment(
                    [
                        pos2(
                            knob_radius * val.cos() + center.x,
                            -knob_radius * val.sin() + center.y,
                        ),
                        pos2(
                            1.1 * knob_radius * val.cos() + center.x,
                            -1.1 * knob_radius * val.sin() + center.y,
                        ),
                    ],
                    Stroke::new(1.0, self.color), // Сделал засечки тоже реагирующими на цвет
                );
            }

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

            ui.painter().circle_stroke(center, 0.9 * knob_radius, Stroke::new(1.0, self.color));
            ui.painter().add(Shape::Path(nih_plug_egui::egui::epaint::PathShape {
                points: stick_points,
                closed: true,
                fill: self.color,
                stroke: PathStroke::NONE,
            }));
        }

        response
    }
}
