use std::cell::RefCell;
use std::default::Default;
use std::mem::take;
use std::rc::Rc;
use eframe::egui;
use eframe::egui::epaint::Shadow;
use rand::Rng;

#[derive(Default)]
struct RollRecord {
    d4: Vec<i32>,
    d6: Vec<i32>,
    d8: Vec<i32>,
    d20: Vec<i32>,
    d100: Vec<i32>,
    constant: i32,

    total: i32
}

#[derive(Default)]
struct DicesState{
    d4: i32,
    d6: i32,
    d8: i32,
    d20: i32,
    d100: i32,
    constant: i32
}

impl DicesState {
    pub fn new(dice_num:[i32;5], constant: i32) -> DicesState{
        dice_num.iter().for_each(|n|{debug_assert!(*n >= 0)});
        DicesState{
            d4: dice_num[0],
            d6: dice_num[1],
            d8: dice_num[2],
            d20: dice_num[3],
            d100: dice_num[4],
            constant
        }
    }

    pub fn roll(&self,rd : &mut rand::rngs::ThreadRng) -> Rc<RollRecord> {
        let mut result = RollRecord::default();
        result.d4.reserve_exact(self.d4 as usize);
        for _ in 0..self.d4{
            result.d4.push(rd.gen_range(0..=4));
        }
        for _ in 0..self.d6{
            result.d6.push(rd.gen_range(0..=6));
        }
        for _ in 0..self.d8{
            result.d8.push(rd.gen_range(0..=8));
        }
        for _ in 0..self.d20{
            result.d20.push(rd.gen_range(0..=20));
        }
        for _ in 0..self.d100{
            result.d100.push(rd.gen_range(0..=100));
        }
        Rc::new(result)
    }
}

const RECORD_MAX_NUM: usize = 32;

#[derive(Default)]
struct RecordManager{
    table: std::collections::VecDeque<Rc<RollRecord>>,
    detail_windows: Vec<Rc<RollRecord>>
}

impl RecordManager {
    pub fn add_record(&mut self,record: Rc<RollRecord>){
        if self.table.len() >= RECORD_MAX_NUM{
            self.table.pop_front();
        }
        self.table.push_back(record);
    }

    pub fn update(&mut self, ctx: &egui::CtxRef){

    }
}

pub struct DiceFeature {
    state: DicesState,
    records: RecordManager,

    rd: RefCell<rand::rngs::ThreadRng>,

    window_open_state: bool
}

impl DiceFeature {
    pub fn update(&mut self, ctx: &egui::CtxRef) {

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                //ui.text_edit_singleline(label);
            });

            //ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));

            if ui.button("Roll").clicked() {
                self.records.add_record(self.state.roll(&mut self.rd.borrow_mut()));
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        self.records.update(ctx);


        egui::Window::new("Window")
            .frame(egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(255,255,255,255))
                .shadow(Shadow::big_light())
            )
            .resizable(true)
            .hscroll(true)
            .open(&mut self.window_open_state)
            .show(ctx, |ui| {
            ui.label("Windows can be moved by dragging them.");
            ui.label("They are automatically sized based on contents.");
            ui.label("You can turn on resizing and scrolling if you like.");
            ui.label("You would normally chose either panels OR windows.");
        });
    }
}

impl Default for DiceFeature {
    fn default() -> Self {
        DiceFeature {
            state: DicesState::default(),
            records: RecordManager::default(),
            rd: RefCell::new(rand::thread_rng()),
            window_open_state: true
        }
    }
}
