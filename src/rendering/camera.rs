//! Camera system for panning and zooming

use macroquad::prelude::*;

/// Game camera for the map view
pub struct GameCamera {
    /// Camera position (center of view)
    pub position: Vec2,

    /// Zoom level (1.0 = normal)
    pub zoom: f32,

    /// Minimum zoom
    pub min_zoom: f32,

    /// Maximum zoom
    pub max_zoom: f32,

    /// Pan speed (pixels per second)
    pub pan_speed: f32,

    /// Zoom speed (multiplier per scroll)
    pub zoom_speed: f32,

    /// Is the camera being dragged?
    drag_start: Option<Vec2>,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.25,
            max_zoom: 4.0,
            pan_speed: 500.0,
            zoom_speed: 0.1,
            drag_start: None,
        }
    }
}

impl GameCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle camera input
    pub fn handle_input(&mut self, dt: f32) {
        // Keyboard panning
        let mut pan = Vec2::ZERO;

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            pan.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            pan.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            pan.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            pan.x += 1.0;
        }

        if pan != Vec2::ZERO {
            self.position += pan.normalize() * self.pan_speed * dt / self.zoom;
        }

        // Mouse drag panning
        if is_mouse_button_pressed(MouseButton::Middle)
            || (is_mouse_button_pressed(MouseButton::Left) && is_key_down(KeyCode::Space))
        {
            self.drag_start = Some(Vec2::from(mouse_position()));
        }

        if is_mouse_button_released(MouseButton::Middle)
            || is_mouse_button_released(MouseButton::Left)
        {
            self.drag_start = None;
        }

        if let Some(start) = self.drag_start {
            let current = Vec2::from(mouse_position());
            let delta = start - current;
            self.position += delta / self.zoom;
            self.drag_start = Some(current);
        }

        // Mouse wheel zoom
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            let zoom_delta = scroll_y * self.zoom_speed;
            self.zoom = (self.zoom + zoom_delta).clamp(self.min_zoom, self.max_zoom);
        }

        // Reset zoom with Home key
        if is_key_pressed(KeyCode::Home) {
            self.zoom = 1.0;
            self.position = Vec2::ZERO;
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let offset = (screen_pos - screen_center) / self.zoom;
        self.position + offset
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let offset = (world_pos - self.position) * self.zoom;
        screen_center + offset
    }

    /// Apply camera transform for rendering
    pub fn apply(&self) {
        // Set up the camera for macroquad
        let cam = Camera2D {
            target: self.position,
            zoom: vec2(
                self.zoom * 2.0 / screen_width(),
                self.zoom * 2.0 / screen_height(),
            ),
            ..Default::default()
        };
        set_camera(&cam);
    }

    /// Reset to default camera (for UI rendering)
    pub fn reset() {
        set_default_camera();
    }
}
