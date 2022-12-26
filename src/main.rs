use std::time::Instant;

use eframe::{
    egui::{self},
    epaint::Vec2,
};

use nvml_wrapper::{Device, Nvml};
use panels::{graph::GraphViewer, process_table::ProcessTable};

use style::make_style;

mod panels;
mod processes;
mod style;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "gpustat",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    nvml: Nvml,
    monitor: GpuDeviceMonitor,
    updated_style: bool,
}

struct GpuDeviceMonitor {
    device_uuid: String,
    last_graph_update: Option<Instant>,
    usage_graph: GraphViewer,
    memory_graph: GraphViewer,
    processes: ProcessTable,
}

impl GpuDeviceMonitor {
    pub fn new(device: &Device) -> Self {
        Self {
            device_uuid: device.uuid().unwrap(),
            last_graph_update: None,
            usage_graph: GraphViewer::new(),
            memory_graph: GraphViewer::new(),
            processes: ProcessTable::new(Box::new(|device| device.running_graphics_processes())),
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
        }

        self.processes.update(device);
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let nvml = Nvml::init().unwrap();

        let monitor = GpuDeviceMonitor::new(&nvml.device_by_index(0).unwrap());

        Self {
            nvml,
            updated_style: false,
            monitor,
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.updated_style {
            ctx.set_style(make_style());
            self.updated_style = true;
        }

        let uuid: &str = &self.monitor.device_uuid;
        let device = self.nvml.device_by_uuid(uuid).unwrap();
        self.monitor.update(&device);

        egui::TopBottomPanel::top("top")
            .exact_height(300.0)
            .show(ctx, |ui| {
                let width = ui.available_width();

                ui.label("GPU Usage %");

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    self.monitor
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

                    self.monitor
                        .memory_graph
                        .render(ui, max_ram, |v| bytes_to_mib_gib(v));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.monitor.processes.ui(ui);
            ctx.request_repaint();
        });
    }
}
