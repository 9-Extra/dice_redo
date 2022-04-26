use rodio::source::Buffered;
use rodio::{Decoder, Source};
use std::os::windows::ffi::OsStrExt;
use eframe::egui;
use rand::Rng;

pub struct SoundPlayer {
    output: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
    sounds: Vec<Buffered<Decoder<std::io::BufReader<std::fs::File>>>>,
    error_message: std::cell::RefCell<Vec<String>>,
    volume: i32,
    pub(crate) is_control_window_show: bool,
    is_error_window_show: std::cell::RefCell<bool>,
}

impl SoundPlayer {
    pub fn new() -> SoundPlayer {
        let mut error_message = Vec::new();
        let mut sounds = Vec::new();

        let output = match rodio::OutputStream::try_default() {
            Ok(output) => Some(output),
            Err(e) => {
                error_message.push(format!("Fail to initialize output device for: {}", e));
                None
            }
        };

        if let Ok(files) = std::fs::read_dir("assets/") {
            files
                .filter_map(|f| f.ok())
                .filter(|f| {
                    if let Ok(t) = f.file_type() {
                        t.is_file()
                    } else {
                        false
                    }
                })
                .for_each(|f| {
                    if f.file_name().encode_wide().next() != Some('!' as u16) {
                        if let Ok(file) = std::fs::File::open(f.path()) {
                            let reader = std::io::BufReader::new(file);
                            if let Ok(sound) = Decoder::new(reader) {
                                sounds.push(sound.buffered());
                            } else {
                                error_message.push(format!(
                                    "Fail to decode: {}",
                                    f.file_name().to_str().unwrap_or("Unknown")
                                ));
                            }
                        } else {
                            error_message.push(format!(
                                "Fail to read: {}",
                                f.file_name().to_str().unwrap_or("Unknown")
                            ));
                        }
                    }
                })
        }

        if sounds.is_empty() {
            error_message.push("No sound is found!".to_string());
        }

        let is_error_window_show = !error_message.is_empty();

        SoundPlayer {
            output,
            sounds,
            error_message: std::cell::RefCell::new(error_message),
            volume: 50,
            is_control_window_show: true,
            is_error_window_show: std::cell::RefCell::new(is_error_window_show),
        }
    }

    pub fn reset_output_device(&mut self) {
        self.output = match rodio::OutputStream::try_default() {
            Ok(o) => Some(o),
            Err(e) => {
                self.error_message.borrow_mut().clear();
                self.add_err_message(format!("Fail to initialize output device for: {}", e));
                None
            }
        };
    }

    pub fn show_audio_control_window(&mut self, ctx: &egui::CtxRef) {
        let mut is_control_window_show = self.is_control_window_show;
        egui::Window::new("Audio Config")
            .auto_sized()
            .open(&mut is_control_window_show)
            .show(ctx, |ui| {
                egui::Grid::new("audio_controls")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Volume");
                        let slider =
                            egui::Slider::new(&mut self.volume, 0..=100).clamp_to_range(true);
                        ui.add(slider);

                        ui.end_row();

                        if ui.button("show warning").clicked() {
                            *self.is_error_window_show.borrow_mut() = true;
                        }

                        if ui.button("reset device").clicked() {
                            self.reset_output_device();
                        }
                    });
            });
        self.is_control_window_show = is_control_window_show;
    }

    pub fn show_err_window(&self, ctx: &egui::CtxRef) {
        egui::Window::new(egui::RichText::new("Warning").color(egui::Color32::RED))
            .open(&mut self.is_error_window_show.borrow_mut())
            .auto_sized()
            .collapsible(false)
            .show(ctx, |ui| {
                for str in self.error_message.borrow().iter() {
                    ui.heading(egui::RichText::new(str).color(egui::Color32::RED));
                }
            });
    }

    #[inline]
    fn add_err_message(&self, msg: String) {
        self.error_message.borrow_mut().push(msg);
        *self.is_error_window_show.borrow_mut() = true;
    }

    pub fn play(&self, rd: &mut rand::rngs::ThreadRng) {
        //println!("play sound{}",index);
        if let Some((_, output)) = &self.output {
            if !self.sounds.is_empty() {
                if let Err(e) = output.play_raw(
                    self.sounds[rd.gen_range(0..self.sounds.len())]
                        .clone()
                        .amplify(self.volume as f32 / 10.0)
                        .convert_samples(),
                ) {
                    self.add_err_message(format!("Fail to initialize output device for: {}", e));
                }
            }
        }
    }
}