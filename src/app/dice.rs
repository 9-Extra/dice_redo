use eframe::egui;
use rand::Rng;
use rodio::source::Buffered;
use rodio::{Decoder, Source};
use std::cell::RefCell;
use std::default::Default;
use std::fs::File;
use std::io::BufReader;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;

const DICE_NUM: usize = 5;
const DICE_TYPE: [i32; DICE_NUM] = [4, 6, 12, 20, 100];

pub struct DiceWrapper {
    dice_feature: DiceFeature<DICE_NUM>,
}

impl DiceWrapper {
    pub fn new() -> DiceWrapper {
        DiceWrapper {
            dice_feature: DiceFeature::new(),
        }
    }
    #[inline]
    pub fn update(&mut self, ctx: &egui::CtxRef) {
        self.dice_feature.update(ctx);
    }
}

struct RollRecord<const N: usize> {
    records: [Vec<i32>; N],
    state: DicesState<N>,

    time: chrono::NaiveTime,
    description: String,
    total: i32,
}

#[derive(Clone)]
struct DicesState<const N: usize> {
    dice_num: [i32; N],
    constant: i32,
}

impl<const N: usize> DicesState<N> {
    pub const fn new(state: [i32; N], constant: i32) -> DicesState<N> {
        DicesState {
            dice_num: state,
            constant,
        }
    }

    pub fn valid(&self) -> bool {
        self.dice_num.iter().any(|&n| n != 0)
    }

