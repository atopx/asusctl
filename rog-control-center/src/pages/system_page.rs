use crate::system_state::SystemState;
use crate::widgets::{
    anime_power_group, app_settings, aura_power_group, platform_profile, rog_bios_group,
};
use crate::RogApp;

impl RogApp {
    pub fn system_page(&mut self, states: &mut SystemState, ctx: &egui::Context) {
        let Self {
            config, supported, ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Base settings");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                let rect = ui.available_rect_before_wrap();
                egui::Grid::new("grid_of_bits")
                    .min_col_width(rect.width() / 2.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.separator();
                            if supported.platform_profile.platform_profile {
                                platform_profile(states, ui);
                            }
                        });
                        ui.vertical(|ui| {
                            ui.separator();
                            aura_power_group(supported, states, ui);
                        });
                        ui.end_row();

                        ui.vertical(|ui| {
                            ui.separator();
                            app_settings(config, states, ui);
                        });
                        ui.vertical(|ui| {
                            ui.separator();
                            rog_bios_group(supported, states, ui);
                        });
                        ui.end_row();

                        ui.vertical(|ui| {
                            ui.separator();
                            if supported.anime_ctrl.0 {
                                anime_power_group(supported, states, ui);
                            }
                        });
                        ui.vertical(|ui| {
                            ui.separator();
                        });
                        ui.end_row();
                    });
            });
        });
    }
}
