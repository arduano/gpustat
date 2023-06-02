use eframe::{
    egui::{
        style::{Spacing, WidgetVisuals, Widgets},
        Style,
        TextStyle::*,
        Visuals,
    },
    epaint::{Color32, FontFamily, FontId, Vec2},
};

fn color_lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv_t = 1.0 - t;
    Color32::from_rgb(
        (a.r() as f32 * inv_t + b.r() as f32 * t) as u8,
        (a.g() as f32 * inv_t + b.g() as f32 * t) as u8,
        (a.b() as f32 * inv_t + b.b() as f32 * t) as u8,
    )
}

pub fn make_style() -> Style {
    let lighten_color = Color32::from_rgb(215, 204, 204);

    let lighten_by = |color: Color32, amount: f32| color_lerp(color, lighten_color, amount);

    let primary = Color32::from_rgb(30, 164, 48);
    let primary_hover = Color32::from_rgb(24, 149, 48);
    let primary_interact = Color32::from_rgb(20, 130, 46);
    let primary_text = Color32::from_rgb(194, 255, 215);
    let bg = Color32::from_rgb(23, 18, 18);
    let bg_dark = Color32::from_rgb(16, 10, 10);

    Style {
        visuals: Visuals {
            widgets: Widgets {
                inactive: WidgetVisuals {
                    rounding: 5.0.into(),
                    expansion: 0.0,
                    bg_fill: primary,
                    bg_stroke: (0.0, Color32::from_rgb(0, 0, 0)).into(),
                    fg_stroke: (1.0, primary_text).into(),
                },
                hovered: WidgetVisuals {
                    rounding: 5.0.into(),
                    expansion: 1.0,
                    bg_fill: primary_hover,
                    bg_stroke: (0.0, Color32::from_rgb(0, 0, 0)).into(),
                    fg_stroke: (1.0, primary_text).into(),
                },
                active: WidgetVisuals {
                    rounding: 5.0.into(),
                    expansion: 0.0,
                    bg_fill: primary_interact,
                    bg_stroke: (0.0, Color32::from_rgb(0, 0, 0)).into(),
                    fg_stroke: (1.0, primary_text).into(),
                },
                ..Default::default()
            },
            faint_bg_color: lighten_by(bg, 0.1),
            extreme_bg_color: bg_dark,
            panel_fill: bg,
            ..Default::default()
        },
        spacing: Spacing {
            button_padding: Vec2::new(8.0, 4.0),
            ..Default::default()
        },
        text_styles: [
            (Heading, FontId::new(30.0, FontFamily::Proportional)),
            (Body, FontId::new(14.0, FontFamily::Proportional)),
            (Monospace, FontId::new(14.0, FontFamily::Monospace)),
            (Button, FontId::new(14.0, FontFamily::Proportional)),
            (Small, FontId::new(10.0, FontFamily::Proportional)),
        ]
        .into(),
        ..Default::default()
    }
}
