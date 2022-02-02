#![windows_subsystem = "windows"]
fn main() {
    let app = dice_redo::app::Application::new("Dice_Rebirth V2.1");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
