//! Reusable UI widgets

use macroquad::prelude::*;
use crate::rendering::palette;

/// A clickable button
pub struct Button {
    pub text: String,
    pub rect: Rect,
    pub enabled: bool,
    hovered: bool,
    pressed: bool,
}

impl Button {
    pub fn new(text: &str, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            text: text.to_string(),
            rect: Rect::new(x, y, width, height),
            enabled: true,
            hovered: false,
            pressed: false,
        }
    }

    /// Update button state and return true if clicked
    pub fn update(&mut self) -> bool {
        let mouse_pos = Vec2::from(mouse_position());
        self.hovered = self.rect.contains(mouse_pos);

        if !self.enabled {
            self.pressed = false;
            return false;
        }

        if self.hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.pressed = true;
        }

        if self.pressed && is_mouse_button_released(MouseButton::Left) {
            self.pressed = false;
            return self.hovered;
        }

        false
    }

    /// Render the button
    pub fn render(&self) {
        let bg_color = if !self.enabled {
            Color::new(0.2, 0.2, 0.2, 0.8)
        } else if self.pressed {
            palette::HIGHLIGHT
        } else if self.hovered {
            Color::new(0.25, 0.25, 0.3, 0.9)
        } else {
            palette::UI_BACKGROUND
        };

        let text_color = if self.enabled {
            palette::TEXT_PRIMARY
        } else {
            palette::TEXT_SECONDARY
        };

        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, bg_color);
        draw_rectangle_lines(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            2.0,
            palette::UI_BORDER,
        );

        let text_dims = measure_text(&self.text, None, 20, 1.0);
        draw_text(
            &self.text,
            self.rect.x + (self.rect.w - text_dims.width) / 2.0,
            self.rect.y + (self.rect.h + text_dims.height) / 2.0 - 4.0,
            20.0,
            text_color,
        );
    }
}

/// A panel/container
pub struct Panel {
    pub rect: Rect,
    pub title: Option<String>,
}

impl Panel {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            title: None,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn render(&self) {
        // Background
        draw_rectangle(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            palette::UI_BACKGROUND,
        );

        // Border
        draw_rectangle_lines(
            self.rect.x,
            self.rect.y,
            self.rect.w,
            self.rect.h,
            2.0,
            palette::UI_BORDER,
        );

        // Title bar
        if let Some(title) = &self.title {
            draw_rectangle(
                self.rect.x,
                self.rect.y,
                self.rect.w,
                24.0,
                Color::new(0.15, 0.15, 0.2, 1.0),
            );
            draw_text(
                title,
                self.rect.x + 8.0,
                self.rect.y + 17.0,
                16.0,
                palette::TEXT_PRIMARY,
            );
        }
    }

    /// Get content area (excluding title bar)
    pub fn content_rect(&self) -> Rect {
        if self.title.is_some() {
            Rect::new(
                self.rect.x + 4.0,
                self.rect.y + 28.0,
                self.rect.w - 8.0,
                self.rect.h - 32.0,
            )
        } else {
            Rect::new(
                self.rect.x + 4.0,
                self.rect.y + 4.0,
                self.rect.w - 8.0,
                self.rect.h - 8.0,
            )
        }
    }
}

/// A tooltip that appears on hover
pub struct Tooltip {
    text: String,
    visible: bool,
}

impl Tooltip {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            visible: false,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn render(&self) {
        if !self.visible {
            return;
        }

        let (mx, my) = mouse_position();
        let padding = 8.0;
        let text_dims = measure_text(&self.text, None, 14, 1.0);
        let width = text_dims.width + padding * 2.0;
        let height = text_dims.height + padding * 2.0;

        // Position tooltip to avoid screen edges
        let x = (mx + 10.0).min(screen_width() - width);
        let y = (my + 10.0).min(screen_height() - height);

        draw_rectangle(x, y, width, height, Color::new(0.1, 0.1, 0.15, 0.95));
        draw_rectangle_lines(x, y, width, height, 1.0, palette::UI_BORDER);
        draw_text(
            &self.text,
            x + padding,
            y + padding + text_dims.height - 2.0,
            14.0,
            palette::TEXT_PRIMARY,
        );
    }
}
