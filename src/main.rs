use gui::run_gpu_app;

mod data;
mod gui;
mod processes;
mod tui;
mod utils;

fn main() {
    // tui::run_tui_app();
    run_gpu_app();
}
