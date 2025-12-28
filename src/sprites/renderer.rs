//! Sprite rendering based on AI-generated descriptions

use macroquad::prelude::*;
use super::{SpriteDescription, SpriteShape, SpritePattern, WarriorArchetype, HelmetStyle, WeaponType};

/// A rendered sprite with position
pub struct RenderedSprite {
    pub description: SpriteDescription,
    pub x: f32,
    pub y: f32,
    pub pixels: Vec<Vec<Option<[u8; 3]>>>,
}

impl RenderedSprite {
    /// Create a new rendered sprite from a description
    pub fn new(description: SpriteDescription, x: f32, y: f32) -> Self {
        let pixels = generate_pixels(&description);
        Self {
            description,
            x,
            y,
            pixels,
        }
    }

    /// Render this sprite
    pub fn render(&self) {
        let (w, h) = self.description.size;
        let scale = 3.0; // Scale up for visibility

        for (py, row) in self.pixels.iter().enumerate() {
            for (px, pixel) in row.iter().enumerate() {
                if let Some([r, g, b]) = pixel {
                    let color = Color::from_rgba(*r, *g, *b, 255);
                    draw_rectangle(
                        self.x + px as f32 * scale,
                        self.y + py as f32 * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }

        // Draw name below sprite
        let name_size = 10.0;
        draw_text(
            &self.description.name,
            self.x,
            self.y + h as f32 * scale + name_size + 2.0,
            name_size,
            WHITE,
        );
    }
}

/// Generate pixel data for a sprite
fn generate_pixels(desc: &SpriteDescription) -> Vec<Vec<Option<[u8; 3]>>> {
    let (w, h) = desc.size;
    let mut pixels = vec![vec![None; w as usize]; h as usize];

    match desc.shape {
        SpriteShape::Blob => generate_blob(&mut pixels, desc),
        SpriteShape::Humanoid => generate_humanoid(&mut pixels, desc),
        SpriteShape::Quadruped => generate_quadruped(&mut pixels, desc),
        SpriteShape::Flying => generate_flying(&mut pixels, desc),
        SpriteShape::Serpent => generate_serpent(&mut pixels, desc),
        SpriteShape::Geometric => generate_geometric(&mut pixels, desc),
    }

    // Apply pattern
    apply_pattern(&mut pixels, desc);

    pixels
}

/// Generate a blob-shaped sprite
fn generate_blob(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let rx = w as f32 / 2.0 - 1.0;
    let ry = h as f32 / 2.0 - 1.0;

    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as f32 - cx) / rx;
            let dy = (y as f32 - cy) / ry;
            if dx * dx + dy * dy <= 1.0 {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Add eyes
    if w >= 12 && h >= 12 {
        let eye_y = (h / 3) as usize;
        let eye_x1 = (w / 3) as usize;
        let eye_x2 = (2 * w / 3) as usize;
        if eye_y < h as usize && eye_x1 < w as usize && eye_x2 < w as usize {
            pixels[eye_y][eye_x1] = Some(desc.accent_color);
            pixels[eye_y][eye_x2] = Some(desc.accent_color);
        }
    }
}

/// Generate a highly detailed humanoid warrior sprite
fn generate_humanoid(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let w = w as usize;
    let h = h as usize;
    let cx = w / 2;

    // Body proportions based on archetype
    let (body_width, shoulder_width, head_height, leg_thickness) = match desc.archetype {
        WarriorArchetype::Tiny => (w * 30 / 100, w * 35 / 100, h * 28 / 100, w * 8 / 100),   // Goblins, halflings - big head, small body
        WarriorArchetype::Light => (w * 35 / 100, w * 45 / 100, h * 20 / 100, w * 10 / 100), // Scouts, rogues - slim
        WarriorArchetype::Medium => (w * 40 / 100, w * 55 / 100, h * 22 / 100, w * 12 / 100), // Infantry - normal
        WarriorArchetype::Heavy => (w * 50 / 100, w * 70 / 100, h * 24 / 100, w * 15 / 100), // Knights - broad
        WarriorArchetype::Giant => (w * 45 / 100, w * 60 / 100, h * 18 / 100, w * 14 / 100), // Berserkers - tall, small head
        WarriorArchetype::Hulk => (w * 55 / 100, w * 80 / 100, h * 18 / 100, w * 18 / 100),  // Demons, mechs - massive
    };

    // Rich color palette
    let armor = desc.primary_color;
    let armor_highlight = lighten(armor, 1.5);
    let armor_light = lighten(armor, 1.25);
    let armor_mid = armor;
    let armor_dark = darken(armor, 0.7);
    let armor_shadow = darken(armor, 0.5);

    let skin = desc.secondary_color;
    let skin_light = lighten(skin, 1.2);
    let skin_dark = darken(skin, 0.75);
    let skin_shadow = darken(skin, 0.55);

    let metal = desc.accent_color;
    let metal_shine = lighten(metal, 1.6);
    let metal_light = lighten(metal, 1.3);
    let metal_dark = darken(metal, 0.65);
    let metal_shadow = darken(metal, 0.4);

    let black = [10, 10, 15];
    let outline = [25, 22, 30];
    let dark_outline = [15, 12, 18];

    // Key positions
    let neck_y = head_height;
    let shoulder_y = neck_y + 2;
    let chest_top = shoulder_y + 3;
    let waist_y = h * 55 / 100;
    let hip_y = h * 60 / 100;
    let knee_y = h * 78 / 100;
    let ankle_y = h * 92 / 100;

    // === CAPE (drawn first, behind everything) ===
    if desc.has_cape {
        draw_detailed_cape(pixels, cx, w, h, shoulder_y, hip_y, armor, armor_dark, armor_shadow);
    }

    // === LEGS WITH ARMOR ===
    let leg_gap = w * 6 / 100;
    let left_leg_cx = cx - leg_gap - leg_thickness / 2;
    let right_leg_cx = cx + leg_gap + leg_thickness / 2;

    for y in hip_y..h {
        let progress = (y - hip_y) as f32 / (h - hip_y) as f32;
        let leg_w = leg_thickness + 2 - (progress * 2.0) as usize;

        // Left leg
        for lx in 0..leg_w {
            let x = left_leg_cx.saturating_sub(leg_w / 2) + lx;
            if x < w && y < h {
                let color = if lx == 0 {
                    outline
                } else if lx == leg_w - 1 {
                    armor_shadow
                } else if lx < leg_w / 3 {
                    armor_light
                } else if lx < leg_w * 2 / 3 {
                    armor_mid
                } else {
                    armor_dark
                };
                pixels[y][x] = Some(color);
            }
        }

        // Right leg
        for lx in 0..leg_w {
            let x = right_leg_cx.saturating_sub(leg_w / 2) + lx;
            if x < w && y < h {
                let color = if lx == 0 {
                    armor_light
                } else if lx == leg_w - 1 {
                    outline
                } else if lx < leg_w / 3 {
                    armor_light
                } else if lx < leg_w * 2 / 3 {
                    armor_mid
                } else {
                    armor_dark
                };
                pixels[y][x] = Some(color);
            }
        }

        // Knee armor plates
        if y >= knee_y - 2 && y <= knee_y + 2 {
            let knee_w = leg_w + 2;
            for leg_cx in [left_leg_cx, right_leg_cx] {
                for kx in 0..knee_w {
                    let x = leg_cx.saturating_sub(knee_w / 2) + kx;
                    if x < w && y < h {
                        pixels[y][x] = Some(if kx < knee_w / 2 { metal_light } else { metal_dark });
                    }
                }
            }
        }
    }

    // === BOOTS ===
    for y in ankle_y..h {
        let boot_w = leg_thickness + 4;
        for leg_cx in [left_leg_cx, right_leg_cx] {
            for bx in 0..boot_w {
                let x = leg_cx.saturating_sub(boot_w / 2) + bx;
                if x < w && y < h {
                    let color = if y == h - 1 {
                        dark_outline
                    } else if bx == 0 || bx == boot_w - 1 {
                        outline
                    } else if bx < boot_w / 3 {
                        darken(armor_dark, 0.8)
                    } else {
                        darken(armor_shadow, 0.9)
                    };
                    pixels[y][x] = Some(color);
                }
            }
        }
    }

    // === TORSO / CHEST ARMOR ===
    for y in chest_top..waist_y {
        let progress = (y - chest_top) as f32 / (waist_y - chest_top) as f32;
        let torso_w = body_width + ((1.0 - progress) * 4.0) as usize;
        let start_x = cx.saturating_sub(torso_w / 2);

        for tx in 0..torso_w {
            let x = start_x + tx;
            if x < w && y < h {
                // Create detailed armor plate look
                let in_center = tx > torso_w / 4 && tx < torso_w * 3 / 4;
                let row_in_plate = (y - chest_top) % 6;

                let color = if tx == 0 {
                    outline
                } else if tx == torso_w - 1 {
                    armor_shadow
                } else if row_in_plate == 0 {
                    armor_dark  // Plate edges
                } else if in_center && row_in_plate == 3 {
                    armor_highlight  // Center highlight
                } else if tx < torso_w / 3 {
                    armor_light
                } else if tx < torso_w * 2 / 3 {
                    armor_mid
                } else {
                    armor_dark
                };
                pixels[y][x] = Some(color);
            }
        }
    }

    // === BELT ===
    for y in waist_y..hip_y {
        let belt_w = body_width + 4;
        let start_x = cx.saturating_sub(belt_w / 2);
        for bx in 0..belt_w {
            let x = start_x + bx;
            if x < w && y < h {
                let color = if bx == 0 || bx == belt_w - 1 {
                    outline
                } else if bx == belt_w / 2 && y == (waist_y + hip_y) / 2 {
                    metal_shine  // Belt buckle
                } else if bx > belt_w / 2 - 2 && bx < belt_w / 2 + 2 {
                    metal_light  // Buckle area
                } else {
                    darken(armor_dark, 0.7)
                };
                pixels[y][x] = Some(color);
            }
        }
    }

    // === SHOULDERS / PAULDRONS ===
    draw_detailed_shoulders(pixels, cx, w, h, shoulder_y, shoulder_width, armor, armor_light, armor_dark, armor_shadow, metal, metal_light, outline, desc.archetype);

    // === ARMS ===
    draw_detailed_arms(pixels, cx, w, h, shoulder_y, waist_y, shoulder_width, body_width, skin, skin_light, skin_dark, armor, armor_dark, desc.archetype);

    // === HEAD / HELMET ===
    draw_detailed_head(pixels, desc, cx, w, h, head_height, armor, armor_light, armor_dark, armor_shadow, armor_highlight, metal, metal_light, metal_dark, metal_shine, metal_shadow, skin, skin_light, skin_dark, outline, black);

    // === SHIELD ===
    if desc.has_shield {
        draw_detailed_shield(pixels, cx, w, h, chest_top, waist_y, shoulder_width, armor, armor_light, armor_dark, metal, metal_light, metal_shine, outline);
    }

    // === WEAPON ===
    draw_detailed_weapon(pixels, desc.weapon, cx, w, h, shoulder_y, waist_y, shoulder_width, metal, metal_light, metal_dark, metal_shine, metal_shadow, outline);
}

/// Draw detailed cape
fn draw_detailed_cape(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    cx: usize, w: usize, h: usize,
    shoulder_y: usize, hip_y: usize,
    armor: [u8; 3], armor_dark: [u8; 3], armor_shadow: [u8; 3],
) {
    let cape_start = shoulder_y;
    let cape_end = h * 88 / 100;

    for y in cape_start..cape_end.min(h) {
        let progress = (y - cape_start) as f32 / (cape_end - cape_start) as f32;
        // Cape gets wider as it goes down with some wave
        let wave = ((y as f32 * 0.15).sin() * 3.0) as i32;
        let cape_width = (w as f32 * (0.25 + progress * 0.35)) as usize;
        let cape_offset = (wave + 2) as usize;

        let cape_left = cx.saturating_sub(cape_width + cape_offset);
        let cape_right = cx.saturating_sub(cape_offset.saturating_sub(2));

        for x in cape_left..cape_right.min(w) {
            if y < h && pixels[y][x].is_none() {
                let local_x = x - cape_left;
                let cape_w = cape_right - cape_left;
                let color = if local_x < cape_w / 4 {
                    armor_shadow
                } else if local_x < cape_w / 2 {
                    armor_dark
                } else if local_x < cape_w * 3 / 4 {
                    armor
                } else {
                    lighten(armor, 1.1)
                };
                pixels[y][x] = Some(color);
            }
        }
    }
}

/// Draw detailed shoulders with pauldrons
fn draw_detailed_shoulders(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    cx: usize, w: usize, h: usize,
    shoulder_y: usize, shoulder_width: usize,
    armor: [u8; 3], armor_light: [u8; 3], armor_dark: [u8; 3], armor_shadow: [u8; 3],
    metal: [u8; 3], metal_light: [u8; 3], outline: [u8; 3],
    archetype: WarriorArchetype,
) {
    let pauldron_height = match archetype {
        WarriorArchetype::Tiny => 3,
        WarriorArchetype::Light => 4,
        WarriorArchetype::Medium => 5,
        WarriorArchetype::Heavy => 7,
        WarriorArchetype::Giant => 6,
        WarriorArchetype::Hulk => 8,
    };

    // Left pauldron
    let left_start = cx.saturating_sub(shoulder_width / 2);
    let left_end = cx.saturating_sub(2);
    for y in shoulder_y..(shoulder_y + pauldron_height).min(h) {
        let row = y - shoulder_y;
        let width_mod = pauldron_height.saturating_sub(row);
        for x in left_start.saturating_sub(width_mod)..left_end {
            if x < w && y < h {
                let local_x = x - left_start.saturating_sub(width_mod);
                let pw = left_end - left_start.saturating_sub(width_mod);
                let color = if row == 0 {
                    if local_x < pw / 2 { armor_light } else { armor }
                } else if row == pauldron_height - 1 {
                    armor_shadow
                } else if local_x == 0 {
                    outline
                } else if local_x < pw / 3 {
                    armor_light
                } else if local_x < pw * 2 / 3 {
                    armor
                } else {
                    armor_dark
                };
                pixels[y][x] = Some(color);
            }
        }
        // Metal trim
        if row == pauldron_height / 2 {
            for x in left_start.saturating_sub(width_mod)..left_end {
                if x < w && y < h {
                    pixels[y][x] = Some(metal);
                }
            }
        }
    }

    // Right pauldron
    let right_start = cx + 2;
    let right_end = cx + shoulder_width / 2;
    for y in shoulder_y..(shoulder_y + pauldron_height).min(h) {
        let row = y - shoulder_y;
        let width_mod = pauldron_height.saturating_sub(row);
        for x in right_start..(right_end + width_mod).min(w) {
            if y < h {
                let local_x = x - right_start;
                let pw = right_end + width_mod - right_start;
                let color = if row == 0 {
                    if local_x < pw / 2 { armor } else { armor_dark }
                } else if row == pauldron_height - 1 {
                    armor_shadow
                } else if local_x == pw - 1 {
                    outline
                } else if local_x < pw / 3 {
                    armor_light
                } else if local_x < pw * 2 / 3 {
                    armor
                } else {
                    armor_dark
                };
                pixels[y][x] = Some(color);
            }
        }
        // Metal trim
        if row == pauldron_height / 2 {
            for x in right_start..(right_end + width_mod).min(w) {
                if y < h {
                    pixels[y][x] = Some(metal);
                }
            }
        }
    }
}

/// Draw detailed arms
fn draw_detailed_arms(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    cx: usize, w: usize, h: usize,
    shoulder_y: usize, waist_y: usize,
    shoulder_width: usize, body_width: usize,
    skin: [u8; 3], skin_light: [u8; 3], skin_dark: [u8; 3],
    armor: [u8; 3], armor_dark: [u8; 3],
    archetype: WarriorArchetype,
) {
    let arm_width = match archetype {
        WarriorArchetype::Tiny => 2,
        WarriorArchetype::Light => 3,
        WarriorArchetype::Medium => 4,
        WarriorArchetype::Heavy => 5,
        WarriorArchetype::Giant => 5,
        WarriorArchetype::Hulk => 6,
    };

    let arm_top = shoulder_y + 4;
    let arm_bottom = waist_y + 3;
    let elbow_y = (arm_top + arm_bottom) / 2;

    // Left arm
    let left_arm_x = cx.saturating_sub(shoulder_width / 2 + 1);
    for y in arm_top..arm_bottom.min(h) {
        let is_upper = y < elbow_y;
        for ax in 0..arm_width {
            let x = left_arm_x.saturating_sub(ax);
            if x < w && y < h {
                let color = if is_upper {
                    if ax == 0 { armor } else { armor_dark }
                } else {
                    if ax == 0 { skin_light } else if ax == arm_width - 1 { skin_dark } else { skin }
                };
                pixels[y][x] = Some(color);
            }
        }
    }

    // Right arm
    let right_arm_x = cx + shoulder_width / 2;
    for y in arm_top..arm_bottom.min(h) {
        let is_upper = y < elbow_y;
        for ax in 0..arm_width {
            let x = right_arm_x + ax;
            if x < w && y < h {
                let color = if is_upper {
                    if ax == arm_width - 1 { armor_dark } else { armor }
                } else {
                    if ax == 0 { skin_light } else if ax == arm_width - 1 { skin_dark } else { skin }
                };
                pixels[y][x] = Some(color);
            }
        }
    }

    // Gauntlets / hand armor
    for y in (arm_bottom - 3).max(arm_top)..arm_bottom.min(h) {
        for ax in 0..arm_width + 1 {
            let left_x = left_arm_x.saturating_sub(ax);
            let right_x = right_arm_x + ax;
            if left_x < w && y < h {
                pixels[y][left_x] = Some(armor_dark);
            }
            if right_x < w && y < h {
                pixels[y][right_x] = Some(armor_dark);
            }
        }
    }
}

/// Draw detailed head/helmet
fn draw_detailed_head(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    desc: &SpriteDescription,
    cx: usize, w: usize, h: usize,
    head_height: usize,
    armor: [u8; 3], armor_light: [u8; 3], armor_dark: [u8; 3], armor_shadow: [u8; 3], armor_highlight: [u8; 3],
    metal: [u8; 3], metal_light: [u8; 3], metal_dark: [u8; 3], metal_shine: [u8; 3], metal_shadow: [u8; 3],
    skin: [u8; 3], skin_light: [u8; 3], skin_dark: [u8; 3],
    outline: [u8; 3], black: [u8; 3],
) {
    let head_width = head_height + 2;
    let face_y = head_height * 55 / 100;

    match desc.helmet {
        HelmetStyle::None => {
            // Bare head with detailed face
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = if progress < 0.3 {
                    (head_width as f32 * (0.7 + progress)) as usize
                } else if progress > 0.8 {
                    (head_width as f32 * (1.0 - (progress - 0.8) * 2.0)) as usize
                } else {
                    head_width
                };
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { skin_dark }
                            else if px < width / 3 { skin_light }
                            else if px < width * 2 / 3 { skin }
                            else { skin_dark };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Hair on top
            for y in 0..head_height / 3 {
                let width = head_width - y;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        pixels[y][x] = Some([40, 30, 25]); // Dark hair
                    }
                }
            }
            // Eyes
            if face_y < h {
                let eye_spacing = head_width / 4;
                if cx.saturating_sub(eye_spacing) < w { pixels[face_y][cx.saturating_sub(eye_spacing)] = Some(black); }
                if cx + eye_spacing < w { pixels[face_y][cx + eye_spacing] = Some(black); }
            }
        }
        HelmetStyle::Hood => {
            for y in 0..head_height + 2 {
                let progress = y as f32 / (head_height + 2) as f32;
                let width = head_width + 4 - (progress * 3.0) as usize;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { armor_shadow }
                            else if y < 2 { armor_dark }
                            else if px < width / 3 { armor_light }
                            else { armor };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Face opening
            let face_top = head_height / 3;
            let face_bottom = head_height * 4 / 5;
            for y in face_top..face_bottom {
                let face_w = head_width / 2;
                let start_x = cx.saturating_sub(face_w / 2);
                for px in 0..face_w {
                    let x = start_x + px;
                    if x < w && y < h {
                        pixels[y][x] = Some(if px < face_w / 3 { skin_light } else { skin });
                    }
                }
            }
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Helm => {
            // Full detailed helmet
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = if progress < 0.2 {
                    (head_width as f32 * (0.8 + progress)) as usize
                } else {
                    head_width + 2
                };
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 || y == 0 { outline }
                            else if px == width - 1 { armor_shadow }
                            else if y < head_height / 4 {
                                if px < width / 2 { armor_light } else { armor }
                            }
                            else if y == head_height / 2 { armor_dark } // Visor line
                            else if px < width / 3 { armor_light }
                            else if px < width * 2 / 3 { armor }
                            else { armor_dark };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Visor slit
            let visor_y = head_height / 2;
            if visor_y < h {
                for px in 0..head_width / 2 {
                    let x = cx.saturating_sub(head_width / 4) + px;
                    if x < w {
                        pixels[visor_y][x] = Some(black);
                    }
                }
            }
            // Nose guard
            if face_y < h && cx < w {
                pixels[face_y][cx] = Some(armor_dark);
                if face_y + 1 < h { pixels[face_y + 1][cx] = Some(armor_shadow); }
            }
        }
        HelmetStyle::Crested => {
            // Roman/Spartan helmet with large crest
            for y in 0..head_height {
                let width = head_width + 2;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { metal_shadow }
                            else if px < width / 3 { metal_light }
                            else if px < width * 2 / 3 { metal }
                            else { metal_dark };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Tall crest/plume
            let crest_height = head_height + 8;
            let crest_colors = [[180, 30, 30], [200, 40, 40], [160, 25, 25]];
            for y in 0..crest_height.min(h) {
                let crest_w = if y < 3 { 2 } else { 3 };
                for px in 0..crest_w {
                    let x = cx.saturating_sub(crest_w / 2) + px;
                    if x < w {
                        let color_idx = (y + px) % 3;
                        pixels[y][x] = Some(crest_colors[color_idx]);
                    }
                }
            }
            // Face guard
            if face_y < h {
                for x in cx.saturating_sub(3)..=(cx + 3).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
            // Cheek guards
            for y in face_y + 1..head_height {
                if y < h {
                    let left_x = cx.saturating_sub(head_width / 2);
                    let right_x = cx + head_width / 2;
                    if left_x < w { pixels[y][left_x] = Some(metal_dark); }
                    if right_x < w { pixels[y][right_x] = Some(metal_dark); }
                }
            }
        }
        HelmetStyle::Horned => {
            // Viking helmet with horns
            for y in 0..head_height {
                let width = head_width + 2;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { armor_shadow }
                            else if px < width / 3 { armor_light }
                            else { armor };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Horns curving outward and up
            let horn_color = [235, 225, 190];
            let horn_dark = [200, 185, 150];
            for horn_i in 0..6 {
                let horn_y = (head_height / 4).saturating_sub(horn_i / 2);
                let offset = head_width / 2 + horn_i + 2;
                if horn_y < h {
                    if cx.saturating_sub(offset) < w {
                        pixels[horn_y][cx.saturating_sub(offset)] = Some(if horn_i < 3 { horn_color } else { horn_dark });
                    }
                    if cx + offset < w {
                        pixels[horn_y][cx + offset] = Some(if horn_i < 3 { horn_dark } else { horn_color });
                    }
                }
            }
            // Visor
            if face_y < h {
                for x in cx.saturating_sub(3)..=(cx + 3).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
        }
        HelmetStyle::Samurai => {
            // Japanese kabuto with flared sides
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let flare = if progress > 0.5 { ((progress - 0.5) * 8.0) as usize } else { 0 };
                let width = head_width + flare;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 || px == width - 1 { outline }
                            else if y < head_height / 3 { armor_dark }
                            else if px < width / 3 { armor_light }
                            else { armor };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Crest ornament
            if cx < w && 0 < h {
                pixels[0][cx] = Some(metal_shine);
                if 1 < h { pixels[1][cx] = Some(metal); }
            }
            // Menpo (face mask)
            let mask_top = head_height * 2 / 3;
            for y in mask_top..head_height {
                let mask_w = head_width / 2;
                let start_x = cx.saturating_sub(mask_w / 2);
                for px in 0..mask_w {
                    let x = start_x + px;
                    if x < w && y < h {
                        pixels[y][x] = Some([140, 30, 30]); // Red mask
                    }
                }
            }
        }
        HelmetStyle::Crown => {
            // Royal crown
            for y in 0..head_height {
                let is_crown = y < head_height / 3;
                let width = head_width + (if is_crown { 4 } else { 0 });
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_crown {
                            if px == 0 || px == width - 1 { [200, 160, 0] }
                            else { [255, 215, 0] }
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Crown points with jewels
            let points = [cx.saturating_sub(4), cx, cx + 4];
            for &point_x in &points {
                if point_x < w {
                    if 0 < h { pixels[0][point_x] = Some([255, 50, 50]); } // Ruby
                    if 1 < h { pixels[1][point_x] = Some([255, 215, 0]); }
                }
            }
            // Face
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Turban => {
            // Elaborate turban
            for y in 0..head_height {
                let is_turban = y < head_height * 2 / 3;
                let width = head_width + (if is_turban { 4 + (y % 2) } else { 0 });
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_turban {
                            if (y + px) % 3 == 0 { armor_light } else { armor }
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Turban jewel
            if 2 < h && cx < w {
                pixels[2][cx] = Some([50, 200, 80]); // Emerald
                if cx > 0 { pixels[2][cx - 1] = Some(metal_shine); }
                if cx + 1 < w { pixels[2][cx + 1] = Some(metal_shine); }
            }
            // Face
            let face_start = head_height * 2 / 3;
            if face_start + 2 < h && cx > 1 && cx + 1 < w {
                pixels[face_start + 2][cx - 1] = Some(black);
                pixels[face_start + 2][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Tricorn => {
            // Colonial tricorn hat
            let hat_bottom = head_height / 2;
            for y in 0..head_height {
                let is_hat = y < hat_bottom;
                let width = if is_hat { head_width + 6 } else { head_width };
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_hat {
                            if y == hat_bottom - 1 { [50, 40, 35] } // Hat band
                            else { [30, 25, 30] } // Dark hat
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Cockade/plume
            if 1 < h && cx + 3 < w {
                pixels[1][cx + 3] = Some([200, 200, 200]);
            }
            // Face
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Mask => {
            // Tribal/ninja mask - face with covering
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = head_width + 1;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let is_mask = progress > 0.4;
                        let color = if is_mask {
                            if px == 0 { outline } else { [30, 30, 35] } // Dark mask
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Eyes visible above mask
            let eye_y = head_height / 3;
            if eye_y < h && cx > 1 && cx + 1 < w {
                pixels[eye_y][cx - 1] = Some(black);
                pixels[eye_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Cap => {
            // Simple cap/hat
            for y in 0..head_height {
                let is_cap = y < head_height / 3;
                let width = head_width + (if is_cap { 2 } else { 0 });
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_cap { armor }
                            else if px < width / 3 { skin_light } else { skin };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Face
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Conical => {
            // Asian conical hat
            for y in 0..head_height + 2 {
                let progress = y as f32 / (head_height + 2) as f32;
                let width = ((1.0 - progress) * (head_width as f32 + 8.0)) as usize;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width.max(1) {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if progress < 0.6 {
                            [180, 160, 120] // Straw color
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Face
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Beret => {
            // Military beret
            for y in 0..head_height {
                let is_beret = y < head_height / 3;
                let offset = if is_beret && y > 0 { 3 } else { 0 }; // Beret tilts
                let width = head_width + (if is_beret { 3 } else { 0 });
                let start_x = cx.saturating_sub(width / 2) + offset;
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_beret { armor_dark }
                            else if px < width / 3 { skin_light } else { skin };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Face
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Gasmask => {
            // WWI/Dystopian gas mask
            for y in 0..head_height {
                let width = head_width + 2;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 || px == width - 1 { outline }
                            else { [60, 65, 55] }; // Olive drab
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Round eye lenses
            let lens_y = head_height / 3;
            if lens_y < h && cx > 2 && cx + 2 < w {
                pixels[lens_y][cx - 2] = Some([100, 150, 100]); // Left lens
                pixels[lens_y][cx + 2] = Some([100, 150, 100]); // Right lens
            }
            // Breathing filter below
            for y in (head_height * 2 / 3)..head_height {
                if y < h && cx < w {
                    pixels[y][cx] = Some([40, 45, 35]);
                    if cx > 0 { pixels[y][cx - 1] = Some([50, 55, 45]); }
                    if cx + 1 < w { pixels[y][cx + 1] = Some([50, 55, 45]); }
                }
            }
        }
        HelmetStyle::Visor => {
            // Futuristic visor / Cyclops style
            for y in 0..head_height {
                let width = head_width + 2;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { armor_shadow }
                            else { armor };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Glowing visor band
            let visor_y = head_height / 3;
            let visor_h = 3.min(head_height / 3);
            for vy in visor_y..(visor_y + visor_h).min(h) {
                for x in cx.saturating_sub(head_width / 2)..=(cx + head_width / 2).min(w - 1) {
                    pixels[vy][x] = Some([255, 50, 50]); // Red visor glow
                }
            }
        }
        HelmetStyle::Mandalorian => {
            // Mandalorian-style T-visor helmet
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = if progress < 0.3 {
                    (head_width as f32 * (0.9 + progress * 0.3)) as usize
                } else {
                    head_width + 2
                };
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 { outline }
                            else if px == width - 1 { metal_shadow }
                            else if px < width / 3 { metal_light }
                            else if px < width * 2 / 3 { metal }
                            else { metal_dark };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // T-visor
            let visor_top = head_height / 3;
            let visor_bottom = head_height * 2 / 3;
            // Horizontal part
            if visor_top < h {
                for x in cx.saturating_sub(head_width / 3)..=(cx + head_width / 3).min(w - 1) {
                    pixels[visor_top][x] = Some([20, 20, 25]); // Dark visor
                }
            }
            // Vertical part
            for y in visor_top..visor_bottom.min(h) {
                if cx < w { pixels[y][cx] = Some([20, 20, 25]); }
            }
        }
        HelmetStyle::Pilot => {
            // Space/fighter pilot helmet
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = head_width + 4;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px == 0 || px == width - 1 { outline }
                            else if progress < 0.6 { [240, 240, 245] } // White helmet
                            else { [50, 50, 55] }; // Dark lower
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Tinted visor
            let visor_y = head_height / 4;
            let visor_h = head_height / 3;
            for vy in visor_y..(visor_y + visor_h).min(h) {
                for px in 0..head_width {
                    let x = cx.saturating_sub(head_width / 2) + px;
                    if x < w {
                        pixels[vy][x] = Some([80, 60, 40]); // Gold tint
                    }
                }
            }
        }
        HelmetStyle::Cyber => {
            // Cyberpunk implants/augments
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = head_width;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px < width / 3 { skin_light } else { skin };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Cyber implants on one side
            for y in (head_height / 4)..(head_height * 3 / 4) {
                let x = cx + head_width / 2;
                if x < w && y < h {
                    pixels[y][x] = Some([0, 255, 255]); // Cyan glow
                }
                if x + 1 < w && y < h {
                    pixels[y][x + 1] = Some([50, 50, 60]); // Metal implant
                }
            }
            // Cyber eye
            let eye_y = head_height / 3;
            if eye_y < h {
                if cx + 2 < w { pixels[eye_y][cx + 2] = Some([255, 0, 0]); } // Red cyber eye
                if cx > 1 { pixels[eye_y][cx - 2] = Some(black); } // Normal eye
            }
        }
        HelmetStyle::Wizard => {
            // Pointed wizard/mage hat
            let hat_top = 0;
            for y in 0..head_height + 4 {
                let progress = y as f32 / (head_height + 4) as f32;
                let is_hat = progress < 0.7;
                let width = if is_hat {
                    ((1.0 - progress / 0.7) * (head_width as f32 + 4.0)) as usize
                } else {
                    head_width
                };
                let width = width.max(1);
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if is_hat {
                            if (y + px) % 4 == 0 { [80, 40, 120] } // Dark purple stripes
                            else { [100, 50, 150] } // Purple hat
                        } else {
                            if px < width / 3 { skin_light } else { skin }
                        };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Star on hat
            if 2 < h && cx < w {
                pixels[2][cx] = Some([255, 255, 100]); // Yellow star
            }
            // Face
            let face_y_adj = head_height * 85 / 100;
            if face_y_adj < h && cx > 1 && cx + 1 < w {
                pixels[face_y_adj][cx - 1] = Some(black);
                pixels[face_y_adj][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Halo => {
            // Angelic halo above head
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = head_width;
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        let color = if px < width / 3 { skin_light } else { skin };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Golden halo ring above head
            let halo_y = 0;
            let halo_w = head_width + 4;
            if halo_y < h {
                for px in 0..halo_w {
                    let x = cx.saturating_sub(halo_w / 2) + px;
                    if x < w && (px < 2 || px >= halo_w - 2) {
                        pixels[halo_y][x] = Some([255, 215, 0]); // Gold
                    }
                }
            }
            // Eyes
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some([100, 180, 255]); // Glowing blue eyes
                pixels[face_y][cx + 1] = Some([100, 180, 255]);
            }
        }
        HelmetStyle::Skull => {
            // Skeleton/death knight skull face
            for y in 0..head_height {
                let progress = y as f32 / head_height as f32;
                let width = if progress < 0.3 || progress > 0.8 {
                    (head_width as f32 * 0.8) as usize
                } else {
                    head_width
                };
                let start_x = cx.saturating_sub(width / 2);
                for px in 0..width {
                    let x = start_x + px;
                    if x < w && y < h {
                        pixels[y][x] = Some([220, 210, 200]); // Bone white
                    }
                }
            }
            // Dark eye sockets
            let eye_y = head_height / 3;
            if eye_y < h && cx > 2 && cx + 2 < w {
                pixels[eye_y][cx - 2] = Some(black);
                pixels[eye_y][cx + 2] = Some(black);
                if eye_y + 1 < h {
                    pixels[eye_y + 1][cx - 2] = Some(black);
                    pixels[eye_y + 1][cx + 2] = Some(black);
                }
            }
            // Nose hole
            let nose_y = head_height / 2;
            if nose_y < h && cx < w {
                pixels[nose_y][cx] = Some([50, 40, 35]);
            }
            // Teeth
            let teeth_y = head_height * 3 / 4;
            if teeth_y < h {
                for tx in 0..4 {
                    let x = cx.saturating_sub(2) + tx;
                    if x < w { pixels[teeth_y][x] = Some([240, 235, 220]); }
                }
            }
        }
    }
}

/// Draw detailed shield
fn draw_detailed_shield(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    cx: usize, w: usize, h: usize,
    chest_top: usize, waist_y: usize, shoulder_width: usize,
    armor: [u8; 3], armor_light: [u8; 3], armor_dark: [u8; 3],
    metal: [u8; 3], metal_light: [u8; 3], metal_shine: [u8; 3],
    outline: [u8; 3],
) {
    let shield_cx = cx.saturating_sub(shoulder_width / 2 + 6);
    let shield_top = chest_top;
    let shield_bottom = waist_y + 4;
    let shield_width = 8;
    let shield_height = shield_bottom - shield_top;

    for y in shield_top..shield_bottom.min(h) {
        let row = y - shield_top;
        let progress = row as f32 / shield_height as f32;

        // Shield tapers at top and bottom
        let taper = if progress < 0.2 { (progress * 5.0 * shield_width as f32) as usize }
            else if progress > 0.8 { ((1.0 - progress) * 5.0 * shield_width as f32) as usize }
            else { shield_width };

        let start_x = shield_cx.saturating_sub(taper / 2);
        for sx in 0..taper {
            let x = start_x + sx;
            if x < w && y < h {
                let color = if sx == 0 { outline }
                    else if sx == taper - 1 { armor_dark }
                    else if sx == taper / 2 && row == shield_height / 2 { metal_shine } // Boss
                    else if sx < taper / 3 { armor_light }
                    else if sx < taper * 2 / 3 { armor }
                    else { armor_dark };
                pixels[y][x] = Some(color);
            }
        }
    }

    // Shield rim
    let rim_y = (shield_top + shield_bottom) / 2;
    if rim_y < h {
        for sx in 0..shield_width {
            let x = shield_cx.saturating_sub(shield_width / 2) + sx;
            if x < w {
                pixels[rim_y][x] = Some(metal);
            }
        }
    }
}

/// Draw detailed weapon
fn draw_detailed_weapon(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    weapon: WeaponType,
    cx: usize, w: usize, h: usize,
    shoulder_y: usize, waist_y: usize, shoulder_width: usize,
    metal: [u8; 3], metal_light: [u8; 3], metal_dark: [u8; 3], metal_shine: [u8; 3], metal_shadow: [u8; 3],
    outline: [u8; 3],
) {
    let weapon_x = cx + shoulder_width / 2 + 4;
    let wood = [101, 67, 33];
    let wood_light = [130, 90, 50];
    let wood_dark = [70, 45, 20];

    match weapon {
        WeaponType::Sword | WeaponType::Saber | WeaponType::Rapier | WeaponType::Scimitar => {
            // Detailed sword with fuller
            let blade_top = 2;
            let blade_bottom = waist_y;
            let blade_width = 3;
            for y in blade_top..blade_bottom.min(h) {
                let progress = (y - blade_top) as f32 / (blade_bottom - blade_top) as f32;
                for bx in 0..blade_width {
                    let x = weapon_x + bx;
                    if x < w {
                        let color = if bx == 0 { metal_shine }
                            else if bx == blade_width - 1 { metal_dark }
                            else if progress < 0.1 { metal_shine }
                            else { metal_light };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Cross-guard
            let guard_y = shoulder_y + 3;
            if guard_y < h {
                for gx in 0..7 {
                    let x = weapon_x.saturating_sub(2) + gx;
                    if x < w { pixels[guard_y][x] = Some(metal_dark); }
                }
            }
            // Handle
            for y in guard_y + 1..(guard_y + 5).min(h) {
                if weapon_x + 1 < w {
                    pixels[y][weapon_x] = Some(wood_dark);
                    pixels[y][weapon_x + 1] = Some(wood);
                }
            }
        }
        WeaponType::Katana | WeaponType::Naginata => {
            // Curved katana
            let blade_top = 1;
            let blade_bottom = waist_y - 2;
            for y in blade_top..blade_bottom.min(h) {
                let curve = ((y - blade_top) as f32 * 0.05) as usize;
                let x = weapon_x + curve;
                if x < w {
                    pixels[y][x] = Some(metal_shine);
                    if x + 1 < w { pixels[y][x + 1] = Some(metal_light); }
                }
            }
            // Tsuba (guard)
            let guard_y = shoulder_y + 2;
            if guard_y < h {
                for gx in 0..5 {
                    let x = weapon_x.saturating_sub(1) + gx;
                    if x < w { pixels[guard_y][x] = Some(metal_dark); }
                }
            }
            // Tsuka (handle)
            for y in guard_y + 1..(guard_y + 6).min(h) {
                let x = weapon_x + 1;
                if x < w {
                    pixels[y][x] = Some(if y % 2 == 0 { [40, 30, 50] } else { wood_dark });
                }
            }
        }
        WeaponType::Spear | WeaponType::Pike | WeaponType::Halberd | WeaponType::Trident => {
            // Long polearm
            for y in 0..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                    if weapon_x + 1 < w && y > h / 2 { pixels[y][weapon_x + 1] = Some(wood_dark); }
                }
            }
            // Spearhead (trident has 3 points)
            let prongs = if matches!(weapon, WeaponType::Trident) { 3 } else { 1 };
            for prong in 0..prongs {
                let offset = (prong as i32 - 1) * 2;
                for y in 0..8.min(h) {
                    let tip_w = if y < 2 { 1 } else if y < 5 { 2 } else { 3 };
                    for tx in 0..tip_w {
                        let x = (weapon_x as i32 + offset + tx as i32) as usize;
                        if x < w {
                            pixels[y][x] = Some(if tx == 0 { metal_shine } else { metal_light });
                        }
                    }
                }
            }
        }
        WeaponType::Axe | WeaponType::Warhammer => {
            // Battle axe / warhammer
            for y in shoulder_y..(h - 2) {
                if weapon_x < w { pixels[y][weapon_x] = Some(wood); }
            }
            let head_top = shoulder_y.saturating_sub(4);
            let head_bottom = shoulder_y + 4;
            for y in head_top..head_bottom.min(h) {
                let row = y.saturating_sub(head_top);
                let blade_w = if row < 2 { 3 } else if row < 5 { 5 } else { 4 };
                for bx in 0..blade_w {
                    let x = weapon_x + 1 + bx;
                    if x < w {
                        pixels[y][x] = Some(if bx == 0 { metal_light } else { metal });
                    }
                }
            }
        }
        WeaponType::Bow | WeaponType::Crossbow => {
            let bow_top = shoulder_y.saturating_sub(6);
            let bow_bottom = waist_y + 6;
            let bow_mid = (bow_top + bow_bottom) / 2;
            for y in bow_top..bow_bottom.min(h) {
                let dist_from_mid = (y as i32 - bow_mid as i32).abs() as usize;
                let curve = dist_from_mid / 3;
                let x = weapon_x.saturating_sub(curve);
                if x < w {
                    pixels[y][x] = Some(wood);
                    if x > 0 { pixels[y][x - 1] = Some(wood_dark); }
                }
            }
            for y in bow_top..bow_bottom.min(h) {
                if weapon_x + 2 < w { pixels[y][weapon_x + 2] = Some([180, 160, 140]); }
            }
        }
        WeaponType::Musket | WeaponType::Rifle | WeaponType::Pistol => {
            // Firearms
            let barrel_len = if matches!(weapon, WeaponType::Pistol) { waist_y / 2 } else { waist_y };
            for y in 0..barrel_len.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some([50, 45, 50]);
                    if weapon_x + 1 < w { pixels[y][weapon_x + 1] = Some([35, 30, 35]); }
                }
            }
            for y in shoulder_y..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                    if weapon_x + 1 < w { pixels[y][weapon_x + 1] = Some(wood_dark); }
                }
            }
        }
        // === SCI-FI WEAPONS ===
        WeaponType::Lightsaber | WeaponType::BeamSword => {
            // Glowing energy blade
            let blade_colors = [[100, 200, 255], [150, 220, 255], [200, 240, 255]]; // Blue default
            let blade_top = 1;
            let blade_bottom = waist_y - 2;
            // Glowing blade
            for y in blade_top..blade_bottom.min(h) {
                let glow_w = 3;
                for gx in 0..glow_w {
                    let x = weapon_x + gx;
                    if x < w {
                        pixels[y][x] = Some(blade_colors[gx % 3]);
                    }
                }
            }
            // Handle
            for y in shoulder_y..(shoulder_y + 6).min(h) {
                if weapon_x + 1 < w {
                    pixels[y][weapon_x] = Some([60, 60, 70]);
                    pixels[y][weapon_x + 1] = Some([40, 40, 50]);
                }
            }
        }
        WeaponType::LaserRifle | WeaponType::PlasmaGun | WeaponType::RailGun => {
            // Sci-fi gun with glowing parts
            let gun_color = [60, 60, 70];
            let glow_color = [0, 255, 200]; // Cyan glow
            // Main body
            for y in 0..waist_y.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(gun_color);
                    if weapon_x + 1 < w { pixels[y][weapon_x + 1] = Some([40, 40, 50]); }
                }
            }
            // Glowing barrel tip
            for y in 0..4.min(h) {
                if weapon_x < w { pixels[y][weapon_x] = Some(glow_color); }
            }
            // Stock
            for y in shoulder_y..h {
                if weapon_x + 2 < w {
                    pixels[y][weapon_x + 2] = Some([50, 50, 55]);
                }
            }
        }
        // === FANTASY WEAPONS ===
        WeaponType::Staff | WeaponType::Wand => {
            // Magic staff with orb
            for y in 0..h {
                if weapon_x < w { pixels[y][weapon_x] = Some(wood); }
            }
            // Crystal orb
            for y in 0..4.min(h) {
                for ox in 0..3 {
                    let x = weapon_x.saturating_sub(1) + ox;
                    if x < w {
                        let color = if y == 0 || y == 3 { [80, 180, 255] }
                            else if ox == 1 { [150, 220, 255] }
                            else { [100, 200, 255] };
                        pixels[y][x] = Some(color);
                    }
                }
            }
        }
        WeaponType::Orb => {
            // Floating magical orb
            for y in 0..6.min(h) {
                let orb_w = if y < 2 || y > 3 { 3 } else { 5 };
                for ox in 0..orb_w {
                    let x = weapon_x.saturating_sub(orb_w / 2) + ox;
                    if x < w {
                        let color = if ox == 0 || ox == orb_w - 1 { [180, 100, 255] }
                            else { [220, 150, 255] };
                        pixels[y][x] = Some(color);
                    }
                }
            }
        }
        WeaponType::HolyBlade => {
            // Glowing holy sword
            let blade_top = 2;
            let blade_bottom = waist_y;
            for y in blade_top..blade_bottom.min(h) {
                for bx in 0..3 {
                    let x = weapon_x + bx;
                    if x < w {
                        pixels[y][x] = Some([255, 255, 200]); // Golden glow
                    }
                }
            }
            // Cross-guard
            let guard_y = shoulder_y + 3;
            if guard_y < h {
                for gx in 0..7 {
                    let x = weapon_x.saturating_sub(2) + gx;
                    if x < w { pixels[guard_y][x] = Some([255, 215, 0]); }
                }
            }
        }
        WeaponType::DemonSword => {
            // Dark cursed blade with red glow
            let blade_top = 2;
            let blade_bottom = waist_y;
            for y in blade_top..blade_bottom.min(h) {
                for bx in 0..3 {
                    let x = weapon_x + bx;
                    if x < w {
                        let color = if bx == 1 { [150, 20, 20] } else { [80, 10, 10] };
                        pixels[y][x] = Some(color);
                    }
                }
            }
            // Jagged edge
            for y in (blade_top..blade_bottom.min(h)).step_by(3) {
                let x = weapon_x + 3;
                if x < w { pixels[y][x] = Some([100, 15, 15]); }
            }
        }
        WeaponType::Torch => {
            // Flaming torch
            for y in shoulder_y..h {
                if weapon_x < w { pixels[y][weapon_x] = Some(wood); }
            }
            let flame_colors = [[255, 200, 50], [255, 150, 30], [255, 100, 20]];
            for y in 0..8.min(h) {
                let flame_w = 5 - (y / 2);
                for fx in 0..flame_w {
                    let x = weapon_x.saturating_sub(flame_w / 2) + fx;
                    if x < w { pixels[y][x] = Some(flame_colors[y % 3]); }
                }
            }
        }
        WeaponType::Scythe => {
            // Death's scythe
            for y in 0..h {
                if weapon_x < w { pixels[y][weapon_x] = Some(wood_dark); }
            }
            // Curved blade
            for y in 0..10.min(h) {
                let curve = y / 2;
                let blade_x = weapon_x.saturating_sub(curve);
                if blade_x < w {
                    pixels[y][blade_x] = Some(metal_shine);
                    if blade_x > 0 { pixels[y][blade_x - 1] = Some(metal_dark); }
                }
            }
        }
        // === EXOTIC WEAPONS ===
        WeaponType::Whip => {
            // Coiled whip
            for y in shoulder_y..h {
                let wave = ((y as f32 * 0.5).sin() * 2.0) as i32;
                let x = (weapon_x as i32 + wave) as usize;
                if x < w && y < h { pixels[y][x] = Some([60, 40, 30]); }
            }
        }
        WeaponType::Claws => {
            // Sharp claws extending from hand
            for claw in 0..3 {
                let claw_x = weapon_x + claw * 2;
                for y in shoulder_y.saturating_sub(8)..shoulder_y {
                    if claw_x < w && y < h {
                        pixels[y][claw_x] = Some(metal_shine);
                    }
                }
            }
        }
        WeaponType::Gauntlet => {
            // Power fist / gauntlet
            for y in shoulder_y.saturating_sub(4)..shoulder_y + 4 {
                for gx in 0..5 {
                    let x = weapon_x + gx;
                    if x < w && y < h {
                        pixels[y][x] = Some(if gx < 2 { metal_light } else { metal });
                    }
                }
            }
        }
        WeaponType::Chakram => {
            // Circular throwing weapon
            for angle in 0..8 {
                let a = angle as f32 * std::f32::consts::PI / 4.0;
                let cx = weapon_x as f32 + a.cos() * 4.0;
                let cy = shoulder_y as f32 + a.sin() * 4.0;
                if (cx as usize) < w && (cy as usize) < h {
                    pixels[cy as usize][cx as usize] = Some(metal_shine);
                }
            }
        }
        WeaponType::Flamethrower => {
            // Flamethrower with fire stream
            for y in shoulder_y..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some([50, 50, 55]);
                    if weapon_x + 1 < w { pixels[y][weapon_x + 1] = Some([40, 40, 45]); }
                }
            }
            // Fire stream
            let flame_colors = [[255, 200, 50], [255, 150, 30], [255, 100, 20]];
            for y in 0..shoulder_y.min(h) {
                let fire_w = 3 + (y % 2);
                for fx in 0..fire_w {
                    let x = weapon_x.saturating_sub(fire_w / 2) + fx;
                    if x < w { pixels[y][x] = Some(flame_colors[(y + fx) % 3]); }
                }
            }
        }
        WeaponType::Grenade => {
            // Grenade in hand
            for y in shoulder_y.saturating_sub(3)..(shoulder_y + 3).min(h) {
                for gx in 0..3 {
                    let x = weapon_x + gx;
                    if x < w { pixels[y][x] = Some([50, 60, 40]); }
                }
            }
            // Pin
            if shoulder_y > 4 && weapon_x + 1 < w && shoulder_y - 4 < h {
                pixels[shoulder_y - 4][weapon_x + 1] = Some([200, 180, 50]);
            }
        }
        _ => {
            // Default: simple weapon line
            for y in 0..waist_y.min(h) {
                if weapon_x < w { pixels[y][weapon_x] = Some(metal); }
            }
        }
    }
}

/// Draw helmet based on style (legacy - kept for compatibility)
fn draw_helmet(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    desc: &SpriteDescription,
    cx: usize, w: usize, h: usize,
    head_height: usize, head_width: usize,
    armor: [u8; 3], armor_light: [u8; 3], armor_dark: [u8; 3],
    metal: [u8; 3], skin: [u8; 3], outline: [u8; 3], black: [u8; 3],
) {
    let face_y = head_height / 2;

    match desc.helmet {
        HelmetStyle::None => {
            // Just a head with face
            for y in 0..head_height {
                let width_at_y = head_width - (head_height.saturating_sub(y)) / 3;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(if x < cx { lighten(skin, 1.1) } else { skin });
                    }
                }
            }
            // Eyes
            if face_y < h && cx > 1 && cx + 1 < w {
                pixels[face_y][cx - 1] = Some(black);
                pixels[face_y][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Hood => {
            for y in 0..head_height {
                let width_at_y = head_width + y / 2;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(if y < 2 { armor_dark } else { armor });
                    }
                }
            }
            // Face opening
            if face_y < h {
                for x in cx.saturating_sub(2)..=(cx + 2).min(w - 1) {
                    pixels[face_y][x] = Some(skin);
                }
                if cx > 0 && cx + 1 < w {
                    pixels[face_y][cx - 1] = Some(black);
                    pixels[face_y][cx + 1] = Some(black);
                }
            }
        }
        HelmetStyle::Cap | HelmetStyle::Beret => {
            // Simple cap on head
            for y in 0..head_height {
                let is_cap = y < head_height / 2;
                let width_at_y = if is_cap { head_width + 2 } else { head_width };
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(if is_cap { armor } else { skin });
                    }
                }
            }
            if face_y + 1 < h && cx > 0 && cx + 1 < w {
                pixels[face_y + 1][cx - 1] = Some(black);
                pixels[face_y + 1][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Helm => {
            // Full helmet
            for y in 0..head_height {
                let width_at_y = head_width;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        if x == start_x || x == end_x - 1 || y == 0 {
                            pixels[y][x] = Some(outline);
                        } else {
                            pixels[y][x] = Some(if x < cx { armor_light } else { armor });
                        }
                    }
                }
            }
            // Visor slit
            if face_y < h {
                for x in cx.saturating_sub(2)..=(cx + 2).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
        }
        HelmetStyle::Crested => {
            // Roman/Greek style with crest
            for y in 0..head_height {
                let width_at_y = head_width;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(if x < cx { metal } else { darken(metal, 0.8) });
                    }
                }
            }
            // Crest (plume)
            let crest_height = head_height + 3;
            for y in 0..crest_height.min(h) {
                if cx < w {
                    pixels[y][cx] = Some([180, 30, 30]); // Red plume
                }
            }
            // Face
            if face_y < h {
                for x in cx.saturating_sub(2)..=(cx + 2).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
        }
        HelmetStyle::Horned => {
            // Viking style with horns
            for y in 0..head_height {
                let width_at_y = head_width;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(if x < cx { armor_light } else { armor });
                    }
                }
            }
            // Horns
            let horn_base = head_height / 3;
            for hy in 0..4.min(h) {
                let left_horn = cx.saturating_sub(head_width / 2 + hy);
                let right_horn = (cx + head_width / 2 + hy).min(w - 1);
                let y = horn_base.saturating_sub(hy);
                if y < h && left_horn < w {
                    pixels[y][left_horn] = Some([230, 220, 180]); // Bone color
                }
                if y < h && right_horn < w {
                    pixels[y][right_horn] = Some([230, 220, 180]);
                }
            }
            // Face
            if face_y < h {
                for x in cx.saturating_sub(2)..=(cx + 2).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
        }
        HelmetStyle::Crown => {
            // Royal crown
            for y in 0..head_height {
                let is_crown = y < head_height / 3;
                let width_at_y = if is_crown { head_width + 2 } else { head_width };
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        if is_crown {
                            pixels[y][x] = Some([255, 215, 0]); // Gold
                        } else {
                            pixels[y][x] = Some(skin);
                        }
                    }
                }
            }
            // Crown points
            for point in [cx.saturating_sub(3), cx, cx + 3] {
                if point < w && 0 < h {
                    pixels[0][point] = Some([255, 50, 50]); // Ruby
                }
            }
            // Face
            if face_y + 1 < h && cx > 0 && cx + 1 < w {
                pixels[face_y + 1][cx - 1] = Some(black);
                pixels[face_y + 1][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Turban => {
            // Middle Eastern turban
            for y in 0..head_height {
                let width_at_y = head_width + 2;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        if y < head_height * 2 / 3 {
                            pixels[y][x] = Some(armor); // Turban cloth
                        } else {
                            pixels[y][x] = Some(skin);
                        }
                    }
                }
            }
            // Turban gem
            if 1 < h && cx < w {
                pixels[1][cx] = Some([50, 200, 50]); // Emerald
            }
            // Face
            if face_y + 2 < h && cx > 0 && cx + 1 < w {
                pixels[face_y + 2][cx - 1] = Some(black);
                pixels[face_y + 2][cx + 1] = Some(black);
            }
        }
        HelmetStyle::Samurai => {
            // Japanese kabuto
            for y in 0..head_height {
                let width_at_y = head_width + (head_height - y) / 2;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        if y < head_height / 3 {
                            pixels[y][x] = Some(armor_dark); // Helmet dome
                        } else {
                            pixels[y][x] = Some(armor);
                        }
                    }
                }
            }
            // Mask
            if face_y < h {
                for x in cx.saturating_sub(3)..=(cx + 3).min(w - 1) {
                    pixels[face_y][x] = Some([150, 30, 30]); // Red mask
                }
            }
        }
        HelmetStyle::Tricorn => {
            // Colonial era hat
            for y in 0..head_height {
                let is_hat = y < head_height / 2;
                let width_at_y = if is_hat { head_width + 4 } else { head_width };
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        if is_hat {
                            pixels[y][x] = Some([30, 30, 40]); // Dark hat
                        } else {
                            pixels[y][x] = Some(skin);
                        }
                    }
                }
            }
            // Face
            if face_y + 1 < h && cx > 0 && cx + 1 < w {
                pixels[face_y + 1][cx - 1] = Some(black);
                pixels[face_y + 1][cx + 1] = Some(black);
            }
        }
        _ => {
            // Default: simple helmet
            for y in 0..head_height {
                let width_at_y = head_width;
                let start_x = cx.saturating_sub(width_at_y / 2);
                let end_x = (cx + width_at_y / 2 + 1).min(w);
                for x in start_x..end_x {
                    if x < w && y < h {
                        pixels[y][x] = Some(armor);
                    }
                }
            }
            if face_y < h {
                for x in cx.saturating_sub(2)..=(cx + 2).min(w - 1) {
                    pixels[face_y][x] = Some(black);
                }
            }
        }
    }
}

/// Draw shield on the left side
fn draw_shield(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    cx: usize, w: usize, h: usize,
    torso_top: usize, torso_bottom: usize, torso_width: usize,
    armor: [u8; 3], armor_dark: [u8; 3], metal: [u8; 3], outline: [u8; 3],
) {
    let shield_left = cx.saturating_sub(torso_width / 2 + 5);
    let shield_right = cx.saturating_sub(torso_width / 2 + 1);
    let shield_top = torso_top + 1;
    let shield_bottom = torso_bottom + 3;
    let shield_cx = (shield_left + shield_right) / 2;
    let shield_cy = (shield_top + shield_bottom) / 2;

    for y in shield_top..shield_bottom.min(h) {
        for x in shield_left..shield_right {
            if x < w && y < h {
                // Rounded shield shape
                let dy = (y as i32 - shield_cy as i32).abs();
                let max_dy = (shield_bottom - shield_top) as i32 / 2;
                if dy <= max_dy {
                    if x == shield_left {
                        pixels[y][x] = Some(outline);
                    } else if x == shield_cx && y == shield_cy {
                        pixels[y][x] = Some(metal); // Boss
                    } else {
                        pixels[y][x] = Some(if y < shield_cy { armor } else { armor_dark });
                    }
                }
            }
        }
    }
}

/// Draw weapon based on type
fn draw_weapon(
    pixels: &mut Vec<Vec<Option<[u8; 3]>>>,
    weapon: WeaponType,
    cx: usize, w: usize, h: usize,
    torso_top: usize, torso_bottom: usize, torso_width: usize,
    metal: [u8; 3], metal_light: [u8; 3], metal_dark: [u8; 3], outline: [u8; 3],
) {
    let weapon_x = (cx + torso_width / 2 + 3).min(w - 1);
    let wood = [101, 67, 33];

    match weapon {
        WeaponType::Sword | WeaponType::Katana | WeaponType::Saber => {
            // Blade
            let blade_top = 1;
            let blade_bottom = torso_bottom;
            for y in blade_top..blade_bottom.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(if y < blade_bottom / 2 { metal_light } else { metal });
                }
            }
            // Hilt
            if torso_top + 2 < h {
                for x in weapon_x.saturating_sub(1)..=(weapon_x + 1).min(w - 1) {
                    pixels[torso_top + 2][x] = Some(metal_dark);
                }
            }
        }
        WeaponType::Spear | WeaponType::Pike | WeaponType::Halberd | WeaponType::Naginata | WeaponType::Trident => {
            // Long shaft
            for y in 0..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                }
            }
            // Spearhead
            for y in 0..4.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(metal);
                }
            }
            // Trident has extra prongs
            if matches!(weapon, WeaponType::Trident) {
                if weapon_x > 0 && 1 < h {
                    pixels[1][weapon_x - 1] = Some(metal);
                }
                if weapon_x + 1 < w && 1 < h {
                    pixels[1][weapon_x + 1] = Some(metal);
                }
            }
        }
        WeaponType::Axe | WeaponType::Flail | WeaponType::Mace | WeaponType::Club => {
            // Handle
            for y in torso_top..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                }
            }
            // Axe head
            let head_y = torso_top.saturating_sub(2);
            for y in head_y..torso_top {
                for x in weapon_x..=(weapon_x + 3).min(w - 1) {
                    if y < h {
                        pixels[y][x] = Some(metal);
                    }
                }
            }
        }
        WeaponType::Bow | WeaponType::Crossbow => {
            // Bow curve
            let bow_top = torso_top.saturating_sub(3);
            let bow_bottom = torso_bottom + 2;
            let bow_mid = (bow_top + bow_bottom) / 2;
            for y in bow_top..bow_bottom.min(h) {
                let curve = if y < bow_mid {
                    (bow_mid - y) / 2
                } else {
                    (y - bow_mid) / 2
                };
                let x = weapon_x.saturating_sub(curve);
                if x < w {
                    pixels[y][x] = Some(wood);
                }
            }
            // String
            for y in bow_top..bow_bottom.min(h) {
                if weapon_x + 1 < w {
                    pixels[y][weapon_x + 1] = Some([200, 180, 160]);
                }
            }
        }
        WeaponType::Musket | WeaponType::Rifle => {
            // Long barrel
            for y in 0..torso_bottom.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(outline);
                }
            }
            // Stock
            for y in torso_top..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                }
            }
        }
        WeaponType::Pistol => {
            // Short barrel
            for y in torso_top.saturating_sub(2)..torso_top + 3 {
                if y < h && weapon_x < w {
                    pixels[y][weapon_x] = Some(outline);
                }
            }
        }
        WeaponType::Staff | WeaponType::Torch => {
            // Wooden staff
            for y in 0..h {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(wood);
                }
            }
            // Torch flame or staff gem
            if matches!(weapon, WeaponType::Torch) {
                for y in 0..3.min(h) {
                    if weapon_x < w {
                        pixels[y][weapon_x] = Some([255, 150, 50]); // Flame
                    }
                    if weapon_x > 0 && y == 1 {
                        pixels[y][weapon_x - 1] = Some([255, 200, 100]);
                    }
                }
            } else {
                if 0 < h && weapon_x < w {
                    pixels[0][weapon_x] = Some([100, 200, 255]); // Magic gem
                }
            }
        }
        WeaponType::Stone => {
            // Thrown rock
            let rock_y = torso_top;
            for dy in 0..2 {
                for dx in 0..2 {
                    let x = weapon_x + dx;
                    let y = rock_y + dy;
                    if x < w && y < h {
                        pixels[y][x] = Some([120, 110, 100]);
                    }
                }
            }
        }
        _ => {
            // Default: simple weapon line
            for y in 0..torso_bottom.min(h) {
                if weapon_x < w {
                    pixels[y][weapon_x] = Some(metal);
                }
            }
        }
    }
}

/// Darken a color by a factor (0.0-1.0)
fn darken(color: [u8; 3], factor: f32) -> [u8; 3] {
    [
        (color[0] as f32 * factor) as u8,
        (color[1] as f32 * factor) as u8,
        (color[2] as f32 * factor) as u8,
    ]
}

/// Lighten a color by a factor (1.0+)
fn lighten(color: [u8; 3], factor: f32) -> [u8; 3] {
    [
        (color[0] as f32 * factor).min(255.0) as u8,
        (color[1] as f32 * factor).min(255.0) as u8,
        (color[2] as f32 * factor).min(255.0) as u8,
    ]
}

/// Generate a quadruped sprite
fn generate_quadruped(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;

    // Body (horizontal oval)
    let body_top = h as usize / 3;
    let body_bottom = 2 * h as usize / 3;
    let body_left = w as usize / 6;
    let body_right = 5 * w as usize / 6;

    for y in body_top..body_bottom {
        for x in body_left..body_right {
            pixels[y][x] = Some(desc.primary_color);
        }
    }

    // Head (left side)
    let head_size = h as usize / 4;
    for y in (body_top - head_size / 2)..(body_top + head_size) {
        for x in 0..(body_left + head_size) {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Legs
    let leg_width = w as usize / 8;
    for y in body_bottom..h as usize {
        // Front legs
        for x in (body_left)..(body_left + leg_width) {
            if x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
        // Back legs
        for x in (body_right - leg_width)..body_right {
            if x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eye
    let eye_y = body_top;
    let eye_x = body_left / 2;
    if eye_y < h as usize && eye_x < w as usize {
        pixels[eye_y][eye_x] = Some(desc.accent_color);
    }
}

/// Generate a flying sprite (with wings)
fn generate_flying(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as usize / 2;
    let cy = h as usize / 2;

    // Body (small oval in center)
    let body_w = w as usize / 4;
    let body_h = h as usize / 3;
    for y in (cy - body_h / 2)..(cy + body_h / 2) {
        for x in (cx - body_w / 2)..(cx + body_w / 2) {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Wings (triangular)
    // Left wing
    let wing_y_start = cy.saturating_sub(body_h);
    let wing_y_end = cy + body_h / 2;
    for y in wing_y_start..wing_y_end {
        let wing_extent = ((cy + body_h / 2) as i32 - y as i32).abs() as usize;
        let wing_min_x = (cx.saturating_sub(body_w / 2)).saturating_sub(wing_extent + 2);
        for x in 0..cx.saturating_sub(body_w / 2) {
            if y < h as usize && x < w as usize && x > wing_min_x {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Right wing
    for y in wing_y_start..wing_y_end {
        let wing_extent = ((cy + body_h / 2) as i32 - y as i32).abs() as usize;
        let wing_max_x = (cx + body_w / 2 + wing_extent + 2).min(w as usize);
        for x in (cx + body_w / 2)..w as usize {
            if y < h as usize && x < wing_max_x {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eyes
    if cy > 0 && cx > 0 {
        let eye_y = cy - 1;
        if eye_y < h as usize {
            if cx > 0 && cx - 1 < w as usize {
                pixels[eye_y][cx - 1] = Some(desc.accent_color);
            }
            if cx + 1 < w as usize {
                pixels[eye_y][cx + 1] = Some(desc.accent_color);
            }
        }
    }
}

/// Generate a serpent sprite
fn generate_serpent(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cy = h as usize / 2;
    let thickness = (h as usize / 4).max(2);

    // Wavy body
    for x in 0..w as usize {
        let wave = ((x as f32 * 0.5).sin() * (h as f32 / 4.0)) as i32;
        let y_center = (cy as i32 + wave) as usize;

        for dy in 0..thickness {
            let y = y_center + dy;
            if y < h as usize {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Head (larger at start)
    let head_size = thickness + 2;
    for y in (cy.saturating_sub(head_size / 2))..(cy + head_size / 2) {
        for x in 0..head_size {
            if y < h as usize && x < w as usize {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Eye
    if cy < h as usize && 2 < w as usize {
        pixels[cy][2] = Some(desc.accent_color);
    }
}

/// Generate a geometric sprite
fn generate_geometric(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;
    let cx = w as usize / 2;
    let cy = h as usize / 2;

    // Diamond shape
    let size = w.min(h) as usize / 2;

    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as i32 - cx as i32).abs();
            let dy = (y as i32 - cy as i32).abs();

            if dx + dy <= size as i32 {
                pixels[y][x] = Some(desc.primary_color);
            }
        }
    }

    // Inner diamond with secondary color
    let inner_size = size / 2;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = (x as i32 - cx as i32).abs();
            let dy = (y as i32 - cy as i32).abs();

            if dx + dy <= inner_size as i32 {
                pixels[y][x] = Some(desc.secondary_color);
            }
        }
    }

    // Center dot
    if cy < h as usize && cx < w as usize {
        pixels[cy][cx] = Some(desc.accent_color);
    }
}

/// Apply pattern overlay to pixels
fn apply_pattern(pixels: &mut Vec<Vec<Option<[u8; 3]>>>, desc: &SpriteDescription) {
    let (w, h) = desc.size;

    match desc.pattern {
        SpritePattern::Solid => {
            // Already solid, no changes
        }
        SpritePattern::Striped => {
            for y in 0..h as usize {
                if y % 3 == 0 {
                    for x in 0..w as usize {
                        if pixels[y][x].is_some() {
                            pixels[y][x] = Some(desc.secondary_color);
                        }
                    }
                }
            }
        }
        SpritePattern::Spotted => {
            for y in (0..h as usize).step_by(3) {
                for x in (0..w as usize).step_by(3) {
                    if pixels[y][x].is_some() {
                        pixels[y][x] = Some(desc.accent_color);
                    }
                }
            }
        }
        SpritePattern::Gradient => {
            for y in 0..h as usize {
                let factor = y as f32 / h as f32;
                for x in 0..w as usize {
                    if let Some(color) = pixels[y][x] {
                        let blended = blend_colors(color, desc.secondary_color, factor);
                        pixels[y][x] = Some(blended);
                    }
                }
            }
        }
        SpritePattern::Checkered => {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    if (x + y) % 2 == 0 && pixels[y][x].is_some() {
                        pixels[y][x] = Some(desc.secondary_color);
                    }
                }
            }
        }
    }
}

/// Blend two colors
fn blend_colors(c1: [u8; 3], c2: [u8; 3], factor: f32) -> [u8; 3] {
    [
        ((c1[0] as f32 * (1.0 - factor) + c2[0] as f32 * factor) as u8),
        ((c1[1] as f32 * (1.0 - factor) + c2[1] as f32 * factor) as u8),
        ((c1[2] as f32 * (1.0 - factor) + c2[2] as f32 * factor) as u8),
    ]
}

/// Manages all rendered sprites on screen
pub struct SpriteManager {
    sprites: Vec<RenderedSprite>,
    max_sprites: usize,
}

impl SpriteManager {
    pub fn new(max_sprites: usize) -> Self {
        Self {
            sprites: Vec::new(),
            max_sprites,
        }
    }

    /// Add a new sprite
    pub fn add(&mut self, sprite: RenderedSprite) {
        if self.sprites.len() >= self.max_sprites {
            self.sprites.remove(0); // Remove oldest
        }
        self.sprites.push(sprite);
    }

    /// Render all sprites
    pub fn render(&self) {
        for sprite in &self.sprites {
            sprite.render();
        }
    }

    /// Get sprite count
    pub fn count(&self) -> usize {
        self.sprites.len()
    }

    /// Clear all sprites
    pub fn clear(&mut self) {
        self.sprites.clear();
    }
}
