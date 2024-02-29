use nvml_wrapper::struct_wrappers::device::ProcessInfo;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub struct ProcessDataBank {
    sys: sysinfo::System,
    last_refresh: std::time::Instant,
}

impl ProcessDataBank {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::new()),
            ),
            last_refresh: std::time::Instant::now(),
        }
    }

    fn get_process_name(&self, pid: u32) -> &str {
        let Some(process) = self.sys.process((pid as usize).into()) else {
            return "Unknown";
        };
        process.name()
    }

    pub fn map_process_list(&mut self, process_list: Vec<ProcessInfo>) -> Vec<ProcessData> {
        // Refresh the process data every 2 seconds
        if self.last_refresh.elapsed().as_secs() > 2 {
            self.sys.refresh_all();
            self.last_refresh = std::time::Instant::now();
        }

        let mut result = Vec::new();
        for process in process_list {
            // We skip system processes that often clog up the list without using any resources
            if process.pid == 0 {
                continue;
            }

            result.push(ProcessData {
                name: self.get_process_name(process.pid).to_string(),
                info: process,
            });
        }
        result
    }
}

pub struct ProcessData {
    pub info: ProcessInfo,
    pub name: String,
}
