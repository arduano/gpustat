use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    symbols,
    widgets::{block::Title, Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use crate::{data::GpuDeviceMonitor, utils::bytes_to_mib_gib};

pub fn render_usage_chart(f: &mut Frame, area: Rect, gpu: &mut GpuDeviceMonitor) {
    let data = gpu.usage_graph_mut();

    let mut points = Vec::new();
    let length = (area.width - 6) as usize * 2;
    for i in 0..length {
        let value = data.get_value_at(i);
        if let Some(value) = value {
            points.push(((length - i - 1) as f64, value as f64));
        }
    }

    let last_usage = gpu.usage_graph_mut().get_value_at(0).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
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

    let mut points = Vec::new();
    let length = (area.width - 6) as usize * 2;
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
        .style(Style::default().fg(Color::Yellow))
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

    let mut points = Vec::new();
    let length = (area.width - 7) as usize * 2;
    for i in 0..length {
        let value = data.get_value_at(i);
        if let Some(value) = value {
            points.push(((length - i - 1) as f64, value as f64));
        }
    }

    let last_temp = gpu.temperature_graph_mut().get_value_at(0).unwrap_or(0.0);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
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
