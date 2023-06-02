use eframe::{
    egui::{Sense, TextStyle, Ui},
    emath::Align2,
    epaint::{Color32, Pos2, Rect, Shape, Stroke},
};

use crate::data::graph::GraphViewerData;

pub struct GraphViewer {
    data: GraphViewerData,
}

impl GraphViewer {
    pub fn new() -> Self {
        Self {
            data: GraphViewerData::new(),
        }
    }

    pub fn update(&mut self, value: Option<f32>) {
        self.data.update(value);
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        max_value: f32,
        value_to_string: impl 'static + Fn(f32) -> String,
    ) {
        let mut highest_index = 0;
        render_graph_body(
            ui,
            |i| {
                highest_index = highest_index.max(i);
                self.data.get_value_at(i)
            },
            max_value,
            value_to_string,
        );

        self.data.trim_length(highest_index)
    }
}

pub fn render_graph_body(
    ui: &mut Ui,
    mut get_value_at: impl FnMut(usize) -> Option<f32>,
    max_value: f32,
    value_to_string: impl Fn(f32) -> String,
) {
    let available_space = ui.available_size();
    let (rect, _) = ui.allocate_exact_size(available_space, Sense::hover());
    ui.set_clip_rect(rect);

    let style = ui.style();

    let bg = style.visuals.extreme_bg_color;
    let line_col = style.visuals.widgets.active.bg_fill;
    let line_col_translucent =
        Color32::from_rgba_unmultiplied(line_col.r(), line_col.g(), line_col.b(), 30);

    ui.painter()
        .rect_filled(rect, style.visuals.window_rounding, bg);

    let edge_padding = 10.5;
    let text_width = 50.0;
    let text_padding = 5.0;
    let rect = Rect {
        min: Pos2::new(
            rect.left() + edge_padding + text_width,
            rect.top() + edge_padding,
        ),
        max: Pos2::new(
            rect.right() - edge_padding - text_width,
            rect.bottom() - edge_padding,
        ),
    };

    let font = &ui.style().text_styles[&TextStyle::Small];

    struct GuideLine {
        y: f32,
        text: String,
        should_draw_line: bool,
    }

    let make_guideline = |height: f32, should_draw_line: bool| {
        let y = rect.bottom() - (height * rect.height());
        let y = y.round() - 0.5;
        let text = value_to_string(height * max_value);
        GuideLine {
            y,
            text,
            should_draw_line,
        }
    };

    let lines = [
        make_guideline(0.0, true),
        make_guideline(0.25, true),
        make_guideline(0.5, true),
        make_guideline(0.75, true),
        make_guideline(1.0, true),
    ];
    for line in lines.iter() {
        if line.should_draw_line {
            ui.painter().line_segment(
                [
                    Pos2::new(rect.left(), line.y),
                    Pos2::new(rect.right(), line.y),
                ],
                (1.0, line_col_translucent),
            );
        }

        ui.painter().text(
            Pos2::new(rect.left() - text_padding, line.y),
            Align2::RIGHT_CENTER,
            &line.text,
            font.clone(),
            Color32::WHITE,
        );
    }

    if let Some(value) = get_value_at(0) {
        let text = value_to_string(value);
        let value = value / max_value;
        ui.painter().text(
            Pos2::new(
                rect.right() + text_padding,
                rect.bottom() - (value * rect.height()),
            ),
            Align2::LEFT_CENTER,
            &text,
            font.clone(),
            Color32::WHITE,
        );
    }

    ui.painter().rect_stroke(rect, 0.0, (1.0, line_col));

    ui.set_clip_rect(rect);

    #[derive(Debug, Clone)]
    struct Point {
        position: Pos2,
    }

    let mut points = Vec::new();
    let mut curr_points = Vec::new();
    let width = rect.width() as usize + 1;
    for i in 0..width {
        if let Some(value) = get_value_at(i) {
            let value = value / max_value;
            curr_points.push(Point {
                position: Pos2 {
                    x: rect.right() - (i as f32),
                    y: rect.bottom() - (value * rect.height()),
                },
            });
        } else {
            if !curr_points.is_empty() {
                points.push(curr_points);
                curr_points = Vec::new();
            }
        }
    }

    if !curr_points.is_empty() {
        points.push(curr_points);
    }

    let painter = ui.painter();
    for mut points in points {
        if points.len() == 1 {
            let first = points[0].clone();
            points.push(first);
        }

        let mut last = points[0].clone();

        for point in points.into_iter() {
            let shape = Shape::convex_polygon(
                vec![
                    last.position,
                    point.position,
                    Pos2::new(point.position.x, rect.bottom()),
                    Pos2::new(last.position.x, rect.bottom()),
                ],
                line_col_translucent,
                Stroke::NONE,
            );
            painter.add(shape);

            painter.line_segment([last.position, point.position], (2.0, line_col));

            last = point;
        }
    }
}
