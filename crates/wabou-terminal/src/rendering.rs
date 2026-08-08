use super::*;

pub(super) fn cell_text(square: Square, extra: Option<&Extras>) -> String {
    let mut text = String::from(square.c());
    if let Some(extra) = extra {
        text.extend(extra.zerowidth.iter().copied());
    }
    text
}

pub(super) fn selection_contains_square(
    selection: SelectionRange,
    point: Pos,
    square: Square,
) -> bool {
    if selection.contains(point) {
        return true;
    }

    match square.wide() {
        Wide::Wide => selection.contains(Pos::new(point.row, point.col + 1)),
        Wide::Spacer if point.col.0 > 0 => {
            selection.contains(Pos::new(point.row, Column(point.col.0 - 1)))
        }
        _ => false,
    }
}

pub(super) fn cursor_visual(
    focused: bool,
    cursor_on: bool,
    shape: CursorShape,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<CursorVisual> {
    if shape == CursorShape::Hidden || (focused && !cursor_on) {
        return None;
    }
    if !focused {
        return Some(CursorVisual::Hollow(Rect::new(
            x + 0.5,
            y + 0.5,
            x + width - 0.5,
            y + height - 0.5,
        )));
    }

    let rect = match shape {
        CursorShape::Beam => Rect::new(x, y, x + 2.0, y + height),
        CursorShape::Underline => Rect::new(x, y + height - 2.0, x + width, y + height),
        CursorShape::Block => Rect::new(x, y, x + width, y + height),
        CursorShape::Hidden => unreachable!(),
    };
    Some(CursorVisual::Filled(rect))
}

pub(super) fn fill_cell(
    scene: &mut Scene,
    column: usize,
    row: usize,
    cell_width: f32,
    line_height: f32,
    device_scale: f64,
    color: Color,
) {
    let rect = cell_fill_rect(column, row, cell_width, line_height, device_scale);
    scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
}

pub(super) fn cell_fill_rect(
    column: usize,
    row: usize,
    cell_width: f32,
    line_height: f32,
    device_scale: f64,
) -> Rect {
    let scale = device_scale.max(f64::EPSILON);
    let snap = |logical: f64| (logical * scale).round() / scale;
    let width = f64::from(cell_width);
    let height = f64::from(line_height);
    Rect::new(
        snap(column as f64 * width),
        snap(row as f64 * height),
        snap((column + 1) as f64 * width),
        snap((row + 1) as f64 * height),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_cell_decorations(
    scene: &mut Scene,
    column: usize,
    y: f32,
    cell_width: f32,
    line_height: f32,
    style: rio_vt::crosswords::style::Style,
    foreground: Color,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
    decoration_override: Option<Color>,
) {
    let (strike_color, underline_color) = decoration_colors(
        style,
        foreground,
        colors,
        theme_foreground,
        theme_background,
        decoration_override,
    );
    let x = column as f32 * cell_width;
    let fill = |scene: &mut Scene, color: Color, x0: f32, y0: f32, x1: f32, y1: f32| {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            color,
            None,
            &Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64),
        );
    };
    if style.flags.contains(StyleFlags::STRIKEOUT) {
        let strike_y = y + line_height * 0.52;
        fill(
            scene,
            strike_color,
            x,
            strike_y,
            x + cell_width,
            strike_y + 1.0,
        );
    }
    let underline_y = y + line_height - 2.0;
    if style.flags.contains(StyleFlags::DOUBLE_UNDERLINE) {
        fill(
            scene,
            underline_color,
            x,
            underline_y - 2.0,
            x + cell_width,
            underline_y - 1.0,
        );
        fill(
            scene,
            underline_color,
            x,
            underline_y,
            x + cell_width,
            underline_y + 1.0,
        );
    } else if style.flags.contains(StyleFlags::DOTTED_UNDERLINE) {
        let mut dot_x = x;
        while dot_x < x + cell_width {
            fill(
                scene,
                underline_color,
                dot_x,
                underline_y,
                (dot_x + 1.0).min(x + cell_width),
                underline_y + 1.0,
            );
            dot_x += 2.0;
        }
    } else if style.flags.contains(StyleFlags::DASHED_UNDERLINE) {
        let mut dash_x = x;
        while dash_x < x + cell_width {
            fill(
                scene,
                underline_color,
                dash_x,
                underline_y,
                (dash_x + 3.0).min(x + cell_width),
                underline_y + 1.0,
            );
            dash_x += 5.0;
        }
    } else if style.flags.contains(StyleFlags::UNDERCURL) {
        let mut curl_x = x;
        let mut high = true;
        while curl_x < x + cell_width {
            let curl_y = underline_y - if high { 1.0 } else { 0.0 };
            fill(
                scene,
                underline_color,
                curl_x,
                curl_y,
                (curl_x + 2.0).min(x + cell_width),
                curl_y + 1.0,
            );
            curl_x += 2.0;
            high = !high;
        }
    } else if style.flags.contains(StyleFlags::UNDERLINE) {
        fill(
            scene,
            underline_color,
            x,
            underline_y,
            x + cell_width,
            underline_y + 1.0,
        );
    }
}

pub(super) fn decoration_colors(
    style: Style,
    foreground: Color,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
    decoration_override: Option<Color>,
) -> (Color, Color) {
    let strike = decoration_override.unwrap_or(foreground);
    let underline = decoration_override
        .or_else(|| {
            style.underline_color.map(|color| {
                terminal_ansi_color(color, true, colors, theme_foreground, theme_background)
            })
        })
        .unwrap_or(foreground);
    (strike, underline)
}

pub(super) fn ansi_color(color: AnsiColor, foreground: bool) -> Color {
    match color {
        AnsiColor::Spec(ColorRgb { r, g, b }) => Color::from_rgb8(r, g, b),
        AnsiColor::Indexed(index) => indexed_color(index),
        AnsiColor::Named(name) => named_color(name, foreground),
    }
}

pub(super) fn terminal_ansi_color(
    color: AnsiColor,
    foreground: bool,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
) -> Color {
    let override_index = match color {
        AnsiColor::Indexed(index) => Some(index as usize),
        AnsiColor::Named(name) => Some(name as usize),
        AnsiColor::Spec(_) => None,
    };
    override_index
        .and_then(|index| colors[index])
        .map(color_from_array)
        .unwrap_or_else(|| match color {
            AnsiColor::Named(
                NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground,
            ) => theme_foreground,
            AnsiColor::Named(NamedColor::Background) => theme_background,
            _ => ansi_color(color, foreground),
        })
}

pub(super) fn terminal_indexed_color(index: u8, colors: &TermColors) -> Color {
    colors[index as usize]
        .map(color_from_array)
        .unwrap_or_else(|| indexed_color(index))
}

pub(super) fn color_from_array([r, g, b, a]: [f32; 4]) -> Color {
    Color::from_rgba8(
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub(super) fn named_color(color: NamedColor, foreground: bool) -> Color {
    let fallback = if foreground { 0xe2e8f0 } else { 0x0f172a };
    let rgb = match color {
        NamedColor::Black | NamedColor::DimBlack => 0x1e293b,
        NamedColor::Red | NamedColor::DimRed => 0xef4444,
        NamedColor::Green | NamedColor::DimGreen => 0x22c55e,
        NamedColor::Yellow | NamedColor::DimYellow => 0xeab308,
        NamedColor::Blue | NamedColor::DimBlue => 0x3b82f6,
        NamedColor::Magenta | NamedColor::DimMagenta => 0xd946ef,
        NamedColor::Cyan | NamedColor::DimCyan => 0x06b6d4,
        NamedColor::White | NamedColor::DimWhite => 0xcbd5e1,
        NamedColor::LightBlack => 0x64748b,
        NamedColor::LightRed => 0xf87171,
        NamedColor::LightGreen => 0x4ade80,
        NamedColor::LightYellow => 0xfacc15,
        NamedColor::LightBlue => 0x60a5fa,
        NamedColor::LightMagenta => 0xe879f9,
        NamedColor::LightCyan => 0x22d3ee,
        NamedColor::LightWhite => 0xf8fafc,
        NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground => {
            fallback
        }
        NamedColor::Background => 0x0f172a,
        NamedColor::Cursor => 0xe2e8f0,
    };
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub(super) fn indexed_color(index: u8) -> Color {
    if index < 16 {
        const TABLE: [u32; 16] = [
            0x1e293b, 0xef4444, 0x22c55e, 0xeab308, 0x3b82f6, 0xd946ef, 0x06b6d4, 0xcbd5e1,
            0x64748b, 0xf87171, 0x4ade80, 0xfacc15, 0x60a5fa, 0xe879f9, 0x22d3ee, 0xf8fafc,
        ];
        let rgb = TABLE[index as usize];
        return Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    }
    if index < 232 {
        let value = index - 16;
        let component = |n: u8| if n == 0 { 0 } else { 55 + n * 40 };
        return Color::from_rgb8(
            component(value / 36),
            component((value / 6) % 6),
            component(value % 6),
        );
    }
    let gray = 8 + (index - 232) * 10;
    Color::from_rgb8(gray, gray, gray)
}

pub(super) fn terminal_color(
    index: usize,
    colors: &TermColors,
    theme_foreground: Color,
    theme_background: Color,
) -> ColorRgb {
    let color = if let Some(color) = (index < TERMINAL_COLOR_COUNT)
        .then(|| colors[index])
        .flatten()
    {
        color_from_array(color)
    } else {
        match index {
            index if index < 256 => indexed_color(index as u8),
            index
                if index == NamedColor::Foreground as usize
                    || index == NamedColor::LightForeground as usize
                    || index == NamedColor::DimForeground as usize =>
            {
                theme_foreground
            }
            index if index == NamedColor::Background as usize => theme_background,
            index if index == NamedColor::Cursor as usize => named_color(NamedColor::Cursor, true),
            _ => named_color(NamedColor::Foreground, true),
        }
    };
    let [r, g, b, _] = color.to_rgba8().to_u8_array();
    ColorRgb { r, g, b }
}

pub(super) fn dim(color: Color) -> Color {
    let [r, g, b, a] = color.to_rgba8().to_u8_array();
    Color::from_rgba8(
        (r as f32 * 0.66) as u8,
        (g as f32 * 0.66) as u8,
        (b as f32 * 0.66) as u8,
        a,
    )
}
