mod dice;

use eframe::egui::Visuals;
use eframe::{egui, epi};

pub struct Application {
    name: &'static str,

    dice_feature: dice::DiceWrapper,
}

impl Application {
    pub fn new(title: &'static str) -> Application {
        Application {
            name: title,
            dice_feature: dice::DiceWrapper::new(),
        }
    }
}

impl epi::App for Application {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, _: &epi::Frame) {
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
        _frame.set_window_size(egui::Vec2::new(1200.0, 600.0));
    }

    fn name(&self) -> &str {
        self.name
    }
}
