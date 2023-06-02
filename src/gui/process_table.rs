use eframe::{
    egui::{self, Layout, Sense},
    emath::Align,
    epaint::Rect,
};
use egui_extras::{Column, TableBuilder};
use nvml_wrapper::enums::device::UsedGpuMemory;

use crate::data::process_table::{
    ProcessTableData, ProcessTableSorting, SortingDirection, TableColumn,
};

pub fn render_process_table(ui: &mut egui::Ui, data: &mut ProcessTableData) {
    if let Err(err) = &data.processes() {
        ui.label("Failed to fetch process list");
        ui.label(format!("Error: {}", err));
        return;
    }

    let old_spacing = ui.style_mut().spacing.item_spacing.x;
    ui.style_mut().spacing.item_spacing.x = 0.0;

    let table = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(Column::initial(200.0).clip(true).resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header
                .col(|ui| table_column_head(ui, TableColumn::Pid, "PID", &mut data.sorting_mut()));
            header.col(|ui| {
                table_column_head(ui, TableColumn::Name, "Name", &mut data.sorting_mut())
            });
            header.col(|ui| {
                table_column_head(
                    ui,
                    TableColumn::GpuMemory,
                    "GPU Memory",
                    &mut data.sorting_mut(),
                )
            });
        });

    table.body(|mut body| {
        let processes = data.get_processes_sorted();
        if let Ok(processes) = processes {
            for process in processes {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        draw_table_cell(ui, |ui| {
                            ui.label(process.info.pid.to_string());
                        });
                    });
                    row.col(|ui| {
                        draw_table_cell(ui, |ui| {
                            ui.label(&process.name);
                        });
                    });
                    row.col(|ui| {
                        draw_table_cell(ui, |ui| {
                            ui.label(format_used_gpu_memory(&process.info.used_gpu_memory));
                        });
                    });
                });
            }
        }
    });

    ui.style_mut().spacing.item_spacing.x = old_spacing;
}

fn make_cell_ui_in_cell_rect(
    ui: &mut egui::Ui,
    rect: Rect,
    draw_contents: impl FnOnce(&mut egui::Ui),
) {
    let mut child = ui.child_ui(rect, Layout::left_to_right(Align::Center));
    child.style_mut().spacing.item_spacing.x = 4.0;
    child.add_space(4.0);

    draw_contents(&mut child);
}

fn draw_table_cell(ui: &mut egui::Ui, draw_contents: impl FnOnce(&mut egui::Ui)) {
    let size = ui.available_size();
    let (rect, _) = ui.allocate_at_least(size, Sense::hover());
    make_cell_ui_in_cell_rect(ui, rect, draw_contents);
}

fn table_column_head(
    ui: &mut egui::Ui,
    column: TableColumn,
    text: &str,
    sorting: &mut ProcessTableSorting,
) {
    let current_column_sorting = if sorting.column == column {
        Some(sorting.direction)
    } else {
        None
    };

    let size = ui.available_size();
    let (rect, response) = ui.allocate_at_least(size, Sense::click());

    if response.clicked() {
        sorting.click(column);
    }

    let theme = *ui.style().interact(&response);

    if response.hovered() {
        ui.painter().rect_filled(rect, 0.0, theme.bg_fill);
    }

    make_cell_ui_in_cell_rect(ui, rect, |ui| {
        if response.hovered() {
            let style = ui.style_mut();
            style.visuals.override_text_color =
                Some(style.visuals.widgets.inactive.fg_stroke.color);
        }

        if let Some(sorting) = current_column_sorting {
            let icon = match sorting {
                SortingDirection::Ascending => "🔽",
                SortingDirection::Descending => "🔼",
            };
            ui.label(icon);
        }

        ui.label(text);
    });
}

fn format_used_gpu_memory(memory: &UsedGpuMemory) -> String {
    match memory {
        UsedGpuMemory::Unavailable => "Unavailable".to_string(),
        UsedGpuMemory::Used(bytes) => format!("{:.2} MiB", *bytes as f64 / 1024.0 / 1024.0),
    }
}
