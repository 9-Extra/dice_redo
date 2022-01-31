mod dice;

use eframe::{egui, epi};
use eframe::egui::Visuals;

pub struct Application {
    name: &'static str,

    dice_feature: dice::DiceFeature,
}

impl Application {

    pub fn new(title: &'static str) -> Application {
        Application {
            name: title,
            dice_feature: Default::default(),
        }
    }
}

impl epi::App for Application {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });
        self.dice_feature.update(ctx);
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        _ctx.set_visuals(Visuals::light());
    }

    fn name(&self) -> &str {
        self.name
    }
}
