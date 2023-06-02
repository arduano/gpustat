use std::time::Instant;

use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};

use self::{graph::GraphViewerData, process_table::ProcessTableData};

pub mod graph;
pub mod process_table;

pub struct GpuMonitoringData {
    nvml: Nvml,
    monitors: Vec<GpuDeviceMonitor>,
}

impl GpuMonitoringData {
    pub fn new() -> Self {
        let nvml = Nvml::init().unwrap();

        let gpu_count = nvml.device_count().unwrap();

        let monitors = (0..gpu_count)
            .map(|i| {
                let device = nvml.device_by_index(i).unwrap();
                GpuDeviceMonitor::new(&device)
            })
            .collect();

        Self { nvml, monitors }
    }

    pub fn update(&mut self) {
        for monitor in self.monitors.iter_mut() {
            let uuid = monitor.device_uuid();
            let device = self.nvml.device_by_uuid(uuid).unwrap();
            monitor.update(&device);
        }
    }

    pub fn gpus(&mut self) -> &mut [GpuDeviceMonitor] {
        &mut self.monitors
    }
}

pub struct GpuDeviceMonitor {
    device_uuid: String,
    device_name: String,
    last_graph_update: Option<Instant>,
    usage_graph: GraphViewerData,
    memory_graph: GraphViewerData,
    temperature_graph: GraphViewerData,

    graphics_processes: ProcessTableData,
    compute_processes: ProcessTableData,

    max_memory: u64,
}

impl GpuDeviceMonitor {
    pub fn new(device: &Device) -> Self {
        Self {
            device_uuid: device.uuid().unwrap(),
            device_name: device.name().unwrap(),
            last_graph_update: None,
            usage_graph: GraphViewerData::new(),
            memory_graph: GraphViewerData::new(),
            temperature_graph: GraphViewerData::new(),

            graphics_processes: ProcessTableData::new(Box::new(|device| {
                device.running_graphics_processes()
            })),
            compute_processes: ProcessTableData::new(Box::new(|device| {
                device.running_compute_processes()
            })),

            max_memory: device.memory_info().unwrap().total,
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

    pub fn memory_graph_mut(&mut self) -> &mut GraphViewerData {
        &mut self.memory_graph
    }

    pub fn usage_graph_mut(&mut self) -> &mut GraphViewerData {
        &mut self.usage_graph
    }

    pub fn temperature_graph_mut(&mut self) -> &mut GraphViewerData {
        &mut self.temperature_graph
    }

    pub fn graphics_processes_mut(&mut self) -> &mut ProcessTableData {
        &mut self.graphics_processes
    }

    pub fn compute_processes_mut(&mut self) -> &mut ProcessTableData {
        &mut self.compute_processes
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn device_uuid(&self) -> &str {
        &self.device_uuid
    }

    pub fn max_memory(&self) -> u64 {
        self.max_memory
    }
}
