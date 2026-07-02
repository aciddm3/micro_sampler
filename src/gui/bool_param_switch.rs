use nih_plug::{context::gui::ParamSetter, params::BoolParam};
use nih_plug_egui::egui::{Color32, Rect, Stroke};

pub struct BoolParamSwitcher<'a> {
    pub setter: &'a ParamSetter<'a>,
    pub param: &'a BoolParam,
}

impl<'a> nih_plug_egui::egui::Widget for BoolParamSwitcher<'a> {
    fn ui(self, ui: &mut nih_plug_egui::egui::Ui) -> nih_plug_egui::egui::Response {
        let desired_size = ui.available_size_before_wrap();
        let (rect, response) =
            ui.allocate_exact_size(desired_size, nih_plug_egui::egui::Sense::click());

        let val = self.param.value();

        if ui.is_rect_visible(rect) {
            let corner_radius = desired_size.x.min(desired_size.y) / 2.0;

            let color = match val {
                true => Color32::GREEN,
                false => Color32::RED,
            };
            ui.painter().rect(
                rect,
                corner_radius,
                Color32::TRANSPARENT,
                Stroke::new(corner_radius / 2.0, color),
                nih_plug_egui::egui::StrokeKind::Inside,
            );
            ui.painter().rect(
                Rect::from_two_pos(
                    rect.center_top(),
                    match val {
                        true => rect.left_bottom(),
                        false => rect.right_bottom(),
                    },
                ),
                corner_radius,
                color,
                Stroke::NONE,
                nih_plug_egui::egui::StrokeKind::Middle,
            );
        }

        if response.hovered() {
            use nih_plug_egui::egui::CursorIcon;
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        if response.clicked() {
            self.setter.begin_set_parameter(self.param);
            self.setter.set_parameter(self.param, !val);
            self.setter.end_set_parameter(self.param);
        }

        response
    }
}
