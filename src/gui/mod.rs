use eframe::{
    egui::{self, IconData, Layout},
    emath::Align,
    epaint::Vec2,
};

use style::make_style;

use crate::{data::GpuMonitoringData, utils::bytes_to_mib_gib};

use self::{graph::render_graph, process_table::render_process_table};

mod graph;
mod process_table;
mod style;

const ICON: &[u8] = include_bytes!("../../assets/icon_64px.png");

fn load_icon(data: &[u8]) -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(data)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
enum SelectedProcessTab {
    #[default]
    All,
    Graphics,
    Compute,
}

pub fn run_gpu_app() {
    let options = eframe::NativeOptions {
        window_builder: Some(Box::new(|w| {
            w.with_icon(load_icon(ICON))
                .with_inner_size(egui::vec2(500.0, 720.0))
                .with_min_inner_size(egui::vec2(380.0, 600.0))
        })),
        ..Default::default()
    };

    eframe::run_native(
        "gpustat",
        options,
        Box::new(|_cc| Box::new(GpuApp::default())),
    )
    .unwrap();
}

pub struct GpuApp {
    data: GpuMonitoringData,

    selected_process_tab: SelectedProcessTab,
    updated_style: bool,

    selected_gpu: usize,
}

impl Default for GpuApp {
    fn default() -> Self {
        Self {
            data: GpuMonitoringData::new(),
            updated_style: false,
            selected_process_tab: Default::default(),
            selected_gpu: 0,
        }
    }
}

impl eframe::App for GpuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.updated_style {
            ctx.set_style(make_style());
            self.updated_style = true;
        }

        self.data.update();

        egui::TopBottomPanel::top("top")
            .exact_height(400.0)
            .show(ctx, |ui| {
                let width = ui.available_width();

                ui.add_space(6.0);

                let gpus = self.data.gpus();

                ui.allocate_ui_with_layout(
                    Vec2::new(width, 80.0),
                    Layout::right_to_left(Align::Min),
                    |ui| {
                        egui::ComboBox::from_label("Selected GPU")
                            .width(250.0)
                            .wrap(false)
                            .show_index(ui, &mut self.selected_gpu, gpus.len(), |i| {
                                gpus[i].device_name().to_string()
                            });
                    },
                );

                let monitor = &mut gpus[self.selected_gpu];

                ui.label("GPU Usage %");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    render_graph(ui, monitor.usage_graph_mut(), 100.0, |v| {
                        format!("{:.0}%", v)
                    })
                });

                ui.add_space(5.0);

                ui.label("VRAM Usage");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    let max_memory = monitor.max_memory();
                    render_graph(ui, monitor.memory_graph_mut(), max_memory as f32, |v| {
                        bytes_to_mib_gib(v)
                    });
                });

                ui.add_space(5.0);

                ui.label("GPU Temperature");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    render_graph(ui, monitor.temperature_graph_mut(), 100.0, |v| {
                        format!("{:.0}°C", v)
                    });
                });

                ui.add_space(6.0);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();
            ui.allocate_ui_with_layout(
                Vec2::new(width, 20.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.selectable_value(
                        &mut self.selected_process_tab,
                        SelectedProcessTab::All,
                        "All",
                    );
                    ui.selectable_value(
                        &mut self.selected_process_tab,
                        SelectedProcessTab::Graphics,
                        "Graphics",
                    );
                    ui.selectable_value(
                        &mut self.selected_process_tab,
                        SelectedProcessTab::Compute,
                        "Compute",
                    );
                },
            );

            let monitor = &mut self.data.gpus()[self.selected_gpu];

            match self.selected_process_tab {
                SelectedProcessTab::All => {
                    let mut ui = ui.child_ui_with_id_source(
                        ui.available_rect_before_wrap(),
                        Layout::top_down(Align::Min),
                        "all",
                    );

                    render_process_table(&mut ui, monitor.all_processes_mut());
                }
                SelectedProcessTab::Graphics => {
                    let mut ui = ui.child_ui_with_id_source(
                        ui.available_rect_before_wrap(),
                        Layout::top_down(Align::Min),
                        "graphics",
                    );

                    render_process_table(&mut ui, monitor.graphics_processes_mut());
                }
                SelectedProcessTab::Compute => {
                    let mut ui = ui.child_ui_with_id_source(
                        ui.available_rect_before_wrap(),
                        Layout::top_down(Align::Min),
                        "compute",
                    );

                    render_process_table(&mut ui, monitor.compute_processes_mut());
                }
            };
        });

        ctx.request_repaint();
    }
}
