use crate::executor::{Executor, QueryResult};
use eframe::{App, egui};
use egui::Color32;
use egui_extras;
use egui_extras::syntax_highlighting::CodeTheme;

pub struct Application {
    exe: Executor,
    query: String,
    result: Option<QueryResult>,
}

impl App for Application {
    fn update(&mut self, _ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(_ctx, |ui| {
            let max_rect = ui.max_rect();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(max_rect.width() * 0.5);
                    self.draw_code_editor(max_rect.height() - 20., ui);
                    ui.button("Query!").clicked().then(|| {
                        let result = self.exe.run(self.query.clone());
                        self.result = Some(result);
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    if let None = &self.result {
                        ui.label("No results yet.");
                    } else {
                        let result = self.result.as_ref().unwrap();
                        match result {
                            QueryResult::Rows(_rows) => {
                                todo!()
                            }
                            QueryResult::Success => {
                                ui.colored_label(Color32::GREEN, "Query executed successfully.");
                            }
                            QueryResult::Error(msg) => {
                                ui.colored_label(Color32::RED, format!("Error: {}", msg));
                            }
                        }
                    }
                });
            });
        });
    }
}

impl Application {
    pub fn new() -> Self {
        Self {
            exe: Executor::new(),
            query: String::new(),
            result: None,
        }
    }

    pub fn launch(self) {
        let options = eframe::NativeOptions::default();
        eframe::run_native("SQuirreL GUI", options, Box::new(|_cc| Ok(Box::new(self))));
    }

    fn draw_code_editor(&mut self, height: f32, ui: &mut egui::Ui) {
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &CodeTheme::dark(20.0),
                buf.as_str(),
                "SQL",
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(layout_job))
        };
        egui::ScrollArea::vertical()
            .min_scrolled_height(height)
            .show(ui, |ui| {
                ui.take_available_height();
                let editor = egui::TextEdit::multiline(&mut self.query)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(999)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter);
                ui.add(editor);
            });
    }
}
