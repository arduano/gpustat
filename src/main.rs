use std::{collections::HashMap, time::Instant};

use eframe::{
    egui::{
        self,
        style::{Spacing, WidgetVisuals, Widgets},
        FontDefinitions, Layout, Sense, Style, TextStyle, Visuals,
    },
    emath::Align,
    epaint::{Color32, FontFamily, FontId, Pos2, Vec2},
};

use egui_extras::{Column, TableBuilder};
use nvml_wrapper::{
    enums::device::UsedGpuMemory,
    error::NvmlError,
    struct_wrappers::device::{MemoryInfo, ProcessInfo},
    Device, Nvml,
};
use panels::{
    graph::{render_graph_body, GraphViewer},
    process_table::ProcessTable,
};
use processes::{ProcessData, ProcessDataBank};
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
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct GpuData {
    uuid: String,
    persistent: PersistentGpuData,
    dynamic: DynamicGpuData,
}

struct PersistentGpuData {
    name: String,
}

struct DynamicGpuData {
    memory_info: Result<MemoryInfo, NvmlError>,
    graphics_processes: Result<Vec<ProcessInfo>, NvmlError>,
    compute_processes: Result<Vec<ProcessInfo>, NvmlError>,
}

struct MyApp {
    nvml: Nvml,
    table: ProcessTable,
    updated_style: bool,
    last_graph_update: Option<Instant>,
    usage_graph: GraphViewer,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            nvml: Nvml::init().unwrap(),
            table: ProcessTable::new(Box::new(|device| device.running_graphics_processes())),
            updated_style: false,

            last_graph_update: None,
            usage_graph: GraphViewer::new(|v| format!("{:.0}%", v * 100.0)),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.updated_style {
            ctx.set_style(make_style());
            self.updated_style = true;
        }

        let device = self.nvml.device_by_index(0).unwrap();

        // add usage percent

        if self.last_graph_update.is_none()
            || self.last_graph_update.unwrap().elapsed().as_millis() > 500
        {
            let percent = device.utilization_rates().unwrap().gpu as f32;
            self.usage_graph.update(percent / 100.0);

            self.last_graph_update = Some(Instant::now());
        }

        egui::TopBottomPanel::top("top")
            .exact_height(300.0)
            .show(ctx, |ui| {
                let width = ui.available_width();

                ui.allocate_ui(Vec2::new(width, 100.0), |ui| {
                    self.usage_graph.render(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.table.ui(ui, &device);
            ctx.request_repaint();
        });
    }
}

struct GpuDataView {
    persistent: PersistentGpuData,
}

fn fetch_persistent_device_info(device: &Device) -> PersistentGpuData {
    PersistentGpuData {
        name: device
            .name()
            .unwrap_or_else(|_| "<Failed to get GPU name>".to_string()),
    }
}

fn fetch_dynamic_device_info(device: &Device) -> DynamicGpuData {
    DynamicGpuData {
        memory_info: device.memory_info(),
        graphics_processes: device.running_graphics_processes(),
        compute_processes: device.running_compute_processes(),
    }
}
