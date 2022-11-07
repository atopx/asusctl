use crate::{
    page_states::{FanCurvesState, PageDataStates},
    widgets::fan_graphs,
    RogApp,
};
use egui::Ui;
use rog_dbus::RogDbusClient;
use rog_platform::supported::SupportedFunctions;
use rog_profiles::Profile;

impl RogApp {
    pub async fn fan_curve_page(&mut self, ctx: &egui::Context) {
        let Self {
            supported,
            states,
            asus_dbus: dbus,
            ..
        } = self;

        let mut states = states.lock().await;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Custom fan curves");
            ui.label("A fan curve is only active when the related profile is active and the curve is enabled");
            Self::fan_curve(
                supported,
                &mut states,
                dbus,
                ui,
            );

            fan_graphs(supported, &mut states, dbus, ui);
        });
    }

    fn fan_curve(
        supported: &SupportedFunctions,
        states: &mut PageDataStates,
        dbus: &RogDbusClient,
        ui: &mut Ui,
    ) {
        ui.separator();
        ui.label("Enabled fan-curves");

        let mut changed = false;
        ui.horizontal(|ui| {
            let mut item = |p: Profile, curves: &mut FanCurvesState, mut checked: bool| {
                if ui
                    .add(egui::Checkbox::new(&mut checked, format!("{:?}", p)))
                    .changed()
                {
                    tokio::task::block_in_place(|| async {
                        dbus.proxies()
                            .profile()
                            .set_fan_curve_enabled(p, checked)
                            .await
                            .map_err(|err| {
                                states.error = Some(err.to_string());
                            })
                            .ok();
                    });

                    if !checked {
                        curves.enabled.remove(&p);
                    } else {
                        curves.enabled.insert(p);
                    }
                    changed = true;
                }
            };

            states.profiles.list.sort();
            for f in states.profiles.list.iter() {
                item(*f, &mut states.fan_curves, states.fan_curves.enabled.contains(f));
            }
        });

        if changed {
            let selected_profile = states.fan_curves.show_curve;
            let selected_pu = states.fan_curves.show_graph;

            let notif = states.fan_curves.was_notified.clone();
            tokio::spawn(async {
                match FanCurvesState::new(notif, supported, dbus).await {
                    Ok(f) => states.fan_curves = f,
                    Err(e) => states.error = Some(e.to_string()),
                }
            });

            states.fan_curves.show_curve = selected_profile;
            states.fan_curves.show_graph = selected_pu;
        }
    }
}
