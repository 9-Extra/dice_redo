#![windows_subsystem = "windows"]
fn main() {
    let app = dice_redo::app::TemplateApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}