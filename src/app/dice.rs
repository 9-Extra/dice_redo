use eframe::egui;
use rand::Rng;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;

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
    pub fn vaild(&self) -> bool {
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
            if plus{ s.push_str(PLUS); } else { plus = true; }
            s.push_str(&format!("{}D6", self.d6));
        }

        if self.d8 != 0 {
            if plus{ s.push_str(PLUS); } else { plus = true; }
            s.push_str(&format!("{}D8", self.d8));
        }

        if self.d20 != 0 {
            if plus{ s.push_str(PLUS); } else { plus = true; }
            s.push_str(&format!("{}D20", self.d20));
        }

        if self.d20 != 0 {
            if plus{ s.push_str(PLUS); } else { plus = true; }
            s.push_str(&format!("{}D6", self.d20));
        }

        if self.d100 != 0 {
            if plus{ s.push_str(PLUS); } else { plus = true; }
            s.push_str(&format!("{}D100", self.d100));
        }

        if self.constant != 0 {
            if plus{ s.push_str(PLUS)};
            s.push_str(&format!("{}", self.constant));
        }
        s
    }
}

impl DicesState {
    pub fn roll(&self, rd: &mut rand::rngs::ThreadRng) -> Rc<RollRecord> {
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

        Rc::new(RollRecord {
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

#[derive(Clone)]
struct RecordWindow {
    record: Rc<RollRecord>,
    should_close: bool,
}

impl RecordWindow {
    pub fn new(record: &Rc<RollRecord>) -> RecordWindow {
        RecordWindow {
            record: record.clone(),
            should_close: true,
        }
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }

    pub fn show(&mut self, ctx: &egui::CtxRef) {
        let record = self.record.clone();
        egui::Window::new(egui::RichText::new(
            record.time.format("[%H:%M:%S]  => ").to_string() + &record.total.to_string(),
        ))
        .collapsible(true)
        .vscroll(true)
        .open(&mut self.should_close)
        .show(ctx, |ui| {
            egui::Grid::new(Rc::as_ptr(&record))
                .striped(true)
                .show(ui, |ui| {
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
                    ui.label(egui::RichText::new(self.record.total.to_string()).heading().color(egui::Color32::RED));
                });
        });
    }
}

#[derive(Default)]
struct RecordManager {
    table: std::collections::VecDeque<Rc<RollRecord>>,
    detail_windows: Vec<RecordWindow>,
}

impl RecordManager {
    pub fn add_record(&mut self, record: Rc<RollRecord>) {
        if self.table.len() >= RECORD_MAX_NUM {
            self.table.pop_front();
        }
        self.table.push_back(record);
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
                            self.show_record_table(ui);
                        });
                });

                self.show_record_windows(ctx);

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    let response = ui.add(egui::Button::new(egui::RichText::new("clear").strong()));
                    if response.clicked() && self.table.len() > 1{
                        let back = self.table.back().unwrap().clone();
                        self.table.clear();
                        self.table.push_back(back);
                    }
                    if response.double_clicked(){
                        self.table.clear();
                    }

                    response.on_hover_ui(|ui: &mut egui::Ui| {
                        ui.label("One-Click to remain only one record.");
                        ui.label("Double-Click to remove all.");
                    });

                    ui.separator();
                });
            });
    }

    fn show_record_table(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("record_table")
            .min_col_width(50.0)
            .striped(true)
            .show(ui, |ui| {
                ui.heading("Time");
                ui.heading("Description");
                ui.heading("Result");
                ui.heading("Detail");

                ui.end_row();

                if !self.table.is_empty() {
                    for i in 0..self.table.len() - 1 {
                        let record = &self.table[i];
                        ui.strong(record.time.format("%H:%M:%S").to_string());
                        ui.label(&record.description);
                        ui.add_space(10.0);
                        ui.heading(
                            egui::RichText::new(record.total.to_string())
                                .color(egui::Color32::DARK_GREEN)
                                .text_style(egui::TextStyle::Monospace),
                        );
                        if std::rc::Rc::strong_count(record) == 1 && ui.button("show").clicked() {
                            self.detail_windows.push(RecordWindow::new(record));
                        }
                        ui.end_row();
                    }

                    let record = self.table.back().unwrap();
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
                    if std::rc::Rc::strong_count(record) == 1 && ui.button("show").clicked() {
                        self.detail_windows.push(RecordWindow::new(record));
                    }
                }
            });
    }

    fn show_record_windows(&mut self, ctx: &egui::CtxRef) {
        self.detail_windows = self
            .detail_windows
            .iter()
            .cloned()
            .filter(|w| w.should_close())
            .collect();

        for w in self.detail_windows.iter_mut() {
            w.show(ctx);
        }
    }
}

pub struct DiceFeature {
    state: DicesState,
    records: RecordManager,

    rd: RefCell<rand::rngs::ThreadRng>,
}

impl DiceFeature {
    fn show_select_panel(&mut self, ui: &mut egui::Ui) {
        let clear = egui::Button::new(
            egui::RichText::new("Reset all")
                .heading()
                .color(egui::Color32::DARK_BLUE),
        );
        if ui.add(clear).clicked() {
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
                    if ui.add_sized([200.0, 50.0], roll).clicked() && self.state.vaild() {
                        self.records
                            .add_record(self.state.roll(&mut self.rd.borrow_mut()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |_| {});

        self.records.update(ctx);
    }
}

impl Default for DiceFeature {
    fn default() -> Self {
        DiceFeature {
            state: DicesState::default(),
            records: RecordManager::default(),
            rd: RefCell::new(rand::thread_rng()),
        }
    }
}
