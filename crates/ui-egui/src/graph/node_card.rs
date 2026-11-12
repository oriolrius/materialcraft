use egui::{Color32, CornerRadius, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};

pub(crate) const WIDTH: f32 = 100.0;
pub(crate) const HEADER_HEIGHT: f32 = 18.0;
pub(crate) const FOOTER_HEIGHT: f32 = 14.0;
const PORT_GAP: f32 = 18.0;
const DOT_RADIUS: f32 = 7.0;
/// Smallest on-screen side of a dot's grab area, in points
const DOT_MIN_GRAB: f32 = 16.0;
const CARD_COLOR: Color32 = Color32::from_rgb(0x1e, 0x20, 0x24);
const WELL_COLOR: Color32 = Color32::from_rgb(0x14, 0x15, 0x18);

/// Body height of a node that shows no preview and has at most one port a side
const BARE_BODY_HEIGHT: f32 = 26.0;

pub(crate) fn bounds(position: [f32; 2], inputs: usize, outputs: usize, previews: bool, dot: bool) -> Rect {
    bounds_with_body(position, inputs, outputs, if previews { WIDTH } else { BARE_BODY_HEIGHT }, dot)
}

/// A card whose body is at least `body_height` tall and fits its ports
pub(crate) fn bounds_with_body(position: [f32; 2], inputs: usize, outputs: usize, body_height: f32, dot: bool) -> Rect {
    let position = pos2(position[0], position[1]);
    if dot {
        return Rect::from_center_size(position, vec2(DOT_RADIUS * 2.0, DOT_RADIUS * 2.0));
    }

    let ports = inputs.max(outputs) as f32 * PORT_GAP + 8.0;
    Rect::from_min_size(position, vec2(WIDTH, HEADER_HEIGHT + body_height.max(ports) + FOOTER_HEIGHT))
}

/// The on-screen area that picks up dot `bounds`: at least `DOT_MIN_GRAB` wide, so it stays easy to grab when zoomed out
pub(crate) fn dot_grab(bounds: Rect) -> Rect {
    Rect::from_center_size(bounds.center(), bounds.size().max(vec2(DOT_MIN_GRAB, DOT_MIN_GRAB)))
}

pub(crate) fn port(bounds: Rect, index: usize, count: usize, output: bool, dot: bool) -> Pos2 {
    let x = if output { bounds.right() } else { bounds.left() };
    if dot {
        return pos2(x, bounds.center().y);
    }

    let body_height = bounds.height() - HEADER_HEIGHT - FOOTER_HEIGHT;
    let first = bounds.top() + HEADER_HEIGHT + (body_height - count as f32 * PORT_GAP) * 0.5 + PORT_GAP * 0.5;
    pos2(x, first + index as f32 * PORT_GAP)
}

pub(crate) fn header(bounds: Rect, zoom: f32) -> Rect {
    Rect::from_min_size(bounds.min, vec2(bounds.width(), HEADER_HEIGHT * zoom))
}

pub(crate) fn body(bounds: Rect, zoom: f32) -> Rect {
    Rect::from_min_max(pos2(bounds.left(), bounds.top() + HEADER_HEIGHT * zoom), pos2(bounds.right(), bounds.bottom() - FOOTER_HEIGHT * zoom))
}

pub(crate) fn paint(painter: &Painter, bounds: Rect, zoom: f32, label: &str, color: Color32) {
    let radius = (5.0 * zoom).clamp(1.0, 6.0) as u8;
    let header = header(bounds, zoom);
    painter.rect_filled(bounds, radius, CARD_COLOR);
    painter.rect_filled(header, CornerRadius { nw: radius, ne: radius, sw: 0, se: 0 }, color);
    painter.line_segment([header.left_bottom(), header.right_bottom()], Stroke::new(1.0, Color32::from_black_alpha(110)));

    let font = FontId::proportional((11.0 * zoom).clamp(7.0, 16.0));
    let title = crate::widgets::elided(painter, label, font.size, Color32::WHITE, header.width() - 10.0 * zoom);
    painter.galley(header.left_center() + vec2(5.0 * zoom, -title.size().y * 0.5), title, Color32::WHITE);
}

/// The dark well a preview or a value sits in, inside `body`
pub(crate) fn paint_well(painter: &Painter, well: Rect, zoom: f32) {
    painter.rect_filled(well, (3.0 * zoom).clamp(1.0, 4.0) as u8, WELL_COLOR);
}

/// A short symbol or word centered in `body`, as large as its length allows
pub(crate) fn paint_symbol(painter: &Painter, body: Rect, zoom: f32, symbol: &str, color: Color32) {
    let size = match symbol.chars().count() {
        0..=2 => 24.0,
        3..=4 => 17.0,
        _ => 13.0,
    };
    let text = crate::widgets::elided(painter, symbol, size * zoom, color, body.width() - 12.0 * zoom);
    painter.galley(body.center() - text.size() * 0.5, text, color);
}

pub(crate) fn outline(painter: &Painter, bounds: Rect, zoom: f32, stroke: Stroke) {
    painter.rect_stroke(bounds, (5.0 * zoom).clamp(1.0, 6.0) as u8, stroke, StrokeKind::Outside);
}

pub(crate) fn paint_dot(painter: &Painter, bounds: Rect, color: Color32) {
    painter.circle(bounds.center(), bounds.width() * 0.5, Color32::from_gray(0x50), Stroke::new(1.5, color));
}
