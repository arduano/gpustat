use std::time::Instant;

use eframe::{
    egui::{self, Layout},
    emath::Align,
    epaint::Vec2,
};

use gui::run_gpu_app;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};
use panels::{graph::GraphViewer, process_table::ProcessTable};

mod gui;

mod data;
mod panels;

mod processes;

fn main() {
    run_gpu_app()
}
