//! Small immediate-mode HUD helpers.
//!
//! The full in-game HUD (resource strip, selection panel, minimap) lives in
//! `crate::game::interface`; these are the shared drawing primitives.

use macroquad::prelude::*;

use crate::rendering::palette;

/// Immediate-mode button: draws and reports whether it was clicked this frame.
pub fn ui_button(rect: Rect, label: &str, enabled: bool) -> bool {
    let mouse = Vec2::from(mouse_position());
    let hovered = rect.contains(mouse);
    let clicked = enabled && hovered && is_mouse_button_pressed(MouseButton::Left);

    let bg = if !enabled {
        Color::new(0.16, 0.16, 0.17, 0.85)
    } else if hovered {
        Color::new(0.30, 0.32, 0.28, 0.95)
    } else {
        Color::new(0.20, 0.21, 0.19, 0.92)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, palette::UI_BORDER);

    let color = if enabled {
        palette::TEXT_PRIMARY
    } else {
        palette::TEXT_SECONDARY
    };
    let dims = measure_text(label, None, 14, 1.0);
    draw_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + (rect.h + dims.height) / 2.0 - 2.0,
        14.0,
        color,
    );
    clicked
}

/// Two-line button: bold label plus a smaller detail line (cost strings).
pub fn ui_button_detail(rect: Rect, label: &str, detail: &str, enabled: bool) -> bool {
    let clicked = ui_button(rect, "", enabled);
    let color = if enabled {
        palette::TEXT_PRIMARY
    } else {
        palette::TEXT_SECONDARY
    };
    let dims = measure_text(label, None, 13, 1.0);
    draw_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h * 0.42,
        13.0,
        color,
    );
    let ddims = measure_text(detail, None, 11, 1.0);
    draw_text(
        detail,
        rect.x + (rect.w - ddims.width) / 2.0,
        rect.y + rect.h * 0.8,
        11.0,
        Color::new(0.75, 0.72, 0.55, 1.0),
    );
    clicked
}
