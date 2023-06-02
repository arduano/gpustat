use std::time::Instant;

use eframe::{
    egui::{self, Layout},
    emath::Align,
    epaint::Vec2,
};
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};

use style::make_style;

use self::{graph::GraphViewer, process_table::ProcessTable};

mod graph;
mod process_table;
mod style;

#[derive(Default, Debug, PartialEq, Eq)]
enum SelectedProcessTab {
    #[default]
    Graphics,
    Compute,
}

pub fn run_gpu_app() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 720.0)),
        min_window_size: Some(egui::vec2(380.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "gpustat",
        options,
        Box::new(|_cc| Box::new(GpuApp::default())),
    );
}

pub struct GpuApp {
    nvml: Nvml,
    selected_process_tab: SelectedProcessTab,
    monitors: Vec<GpuDeviceMonitor>,
    updated_style: bool,

    selected_gpu: usize,
}

struct GpuDeviceMonitor {
    device_uuid: String,
    device_name: String,
    last_graph_update: Option<Instant>,
    usage_graph: GraphViewer,
    memory_graph: GraphViewer,
    temperature_graph: GraphViewer,

    graphics_processes: ProcessTable,
    compute_processes: ProcessTable,
}

impl GpuDeviceMonitor {
    pub fn new(device: &Device) -> Self {
        Self {
            device_uuid: device.uuid().unwrap(),
            device_name: device.name().unwrap(),
            last_graph_update: None,
            usage_graph: GraphViewer::new(),
            memory_graph: GraphViewer::new(),
            temperature_graph: GraphViewer::new(),

            graphics_processes: ProcessTable::new(Box::new(|device| {
                device.running_graphics_processes()
            })),
            compute_processes: ProcessTable::new(Box::new(|device| {
                device.running_compute_processes()
            })),
        }
    }

    pub fn update(&mut self, device: &Device) {
        // Update the graphs every 500ms
        if self.last_graph_update.is_none()
            || self.last_graph_update.unwrap().elapsed().as_millis() > 500
        {
            self.last_graph_update = Some(Instant::now());

            let percent = device.utilization_rates().map(|r| r.gpu as f32).ok();
            self.usage_graph.update(percent);

            let used = device.memory_info().map(|m| m.used as f32).ok();
            self.memory_graph.update(used);

            let used = device
                .temperature(TemperatureSensor::Gpu)
                .map(|m| m as f32)
                .ok();
            self.temperature_graph.update(used);
        }

        self.graphics_processes.update(device);
        self.compute_processes.update(device);
    }
}

impl Default for GpuApp {
    fn default() -> Self {
        let nvml = Nvml::init().unwrap();

        let gpu_count = nvml.device_count().unwrap();

        let monitors = (0..gpu_count)
            .map(|i| {
                let device = nvml.device_by_index(i).unwrap();
                GpuDeviceMonitor::new(&device)
            })
            .collect();

        Self {
            nvml,
            updated_style: false,
            selected_process_tab: SelectedProcessTab::Graphics,
            monitors,
            selected_gpu: 0,
        }
    }
}

fn bytes_to_mib_gib(bytes: f32) -> String {
    if bytes > 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2}GiB", bytes / 1024.0 / 1024.0 / 1024.0)
    } else if bytes > 1024.0 * 1024.0 {
        format!("{:.2}MiB", bytes / 1024.0 / 1024.0)
    } else {
        format!("{:.2}KiB", bytes / 1024.0)
    }
}

impl eframe::App for GpuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.updated_style {
            ctx.set_style(make_style());
            self.updated_style = true;
        }

        for monitor in self.monitors.iter_mut() {
            let uuid: &str = &monitor.device_uuid;
            let device = self.nvml.device_by_uuid(uuid).unwrap();
            monitor.update(&device);
        }

        egui::TopBottomPanel::top("top")
            .exact_height(400.0)
            .show(ctx, |ui| {
                let width = ui.available_width();

                ui.add_space(6.0);

                ui.allocate_ui_with_layout(
                    Vec2::new(width, 80.0),
                    Layout::right_to_left(Align::Min),
                    |ui| {
                        egui::ComboBox::from_label("Selected GPU")
                            .width(250.0)
                            .wrap(false)
                            .show_index(ui, &mut self.selected_gpu, self.monitors.len(), |i| {
                                self.monitors[i].device_name.clone()
                            });
                    },
                );

                let monitor = &mut self.monitors[self.selected_gpu];
                let uuid: &str = &monitor.device_uuid;
                let device = self.nvml.device_by_uuid(uuid).unwrap();

                ui.label("GPU Usage %");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    monitor
                        .usage_graph
                        .render(ui, 100.0, |v| format!("{:.0}%", v));
                });

                ui.add_space(5.0);

                ui.label("VRAM Usage");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    let meminfo = device.memory_info();

                    let max_ram = match meminfo {
                        Ok(meminfo) => meminfo.total as f32,
                        Err(_) => 1.0,
                    };

                    monitor
                        .memory_graph
                        .render(ui, max_ram, |v| bytes_to_mib_gib(v));
                });

                ui.add_space(5.0);

                ui.label("GPU Temperature");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    monitor
                        .temperature_graph
                        .render(ui, 100.0, |v| format!("{:.0}°C", v));
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

            let monitor = &mut self.monitors[self.selected_gpu];

            match self.selected_process_tab {
                SelectedProcessTab::Compute => {
                    let mut ui = ui.child_ui_with_id_source(
                        ui.available_rect_before_wrap(),
                        Layout::top_down(Align::Min),
                        "compute",
                    );
                    monitor.compute_processes.ui(&mut ui);
                }
                SelectedProcessTab::Graphics => {
                    let mut ui = ui.child_ui_with_id_source(
                        ui.available_rect_before_wrap(),
                        Layout::top_down(Align::Min),
                        "graphics",
                    );
                    monitor.graphics_processes.ui(&mut ui);
                }
            };
        });

        ctx.request_repaint();
    }
}
