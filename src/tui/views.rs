use nvml_wrapper::{enums::device::UsedGpuMemory, error::NvmlError};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Text,
    widgets::{
        block::Title, Axis, Block, Borders, Cell, Chart, Dataset, GraphType, HighlightSpacing, Row,
        Table, TableState,
    },
    Frame,
};

use crate::{data::GpuDeviceMonitor, processes::ProcessData, utils::bytes_to_mib_gib};

pub fn render_usage_chart(f: &mut Frame, area: Rect, gpu: &mut GpuDeviceMonitor) {
    let data = gpu.usage_graph_mut();

    // Padding of the graph border/axis
    let left_padding = 5;
    let right_padding = 1;

    let mut points = Vec::new();
    let length = (area.width - left_padding - right_padding) as usize * 2;
    for i in 0..length {
        let value = data.get_value_at(i);
        if let Some(value) = value {
            points.push(((length - i - 1) as f64, value as f64));
        }
    }

    let last_usage = gpu.usage_graph_mut().get_value_at(0).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&points)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content("GPU Usage".cyan().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([0.0, length as f64]),
        )
        .y_axis(
            Axis::default()
                .title(format!("Usage ({}%)", last_usage).bold())
                .style(Style::default().gray())
                .bounds([0.0, 100.0])
                .labels(vec!["0".bold(), "50".into(), "100".bold()]),
        );

    f.render_widget(chart, area)
}

pub fn render_memory_chart(f: &mut Frame, area: Rect, gpu: &mut GpuDeviceMonitor) {
    let data = gpu.memory_graph_mut();

    // Padding of the graph border/axis
    let left_padding = 10;
    let right_padding = 1;

    let mut points = Vec::new();
    let length = (area.width - left_padding - right_padding) as usize * 2;
    for i in 0..length {
        let value = data.get_value_at(i);
        if let Some(value) = value {
            points.push(((length - i - 1) as f64, value as f64));
        }
    }

    let last_memory = gpu.memory_graph_mut().get_value_at(0).unwrap_or(0.0);

    let max_memory = gpu.max_memory();

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&points)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content("Memory Usage".cyan().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([0.0, length as f64]),
        )
        .y_axis(
            Axis::default()
                .title(format!("Memory ({})", bytes_to_mib_gib(last_memory)).bold())
                .style(Style::default().gray())
                .bounds([0.0, max_memory as f64])
                .labels(vec![
                    "0".bold(),
                    format!("{}", bytes_to_mib_gib((max_memory / 2) as f32)).into(),
                    format!("{}", bytes_to_mib_gib(max_memory as f32)).bold(),
                ]),
        );

    f.render_widget(chart, area)
}

pub fn render_temperature_chart(f: &mut Frame, area: Rect, gpu: &mut GpuDeviceMonitor) {
    let data = gpu.temperature_graph_mut();

    // Padding of the graph border/axis
    let left_padding = 6;
    let right_padding = 1;

    let mut points = Vec::new();
    let length = (area.width - left_padding - right_padding) as usize * 2;
    for i in 0..length {
        let value = data.get_value_at(i);
        if let Some(value) = value {
            points.push(((length - i - 1) as f64, value as f64));
        }
    }

    let last_temp = gpu.temperature_graph_mut().get_value_at(0).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .graph_type(GraphType::Line)
        .data(&points)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(
                    Title::default()
                        .content("Temperature".cyan().bold())
                        .alignment(Alignment::Center),
                )
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().gray())
                .bounds([0.0, length as f64]),
        )
        .y_axis(
            Axis::default()
                .title(format!("Temperature ({}°)", last_temp).bold())
                .style(Style::default().gray())
                .bounds([0.0, 100.0])
                .labels(vec!["0".bold(), "50°".into(), "100°".bold()]),
        );

    f.render_widget(chart, area)
}

pub struct ProcessTableState {
    inner_state: TableState,
}

impl Default for ProcessTableState {
    fn default() -> Self {
        Self {
            inner_state: TableState::default(),
        }
    }
}

pub fn render_process_table(
    f: &mut Frame,
    area: Rect,
    processes: Result<Vec<&ProcessData>, &NvmlError>,
    state: &mut ProcessTableState,
) {
    state.inner_state.select(Some(1));

    let header_style = Style::default().fg(Color::Cyan).bold();
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(Color::LightGreen);

    let header = ["PID", "Process", "Memory"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style);

    let rows = if let Ok(processes) = processes {
        processes
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let memory_str = if let UsedGpuMemory::Used(memory) = data.info.used_gpu_memory {
                    bytes_to_mib_gib(memory as f32)
                } else {
                    "Unknown".to_string()
                };

                [
                    Cell::from(Text::from(format!("{}", data.info.pid))),
                    Cell::from(Text::from(format!("{}", data.name))),
                    Cell::from(Text::from(format!("{}", memory_str))),
                ]
                .into_iter()
                .collect::<Row>()
                .style(Style::new().fg(Color::Green))
            })
            .collect::<Vec<_>>()
    } else {
        let row = vec![Cell::from("Error fetching processes")]
            .into_iter()
            .collect::<Row>()
            .style(Style::new().fg(Color::Red).bg(Color::Black));

        vec![row]
    };

    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Min(40),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .highlight_spacing(HighlightSpacing::Always);

    f.render_stateful_widget(t, area, &mut state.inner_state);
}
