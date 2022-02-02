use eframe::egui;
use rand::Rng;
use rodio::source::Buffered;
use rodio::{Decoder, Source};
use std::cell::RefCell;
use std::default::Default;
use std::fs::{DirEntry, File};
use std::io::BufReader;
use std::os::windows::ffi::OsStrExt;

struct RollRecord {
    d4: Vec<i32>,
    d6: Vec<i32>,
    d8: Vec<i32>,
    d20: Vec<i32>,
    d100: Vec<i32>,
    state: DicesState,

    time: chrono::NaiveTime,
    description: String,
    total: i32,
}

#[derive(Default, Clone)]
struct DicesState {
    d4: i32,
    d6: i32,
    d8: i32,
    d20: i32,
    d100: i32,
    constant: i32,
}

impl DicesState {
    pub const fn new(state: [i32; 6]) -> DicesState {
        DicesState {
            d4: state[0],
            d6: state[1],
            d8: state[2],
            d20: state[3],
            d100: state[4],
            constant: state[5],
        }
    }

    pub fn valid(&self) -> bool {
        self.d4 != 0 || self.d6 != 0 || self.d8 != 0 || self.d20 != 0 || self.d100 != 0
    }

    pub fn gen_description(&self) -> String {
        let mut s = String::new();
        let mut plus = false;
        const PLUS: &str = " + ";
        if self.d4 != 0 {
            s.push_str(&format!("{}D4", self.d4));
            plus = true;
        }

        if self.d6 != 0 {
            if plus {
                s.push_str(PLUS);
            } else {
                plus = true;
            }
            s.push_str(&format!("{}D6", self.d6));
        }

        if self.d8 != 0 {
            if plus {
                s.push_str(PLUS);
            } else {
                plus = true;
            }
            s.push_str(&format!("{}D8", self.d8));
        }

        if self.d20 != 0 {
            if plus {
                s.push_str(PLUS);
            } else {
                plus = true;
            }
            s.push_str(&format!("{}D20", self.d20));
        }

        if self.d20 != 0 {
            if plus {
                s.push_str(PLUS);
            } else {
                plus = true;
            }
            s.push_str(&format!("{}D6", self.d20));
        }

        if self.d100 != 0 {
            if plus {
                s.push_str(PLUS);
            } else {
                plus = true;
            }
            s.push_str(&format!("{}D100", self.d100));
        }

        if self.constant != 0 {
            if plus {
                s.push_str(PLUS)
            };
            s.push_str(&format!("{}", self.constant));
        }
        s
    }
}

impl DicesState {
    pub fn roll(&self, rd: &mut rand::rngs::ThreadRng) -> Box<RollRecord> {
        let mut sum = 0;

        let mut d4 = Vec::with_capacity(self.d4 as usize);
        for _ in 0..self.d4 {
            let r = rd.gen_range(0..=4);
            sum += r;
            d4.push(r);
        }

        let mut d6 = Vec::with_capacity(self.d6 as usize);
        for _ in 0..self.d6 {
            let r = rd.gen_range(0..=6);
            sum += r;
            d6.push(r);
        }

        let mut d8 = Vec::with_capacity(self.d8 as usize);
        for _ in 0..self.d8 {
            let r = rd.gen_range(0..=8);
            sum += r;
            d8.push(r);
        }

        let mut d20 = Vec::with_capacity(self.d20 as usize);
        for _ in 0..self.d20 {
            let r = rd.gen_range(0..=20);
            sum += r;
            d20.push(r);
        }

        let mut d100 = Vec::with_capacity(self.d100 as usize);
        for _ in 0..self.d100 {
            let r = rd.gen_range(0..=100);
            sum += r;
            d100.push(r);
        }

        Box::new(RollRecord {
            d4,
            d6,
            d8,
            d20,
            d100,
            state: self.clone(),
            time: chrono::Local::now().time(),
            description: self.gen_description(),
            total: sum,
        })
    }
}

const RECORD_MAX_NUM: usize = 32;

struct RecordWindow {
    record: Box<RollRecord>,
    should_open: bool,
}

impl RecordWindow {
    pub fn new(record: Box<RollRecord>) -> RecordWindow {
        RecordWindow {
            record,
            should_open: true,
        }
    }