    pub fn gen_description(&self) -> String {
        let mut s = String::new();
        let mut plus = false;
        const PLUS: &str = " + ";
        for i in 0..N {
            if self.dice_num[i] != 0 {
                if plus {
                    s.push_str(PLUS);
                } else {
                    plus = true;
                }
                s.push_str(&format!("{}D{}", self.dice_num[i], DICE_TYPE[i]));
            }
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

impl<const N: usize> DicesState<N> {
    pub fn roll(&self, rd: &mut rand::rngs::ThreadRng) -> Box<RollRecord<N>> {
        let mut sum = 0;
        let mut records_raw:[MaybeUninit<Vec<i32>>;N] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..N{
            records_raw[i].write(Vec::new());
        }

        let mut records:[Vec<i32>;N] = unsafe { std::mem::transmute_copy(&records_raw) };
        for i in 0..N{
            records[i].resize_with(self.dice_num[i] as usize, ||{
                let r = rd.gen_range(1..=DICE_TYPE[i]);
                sum += r;
                r
            });
        }

        sum += self.constant;

        Box::new(RollRecord {
            records,
            state: self.clone(),
            time: chrono::Local::now().time(),
            description: self.gen_description(),
            total: sum,
        })
    }
}

const RECORD_MAX_NUM: usize = 1024;

struct RecordWindow<const N: usize> {
    record: Box<RollRecord<N>>,
    should_open: bool,
}

impl<const N: usize> RecordWindow<N> {
    pub fn new(record: Box<RollRecord<N>>) -> RecordWindow<N> {
        RecordWindow {
            record,
            should_open: true,
        }
    }

    pub fn show(record: &RollRecord<N>, should_open: &mut bool, ctx: &egui::CtxRef) {
        egui::Window::new(egui::RichText::new(
            record.time.format("[%H:%M:%S]  => ").to_string() + &record.total.to_string(),
        ))
        .collapsible(true)
        .vscroll(true)
        .drag_bounds(ctx.available_rect())
        .open(should_open)
        .show(ctx, |ui| {
            egui::Grid::new(record.time).striped(true).show(ui, |ui| {
                for i in 0..N {
                    if record.state.dice_num[i] != 0 {
                        ui.strong(format!("D{}", DICE_TYPE[i]));
                        record.records[i].iter().for_each(|n| {
                            ui.label(n.to_string());
                        });
                        ui.end_row();
                    }
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

struct RecordLine<const N: usize> {
    record: Box<RollRecord<N>>,
    is_detail_show: bool,
}
impl<const N: usize> RecordLine<N> {
    pub fn new(record: Box<RollRecord<N>>) -> RecordLine<N> {
        RecordLine {
            record,
            is_detail_show: false,
        }
    }
}

#[derive(Default)]
struct RecordManager<const N: usize> {
    table: std::collections::VecDeque<RecordLine<N>>,
    remain_windows: std::collections::VecDeque<RecordWindow<N>>,
}

impl<const N: usize> RecordManager<N> {
    pub fn add_record(&mut self, record: Box<RollRecord<N>>) {
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
                        let mut new_table = std::collections::VecDeque::new();
                        new_table.push_back(self.table.pop_back().unwrap());
                        std::mem::swap(&mut self.table,&mut new_table);
                        for line in new_table{
                            if line.is_detail_show {
                                self.remain_windows
                                    .push_back(RecordWindow::new(line.record));
                            }
                        }
                    }
                    if response.double_clicked() {
                        let mut new_table = std::collections::VecDeque::new();
                        std::mem::swap(&mut self.table,&mut new_table);
                        for line in new_table{
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
        let show_check_box = |ui: &mut egui::Ui, line: &mut RecordLine<N>| {
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

struct DiceFeature<const N: usize> {
    state: DicesState<N>,
    records: RecordManager<N>,

    quick_roll: QuickRoll<N>,

    player: SoundPlayer,

    rd: RefCell<rand::rngs::ThreadRng>,
}

impl<const N: usize> DiceFeature<N> {
    pub fn new() -> DiceFeature<N> {
        DiceFeature {
            state: DicesState::new([0; N], 0),
            records: RecordManager::default(),
            quick_roll: QuickRoll::new(),
            player: SoundPlayer::new(),
            rd: RefCell::new(rand::thread_rng()),
        }
    }

    fn show_select_panel(&mut self, ui: &mut egui::Ui) {
        let reset = egui::Button::new(
            egui::RichText::new("Reset")
                .heading()
                .color(egui::Color32::DARK_BLUE),
        );
        if ui.add_sized([80.0, 30.0], reset).clicked() {
            self.state.dice_num = [0; N];
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
                        for i in 0..N {
                            ui.heading(format!("{}D{}", self.state.dice_num[i], DICE_TYPE[i]));
                            ui.add(
                                egui::DragValue::new(&mut self.state.dice_num[i])
                                    .clamp_range::<i32>(0..=100)
                                    .speed(0.05),
                            );
                            ui.end_row();
                        }
                        ui.heading("Constant");
                        ui.add(
                            egui::DragValue::new(&mut self.state.constant)
                                .clamp_range::<i32>(0..=100)
                                .speed(0.05),
                        );
                        ui.end_row();
                    });
            });
        ui.separator();
    }

    pub fn update(&mut self, ctx: &egui::CtxRef) {
        self.quick_roll.update(
            &mut self.records,
            &self.player,
            &mut self.rd.borrow_mut(),
            ctx,
        );

        self.player.show_audio_control_window(ctx);
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

                if !self.player.is_control_window_show {
                    let show = egui::Button::new(egui::RichText::new("audio config").strong());
                    if ui.add_sized([100.0, 30.0], show).clicked() {
                        self.player.is_control_window_show = true;
                    }
                }
            });
        });

        self.records.update(ctx);
    }
}

struct QuickRoll<const N: usize> {
    is_show: bool,
}

impl<const N: usize> QuickRoll<N> {
    pub fn new() -> QuickRoll<N> {
        QuickRoll { is_show: true }
    }

    const fn array_n(ori: &[i32]) -> [i32; N] {
        let mut trans = [0; N];
        trans[0] = ori[0];
        trans[1] = ori[1];
        trans[2] = ori[2];
        trans[3] = ori[3];
        trans[4] = ori[4];
        trans
    }

    const fn gen_state(ori: &[i32]) -> DicesState<N> {
        DicesState::new(QuickRoll::array_n(ori), 0)
    }

    const STATE_1D4: DicesState<N> = QuickRoll::gen_state(&[1, 0, 0, 0, 0]);
    const STATE_3D4: DicesState<N> = QuickRoll::gen_state(&[3, 0, 0, 0, 0]);
    const STATE_1D6: DicesState<N> = QuickRoll::gen_state(&[0, 1, 0, 0, 0]);
    const STATE_3D6: DicesState<N> = QuickRoll::gen_state(&[0, 3, 0, 0, 0]);
    const STATE_1D100: DicesState<N> = QuickRoll::gen_state(&[0, 0, 0, 0, 1]);

    pub fn update(
        &mut self,
        records: &mut RecordManager<N>,
        player: &SoundPlayer,
        rd: &mut rand::rngs::ThreadRng,
        ctx: &egui::CtxRef,
    ) {
        egui::Window::new("QuickRoll")
            .auto_sized()
            .open(&mut self.is_show)
            .show(ctx, |ui| {
                let mut add_button = |name: &str, state: &DicesState<N>| {
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
    volume: i32,
    is_control_window_show: bool,
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
            error_message,
            volume: 50,
            is_control_window_show: true,
            is_error_window_show,
        }
    }

    pub fn reset_output_device(
        output: &mut Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
        error_message: &mut Vec<String>,
        is_error_window_show: &mut bool,
    ) {
        *output = match rodio::OutputStream::try_default() {
            Ok(o) => Option::Some(o),
            Err(e) => {
                error_message.clear();
                error_message.push(format!("Fail to initialize output device for: {}", e));
                *is_error_window_show = true;
                Option::None
            }
        };
    }

    pub fn show_audio_control_window(&mut self, ctx: &egui::CtxRef) {
        let Self {
            output,
            sounds: _sounds,
            error_message,
            volume,
            is_control_window_show,
            is_error_window_show,
        } = self;
        egui::Window::new("Audio Config")
            .auto_sized()
            .open(is_control_window_show)
            .show(ctx, |ui| {
                egui::Grid::new("audio_controls")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Volume");
                        let slider = egui::Slider::new(volume, 0..=100).clamp_to_range(true);
                        ui.add(slider);

                        ui.end_row();

                        if ui.button("show warning").clicked() {
                            *is_error_window_show = true;
                        }

                        if ui.button("reset device").clicked() {
                            SoundPlayer::reset_output_device(
                                output,
                                error_message,
                                is_error_window_show,
                            );
                        }
                    });
            });
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
            if !self.sounds.is_empty() {
                if let Err(e) = output.play_raw(
                    self.sounds[rd.gen_range(0..self.sounds.len())]
                        .clone()
                        .amplify(self.volume as f32 / 10.0)
                        .convert_samples(),
                ) {
                    println!("{}", e);
                }
            }
        }
    }
}
