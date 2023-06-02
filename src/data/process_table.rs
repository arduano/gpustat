use std::cmp::Ordering;

use eframe::{
    egui::{self, Layout, Sense},
    emath::Align,
    epaint::Rect,
};

use nvml_wrapper::{
    enums::device::UsedGpuMemory, error::NvmlError, struct_wrappers::device::ProcessInfo, Device,
};

use crate::processes::{ProcessData, ProcessDataBank};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TableColumn {
    Pid,
    Name,
    GpuMemory,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortingDirection {
    Ascending,
    Descending,
}

pub struct ProcessTableSorting {
    pub column: TableColumn,
    pub direction: SortingDirection,
}

impl ProcessTableSorting {
    pub fn click(&mut self, column: TableColumn) {
        if self.column == column {
            self.direction = match self.direction {
                SortingDirection::Ascending => SortingDirection::Descending,
                SortingDirection::Descending => SortingDirection::Ascending,
            };
        } else {
            self.column = column;
            self.direction = SortingDirection::Ascending;
        }
    }
}

pub struct ProcessTableData {
    sorting: ProcessTableSorting,
    processes: Result<Vec<ProcessData>, NvmlError>,
    last_refresh: Option<std::time::Instant>,
    process_bank: ProcessDataBank,
    fetcher: Box<dyn Fn(&Device) -> Result<Vec<ProcessInfo>, NvmlError>>,
}

impl ProcessTableData {
    pub fn new(fetcher: Box<dyn Fn(&Device) -> Result<Vec<ProcessInfo>, NvmlError>>) -> Self {
        Self {
            sorting: ProcessTableSorting {
                column: TableColumn::GpuMemory,
                direction: SortingDirection::Descending,
            },
            processes: Err(NvmlError::Unknown),
            last_refresh: None,
            process_bank: ProcessDataBank::new(),
            fetcher,
        }
    }

    fn fetch_last_process_array(&mut self, device: &Device) -> Result<Vec<ProcessData>, NvmlError> {
        let processes = (self.fetcher)(device)?;
        Ok(self.process_bank.map_process_list(processes))
    }

    pub fn update(&mut self, device: &Device) {
        // Refresh the process data every 1 second
        if self.last_refresh.is_none() || self.last_refresh.unwrap().elapsed().as_millis() > 1000 {
            self.processes = self.fetch_last_process_array(device);
            self.last_refresh = Some(std::time::Instant::now());
        }
    }

    fn get_processes_sorted_by(
        &self,
        by: impl Fn(&ProcessData, &ProcessData) -> Ordering,
        direction: SortingDirection,
    ) -> Result<Vec<&ProcessData>, &NvmlError> {
        let mut processes = self.processes.as_ref()?.iter().collect::<Vec<_>>();
        processes.sort_by(|a, b| {
            if direction == SortingDirection::Ascending {
                by(a, b)
            } else {
                by(b, a)
            }
        });
        Ok(processes)
    }

    pub fn get_processes_sorted(&self) -> Result<Vec<&ProcessData>, &NvmlError> {
        match self.sorting.column {
            TableColumn::Pid => self.get_processes_sorted_by(
                |a, b| a.info.pid.cmp(&b.info.pid),
                self.sorting.direction,
            ),
            TableColumn::Name => {
                self.get_processes_sorted_by(|a, b| a.name.cmp(&b.name), self.sorting.direction)
            }
            TableColumn::GpuMemory => self.get_processes_sorted_by(
                |a, b| match (&a.info.used_gpu_memory, &b.info.used_gpu_memory) {
                    (UsedGpuMemory::Used(a), UsedGpuMemory::Used(b)) => a.cmp(&b),
                    (UsedGpuMemory::Used(_), UsedGpuMemory::Unavailable) => Ordering::Less,
                    (UsedGpuMemory::Unavailable, UsedGpuMemory::Used(_)) => Ordering::Greater,
                    (UsedGpuMemory::Unavailable, UsedGpuMemory::Unavailable) => Ordering::Equal,
                },
                self.sorting.direction,
            ),
        }
    }

    pub fn sorting(&self) -> &ProcessTableSorting {
        &self.sorting
    }

    pub fn sorting_mut(&mut self) -> &mut ProcessTableSorting {
        &mut self.sorting
    }

    pub fn processes(&self) -> Result<Vec<&ProcessData>, &NvmlError> {
        self.get_processes_sorted()
    }
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