    pub fn show(record: &RollRecord, should_open: &mut bool, ctx: &egui::CtxRef) {
        egui::Window::new(egui::RichText::new(
            record.time.format("[%H:%M:%S]  => ").to_string() + &record.total.to_string(),
        ))
        .collapsible(true)
        .vscroll(true)
        .drag_bounds(ctx.available_rect())
        .open(should_open)
        .show(ctx, |ui| {
            egui::Grid::new(record.time).striped(true).show(ui, |ui| {
                if record.state.d4 != 0 {
                    ui.strong("D4");
                    record.d4.iter().for_each(|n| {
                        ui.label(n.to_string());
                    });
                    ui.end_row();
                }
                if record.state.d6 != 0 {
                    ui.strong("D6");
                    record.d6.iter().for_each(|n| {
                        ui.label(n.to_string());
                    });
                    ui.end_row();
                }
                if record.state.d8 != 0 {
                    ui.strong("D8");
                    record.d8.iter().for_each(|n| {
                        ui.label(n.to_string());
                    });
                    ui.end_row();
                }
                if record.state.d20 != 0 {
                    ui.strong("D20");
                    record.d20.iter().for_each(|n| {
                        ui.label(n.to_string());
                    });
                    ui.end_row();
                }
                if record.state.d100 != 0 {
                    ui.strong("D100");
                    record.d100.iter().for_each(|n| {
                        ui.label(n.to_string());
                    });
                    ui.end_row();
                }
                if record.state.constant != 0 {
                    ui.strong("Const");
                    ui.label(record.state.constant.to_string());
                    ui.end_row();
                }

                ui.heading("Result:");
                ui.label(
                    egui::RichText::new(record.total.to_string())
                        .heading()
                        .color(egui::Color32::RED),
                );
            });
        });
    }
}

struct RecordLine {
    record: Box<RollRecord>,
    is_detail_show: bool,
}
impl RecordLine {
    pub fn new(record: Box<RollRecord>) -> RecordLine {
        RecordLine {
            record,
            is_detail_show: false,
        }
    }
}

#[derive(Default)]
struct RecordManager {
    table: std::collections::VecDeque<RecordLine>,
    remain_windows: std::collections::VecDeque<RecordWindow>,
}

impl RecordManager {
    pub fn add_record(&mut self, record: Box<RollRecord>) {
        if self.table.len() >= RECORD_MAX_NUM {
            let front = self.table.pop_front().unwrap();
            if front.is_detail_show {
                self.remain_windows
                    .push_back(RecordWindow::new(front.record));
            }
        }
        self.table.push_back(RecordLine::new(record));
    }

    pub fn update(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::right("record_panel")
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_max_height(ui.available_height() - 30.0);
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .stick_to_bottom()
                        .show(ui, |ui| {
                            self.show_record_table(ui, ctx);
                        });
                });

                self.show_remain_windows(ctx);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    let response = ui.add(egui::Button::new(egui::RichText::new("clear").strong()));
                    if response.clicked() && self.table.len() > 1 {
                        let back = self.table.pop_back().unwrap();
                        while let Some(line) = self.table.pop_front() {
                            if line.is_detail_show {
                                self.remain_windows
                                    .push_back(RecordWindow::new(line.record));
                            }
                        }
                        self.table.push_back(back);
                    }
                    if response.double_clicked() {
                        while let Some(line) = self.table.pop_front() {
                            if line.is_detail_show {
                                self.remain_windows
                                    .push_back(RecordWindow::new(line.record));
                            }
                        }
                    }

                    response.on_hover_ui(|ui: &mut egui::Ui| {
                        ui.label("Single-Click to remain only one record.");
                        ui.label("Double-Click to remove all.");
                    });

                    ui.separator();
                });
            });
    }

    fn show_record_table(&mut self, ui: &mut egui::Ui, ctx: &egui::CtxRef) {
        let show_check_box = |ui: &mut egui::Ui, line: &mut RecordLine| {
            let check_box = egui::Checkbox::new(&mut line.is_detail_show, "");
            let record = &line.record;
            let tooltip = |ui: &mut egui::Ui| {
                ui.strong("Open a extra window for more detail.");
            };
            ui.add(check_box).on_hover_ui(tooltip);
            if line.is_detail_show {
                RecordWindow::show(record, &mut line.is_detail_show, ctx);
            }
        };

        egui::Grid::new("record_table")
            .min_col_width(50.0)
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Time");
                ui.heading("Description");
                ui.heading("Result");
                ui.heading("Detail");

                ui.end_row();

                let last = self.table.pop_back();
                for line in self.table.iter_mut() {
                    let record = &line.record;
                    ui.strong(record.time.format("%H:%M:%S").to_string());
                    ui.label(&record.description);
                    ui.add_space(10.0);
                    ui.heading(
                        egui::RichText::new(record.total.to_string())
                            .color(egui::Color32::DARK_GREEN)
                            .text_style(egui::TextStyle::Monospace),
                    );
                    show_check_box(ui, line);
                    ui.end_row();
                }
                if let Some(mut line) = last {
                    let record = &line.record;
                    ui.strong(
                        egui::RichText::new(record.time.format("%H:%M:%S").to_string())
                            .color(egui::Color32::RED),
                    );
                    ui.strong(egui::RichText::new(&record.description).color(egui::Color32::RED));
                    ui.add_space(10.0);
                    ui.heading(
                        egui::RichText::new(record.total.to_string())
                            .color(egui::Color32::DARK_RED)
                            .text_style(egui::TextStyle::Monospace),
                    );
                    show_check_box(ui, &mut line);
                    self.table.push_back(line);
                }
            });
    }

    fn show_remain_windows(&mut self, ctx: &egui::CtxRef) {
        for w in self.remain_windows.iter_mut() {
            RecordWindow::show(&w.record, &mut w.should_open, ctx);
        }
    }
}

pub struct DiceFeature {
    state: DicesState,
    records: RecordManager,

    quick_roll: QuickRoll,

    player: SoundPlayer,

    rd: RefCell<rand::rngs::ThreadRng>,
}

impl Default for DiceFeature {
    fn default() -> Self {
        DiceFeature {
            state: DicesState::default(),
            records: RecordManager::default(),
            quick_roll: QuickRoll::new(),
            player: SoundPlayer::new(),
            rd: RefCell::new(rand::thread_rng()),
        }
    }
}

impl DiceFeature {
    fn show_select_panel(&mut self, ui: &mut egui::Ui) {
        let clear = egui::Button::new(
            egui::RichText::new("Reset")
                .heading()
                .color(egui::Color32::DARK_BLUE),
        );
        if ui.add_sized([80.0, 30.0], clear).clicked() {
            self.state.d4 = 0;
            self.state.d6 = 0;
            self.state.d8 = 0;
            self.state.d20 = 0;
            self.state.d100 = 0;
            self.state.constant = 0;
        }

        egui::ScrollArea::vertical()
            .stick_to_bottom()
            .max_height(ui.available_height() - 80.0)
            .show(ui, |ui| {
                egui::Grid::new("Selections")
                    .striped(true)
                    .min_col_width(100.0)
                    .min_row_height(40.0)
                    .show(ui, |ui| {
                        ui.heading(format!("{}D4", self.state.d4));
                        DiceFeature::generate_buttons(&mut self.state.d4, ui);
                        ui.end_row();
                        ui.heading(format!("{}D6", self.state.d6));
                        DiceFeature::generate_buttons(&mut self.state.d6, ui);
                        ui.end_row();
                        ui.heading(format!("{}D8", self.state.d8));
                        DiceFeature::generate_buttons(&mut self.state.d8, ui);
                        ui.end_row();
                        ui.heading(format!("{}D20", self.state.d20));
                        DiceFeature::generate_buttons(&mut self.state.d20, ui);
                        ui.end_row();
                        ui.heading(format!("{}D100", self.state.d100));
                        DiceFeature::generate_buttons(&mut self.state.d100, ui);
                        ui.end_row();
                        ui.heading("Constant");
                        DiceFeature::generate_buttons(&mut self.state.constant, ui);
                        ui.end_row();
                    });
            });
        ui.separator();
    }

    fn generate_buttons(num: &mut i32, ui: &mut egui::Ui) {
        ui.add(
            egui::DragValue::new(num)
                .clamp_range(0..=i32::MAX)
                .speed(0.05),
        );
    }

    pub fn update(&mut self, ctx: &egui::CtxRef) {
        self.quick_roll.update(
            &mut self.records,
            &self.player,
            &mut self.rd.borrow_mut(),
            ctx,
        );

        self.player.show_err_window(ctx);

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.heading("Selections");
                });

                self.show_select_panel(ui);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("powered by ");
                        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                        ui.label(" and ");
                        ui.hyperlink_to(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/eframe",
                        );
                    });
                    let roll = egui::Button::new(
                        egui::RichText::new("Roll")
                            .heading()
                            .color(egui::Color32::RED),
                    );

                    let tool_tip = |ui: &mut egui::Ui| {
                        ui.label("There will be no sound if you Right-Click.");
                    };
                    let response = ui.add_sized([200.0, 50.0], roll).on_hover_ui(tool_tip);
                    if response.clicked_by(egui::PointerButton::Primary) && self.state.valid() {
                        self.player.play(&mut self.rd.borrow_mut());
                        self.records
                            .add_record(self.state.roll(&mut self.rd.borrow_mut()));
                    }

                    if response.clicked_by(egui::PointerButton::Secondary) && self.state.valid() {
                        self.records
                            .add_record(self.state.roll(&mut self.rd.borrow_mut()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                let close = egui::Button::new(egui::RichText::new("close all").strong());

                let tool_tip = |ui: &mut egui::Ui| {
                    ui.label("Double-Click to close all the detail windows.");
                };
                if ui
                    .add_sized([100.0, 30.0], close)
                    .on_hover_ui(tool_tip)
                    .double_clicked()
                {
                    self.records.remain_windows.clear();
                    self.records
                        .table
                        .iter_mut()
                        .for_each(|l| l.is_detail_show = false);
                };

                if !self.quick_roll.is_show {
                    let show = egui::Button::new(egui::RichText::new("quick roll").strong());
                    if ui.add_sized([100.0, 30.0], show).clicked() {
                        self.quick_roll.is_show = true;
                    }
                }
            });
        });

        self.records.update(ctx);
    }
}

struct QuickRoll {
    is_show: bool,
}

impl QuickRoll {
    pub fn new() -> QuickRoll {
        QuickRoll { is_show: true }
    }

    const STATE_1D4: DicesState = DicesState::new([1, 0, 0, 0, 0, 0]);
    const STATE_3D4: DicesState = DicesState::new([3, 0, 0, 0, 0, 0]);
    const STATE_1D6: DicesState = DicesState::new([0, 1, 0, 0, 0, 0]);
    const STATE_3D6: DicesState = DicesState::new([0, 3, 0, 0, 0, 0]);
    const STATE_1D100: DicesState = DicesState::new([0, 0, 0, 0, 1, 0]);

    pub fn update(
        &mut self,
        records: &mut RecordManager,
        player: &SoundPlayer,
        rd: &mut rand::rngs::ThreadRng,
        ctx: &egui::CtxRef,
    ) {
        egui::Window::new("QuickRoll")
            .auto_sized()
            .open(&mut self.is_show)
            .show(ctx, |ui| {
                let mut add_button = |name: &str, state: &DicesState| {
                    let response = ui.add_sized(
                        egui::Vec2::new(70.0, 30.0),
                        egui::Button::new(egui::RichText::new(name).heading()),
                    );
                    if response.clicked_by(egui::PointerButton::Primary) {
                        player.play(rd);
                        records.add_record(state.roll(rd));
                    };

                    if response.clicked_by(egui::PointerButton::Secondary) {
                        records.add_record(state.roll(rd));
                    }
                };

                add_button("1D4", &QuickRoll::STATE_1D4);
                add_button("3D4", &QuickRoll::STATE_3D4);
                add_button("1D6", &QuickRoll::STATE_1D6);
                add_button("3D6", &QuickRoll::STATE_3D6);
                add_button("1D100", &QuickRoll::STATE_1D100);
            });
    }
}

struct SoundPlayer {
    output: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
    sounds: Vec<Buffered<Decoder<BufReader<File>>>>,
    error_message: Vec<String>,
    is_error_window_show: bool,
}

impl SoundPlayer {
    pub fn new() -> SoundPlayer {
        let mut error_message = Vec::new();
        let mut sounds = Vec::new();

        let output = match rodio::OutputStream::try_default() {
            Ok(output) => Option::Some(output),
            Err(e) => {
                error_message.push(format!("Fail to initialize output device for: {}", e));
                Option::None
            }
        };
        if let Ok(files) = std::fs::read_dir("assets/"){
            files.filter_map(|f|{f.ok()})
                .filter(|f|{if let Ok(t) = f.file_type() {t.is_file()} else { false }})
                .for_each(|f|{
                    if let Ok(file) = std::fs::File::open(f.path()) {
                        let reader = std::io::BufReader::new(file);
                        if let Ok(sound) = Decoder::new(reader){
                            sounds.push(sound.buffered());
                        } else {
                            error_message.push(format!("Fail to decode: {}", f.file_name().to_str().unwrap_or("Unknown")));
                        }
                    } else {
                        error_message.push(format!("Fail to read: {}", f.file_name().to_str().unwrap_or("Unknown")));
                    }
            })
        }

        if sounds.is_empty(){
            error_message.push("No sound is found!".to_string());
        }

        let is_error_window_show = !error_message.is_empty();

        SoundPlayer {
            output,
            sounds,
            error_message,
            is_error_window_show,
        }
    }

    pub fn show_err_window(&mut self, ctx: &egui::CtxRef) {
        egui::Window::new(egui::RichText::new("Warning").color(egui::Color32::RED))
            .open(&mut self.is_error_window_show)
            .auto_sized()
            .collapsible(false)
            .show(ctx, |ui| {
                for str in &self.error_message {
                    ui.heading(egui::RichText::new(str).color(egui::Color32::RED));
                }
            });
    }

    pub fn play(&self, rd: &mut rand::rngs::ThreadRng) {
        //println!("play sound{}",index);
        if let Some((_, output)) = &self.output {
            if let Err(e) = output.play_raw(
                self.sounds[rd.gen_range(0..self.sounds.len())]
                    .clone()
                    .convert_samples(),
            ) {
                println!("{}", e);
            }
        }
    }
}
