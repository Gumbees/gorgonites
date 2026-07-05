//! Player interaction: selection, orders, building placement, and the
//! Rise of Nations-style HUD (resource strip with live rates, selection
//! panel with production buttons, minimap).

use macroquad::prelude::*;

use crate::rendering::GameCamera;
use crate::ui::ui_button_detail;

use super::entities::*;
use super::mapgen::{Terrain, MAP_H, MAP_W, TILE};
use super::render_world::{draw_placement_ghost, draw_selection};
use super::world::{age_up_cost, World};
use crate::systems::rts::{commerce_cap, Resource};

const PLAYER: usize = 0;
const TOP_BAR_H: f32 = 56.0;
const MINIMAP_SIZE: f32 = 176.0;
const PANEL_W: f32 = 470.0;
const PANEL_H: f32 = 148.0;

pub struct Interface {
    pub selection: Vec<Id>,
    pub selected_building: Option<Id>,
    pub placing: Option<BuildingKind>,
    drag_start: Option<Vec2>,
    toast: Option<(String, f32)>,
    minimap: Option<Texture2D>,
    minimap_timer: f32,
}

impl Default for Interface {
    fn default() -> Self {
        Self {
            selection: Vec::new(),
            selected_building: None,
            placing: None,
            drag_start: None,
            toast: None,
            minimap: None,
            minimap_timer: 0.0,
        }
    }
}

impl Interface {
    pub fn new() -> Self {
        Self::default()
    }

    fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), 2.5));
    }

    fn panel_rect(&self) -> Rect {
        Rect::new(10.0, screen_height() - PANEL_H - 10.0, PANEL_W, PANEL_H)
    }

    fn minimap_rect(&self) -> Rect {
        Rect::new(
            screen_width() - MINIMAP_SIZE - 10.0,
            screen_height() - MINIMAP_SIZE - 10.0,
            MINIMAP_SIZE,
            MINIMAP_SIZE,
        )
    }

    fn pointer_over_ui(&self) -> bool {
        let m = Vec2::from(mouse_position());
        if m.y <= TOP_BAR_H {
            return true;
        }
        if self.minimap_rect().contains(m) {
            return true;
        }
        let showing_panel = !self.selection.is_empty() || self.selected_building.is_some();
        showing_panel && self.panel_rect().contains(m)
    }

    /// Drop references to entities that died since last frame.
    fn prune(&mut self, world: &World) {
        self.selection.retain(|id| world.unit(*id).is_some());
        if let Some(id) = self.selected_building {
            if world.building(id).is_none() {
                self.selected_building = None;
            }
        }
    }

    pub fn handle_input(&mut self, world: &mut World, camera: &GameCamera, dt: f32) {
        self.prune(world);
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
            if *t <= 0.0 {
                self.toast = None;
            }
        }

        let mouse_screen = Vec2::from(mouse_position());
        let mouse_world = camera.screen_to_world(mouse_screen);

        // Minimap click: jump the camera.
        let mm = self.minimap_rect();
        if mm.contains(mouse_screen) && is_mouse_button_down(MouseButton::Left) {
            return; // camera jump handled by caller via minimap_world_target
        }

        if is_key_pressed(KeyCode::Escape) && self.placing.is_some() {
            self.placing = None;
            return;
        }

        // Building placement mode.
        if let Some(kind) = self.placing {
            if is_mouse_button_pressed(MouseButton::Right) {
                self.placing = None;
                return;
            }
            if is_mouse_button_pressed(MouseButton::Left) && !self.pointer_over_ui() {
                let fp = kind.footprint();
                let tile = (
                    (mouse_world.x / TILE).floor() as i32 - fp / 2,
                    (mouse_world.y / TILE).floor() as i32 - fp / 2,
                );
                match world.can_place(PLAYER, kind, tile) {
                    Ok(()) => {
                        let cost = kind.cost();
                        if world.nations[PLAYER].stockpile.pay(&cost) {
                            world.place_building(PLAYER, kind, tile, false);
                            self.placing = None;
                        } else {
                            self.toast(format!("Not enough resources ({})", cost.describe()));
                        }
                    }
                    Err(e) => self.toast(e),
                }
            }
            return;
        }

        if self.pointer_over_ui() {
            self.drag_start = None;
            return;
        }

        // Left: select (click or drag box).
        if is_mouse_button_pressed(MouseButton::Left) {
            self.drag_start = Some(mouse_screen);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(start) = self.drag_start.take() {
                if (mouse_screen - start).length() > 8.0 {
                    self.box_select(world, camera, start, mouse_screen);
                } else {
                    self.point_select(world, mouse_world);
                }
            }
        }

        // Right: context order.
        if is_mouse_button_pressed(MouseButton::Right) && !self.selection.is_empty() {
            self.issue_order(world, mouse_world);
        }
    }

    /// If the player is dragging on the minimap, where should the camera go?
    pub fn minimap_world_target(&self) -> Option<Vec2> {
        let mm = self.minimap_rect();
        let m = Vec2::from(mouse_position());
        if mm.contains(m) && is_mouse_button_down(MouseButton::Left) {
            let fx = (m.x - mm.x) / mm.w;
            let fy = (m.y - mm.y) / mm.h;
            Some(vec2(fx * MAP_W as f32 * TILE, fy * MAP_H as f32 * TILE))
        } else {
            None
        }
    }

    fn box_select(&mut self, world: &World, camera: &GameCamera, a: Vec2, b: Vec2) {
        let wa = camera.screen_to_world(a);
        let wb = camera.screen_to_world(b);
        let min = wa.min(wb);
        let max = wa.max(wb);
        self.selection = world
            .units
            .iter()
            .filter(|u| u.nation == PLAYER)
            .filter(|u| {
                u.pos.x >= min.x && u.pos.x <= max.x && u.pos.y >= min.y && u.pos.y <= max.y
            })
            .map(|u| u.id)
            .collect();
        self.selected_building = None;
    }

    fn point_select(&mut self, world: &World, at: Vec2) {
        // Own unit first.
        let mut best: Option<(Id, f32)> = None;
        for u in &world.units {
            if u.nation != PLAYER {
                continue;
            }
            let d = (u.pos - at).length();
            if d < u.radius() + 8.0 && best.map_or(true, |(_, bd)| d < bd) {
                best = Some((u.id, d));
            }
        }
        if let Some((id, _)) = best {
            self.selection = vec![id];
            self.selected_building = None;
            return;
        }
        // Then any building whose footprint contains the point.
        for b in &world.buildings {
            let he = b.half_extent();
            if (at.x - b.pos.x).abs() <= he && (at.y - b.pos.y).abs() <= he {
                self.selected_building = Some(b.id);
                self.selection.clear();
                return;
            }
        }
        self.selection.clear();
        self.selected_building = None;
    }

    fn issue_order(&mut self, world: &mut World, at: Vec2) {
        // Enemy unit under cursor?
        let enemy_unit = world
            .units
            .iter()
            .filter(|u| u.nation != PLAYER)
            .find(|u| (u.pos - at).length() < u.radius() + 8.0)
            .map(|u| u.id);
        // Building under cursor?
        let building = world
            .buildings
            .iter()
            .find(|b| {
                let he = b.half_extent();
                (at.x - b.pos.x).abs() <= he && (at.y - b.pos.y).abs() <= he
            })
            .map(|b| (b.id, b.nation, b.kind.output().is_some()));

        let ids: Vec<Id> = self.selection.clone();
        if let Some(target) = enemy_unit {
            for id in &ids {
                if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                    u.order = if u.kind.is_military() {
                        Order::AttackUnit(target)
                    } else {
                        Order::Move { dest: at, aggro: false }
                    };
                }
            }
            return;
        }
        if let Some((bid, b_nation, has_slots)) = building {
            if b_nation != PLAYER {
                for id in &ids {
                    if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                        if u.kind.is_military() {
                            u.order = Order::AttackBuilding(bid);
                        }
                    }
                }
                return;
            }
            if has_slots {
                // Citizens report for work; escorts just move nearby.
                for id in &ids {
                    if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                        u.order = if u.kind == UnitKind::Citizen {
                            Order::Work { building: bid }
                        } else {
                            Order::Move { dest: at, aggro: true }
                        };
                    }
                }
                return;
            }
        }
        // Plain move in loose formation; military stays alert on the way.
        for (i, id) in ids.iter().enumerate() {
            let offset = vec2(
                ((i % 5) as f32 - 2.0) * 22.0,
                ((i / 5) as f32) * 22.0,
            );
            if let Some(u) = world.units.iter_mut().find(|u| u.id == *id) {
                let aggro = u.kind.is_military();
                u.order = Order::Move { dest: at + offset, aggro };
            }
        }
    }

    // -- drawing --------------------------------------------------------------

    pub fn draw(&mut self, world: &mut World, camera: &GameCamera, dt: f32) {
        draw_selection(world, camera, &self.selection, self.selected_building);

        // Drag box.
        if let Some(start) = self.drag_start {
            let m = Vec2::from(mouse_position());
            let min = start.min(m);
            let size = (start - m).abs();
            draw_rectangle(min.x, min.y, size.x, size.y, Color::new(0.7, 0.85, 0.7, 0.15));
            draw_rectangle_lines(min.x, min.y, size.x, size.y, 1.5, Color::new(0.8, 0.95, 0.8, 0.7));
        }

        // Placement ghost.
        if let Some(kind) = self.placing {
            let mw = camera.screen_to_world(Vec2::from(mouse_position()));
            let fp = kind.footprint();
            let tile = (
                (mw.x / TILE).floor() as i32 - fp / 2,
                (mw.y / TILE).floor() as i32 - fp / 2,
            );
            draw_placement_ghost(world, camera, PLAYER, kind, tile);
        }

        self.draw_top_bar(world);
        self.draw_selection_panel(world);
        self.draw_minimap(world, camera, dt);
        self.draw_capital_warnings(world);

        if let Some((msg, t)) = &self.toast {
            let alpha = (*t / 0.6).min(1.0);
            let dims = measure_text(msg, None, 18, 1.0);
            let x = (screen_width() - dims.width) / 2.0;
            let y = screen_height() - PANEL_H - 40.0;
            draw_rectangle(
                x - 10.0,
                y - 20.0,
                dims.width + 20.0,
                30.0,
                Color::new(0.05, 0.05, 0.05, 0.7 * alpha),
            );
            draw_text(msg, x, y, 18.0, Color::new(0.95, 0.85, 0.6, alpha));
        }
    }

    fn draw_top_bar(&self, world: &World) {
        let nation = &world.nations[PLAYER];
        draw_rectangle(0.0, 0.0, screen_width(), TOP_BAR_H, Color::new(0.07, 0.07, 0.06, 0.92));
        draw_line(0.0, TOP_BAR_H, screen_width(), TOP_BAR_H, 1.5, Color::new(0.3, 0.29, 0.25, 1.0));

        let icon_colors = [
            Color::from_rgba(168, 148, 82, 255),  // food
            Color::from_rgba(122, 96, 60, 255),   // timber
            Color::from_rgba(148, 148, 152, 255), // metal
            Color::from_rgba(196, 170, 84, 255),  // wealth
            Color::from_rgba(110, 140, 180, 255), // knowledge
            Color::from_rgba(52, 50, 48, 255),    // oil
        ];

        let mut x = 12.0;
        for r in Resource::ALL {
            let idx = r.index();
            draw_rectangle(x, 10.0, 16.0, 16.0, icon_colors[idx]);
            draw_rectangle_lines(x, 10.0, 16.0, 16.0, 1.0, Color::new(0.0, 0.0, 0.0, 0.6));
            let amount = format!("{}", nation.stockpile.get(r) as i64);
            draw_text(&amount, x + 21.0, 23.0, 16.0, WHITE);
            let rate = nation.income[idx];
            let rate_txt = format!("+{:.1}", rate);
            let rate_col = if nation.capped[idx] {
                Color::from_rgba(220, 120, 90, 255)
            } else {
                Color::from_rgba(150, 160, 130, 255)
            };
            draw_text(&rate_txt, x + 21.0, 40.0, 12.0, rate_col);
            x += 92.0;
        }

        // Population.
        let pop = format!("Pop {}/{}", nation.pop, nation.pop_cap);
        draw_text(&pop, x + 8.0, 23.0, 16.0, WHITE);
        let cap = format!("cap {:.0}/s", commerce_cap(nation.age));
        draw_text(&cap, x + 8.0, 40.0, 12.0, Color::from_rgba(130, 130, 120, 255));

        // Age + clock on the right.
        let era = crate::game::Era::from_index(nation.age);
        let age_txt = format!("{}  (Age {})", era.display_name(), nation.age + 1);
        let dims = measure_text(&age_txt, None, 18, 1.0);
        draw_text(&age_txt, screen_width() - dims.width - 120.0, 24.0, 18.0, Color::from_rgba(220, 210, 170, 255));
        let mins = (world.game_time / 60.0) as i32;
        let secs = (world.game_time % 60.0) as i32;
        let clock = format!("{:02}:{:02}", mins, secs);
        draw_text(&clock, screen_width() - 70.0, 24.0, 18.0, WHITE);

        // Enemy status line.
        let enemy = &world.nations[1];
        let status = format!(
            "{}: Age {}  kills {} | Your kills: {}",
            enemy.name,
            enemy.age + 1,
            enemy.kills,
            nation.kills
        );
        let sdims = measure_text(&status, None, 12, 1.0);
        draw_text(
            &status,
            screen_width() - sdims.width - 12.0,
            44.0,
            12.0,
            Color::from_rgba(150, 140, 130, 255),
        );
    }

    fn draw_minimap(&mut self, world: &World, camera: &GameCamera, dt: f32) {
        self.minimap_timer -= dt;
        if self.minimap.is_none() || self.minimap_timer <= 0.0 {
            self.minimap_timer = 2.0;
            self.minimap = Some(build_minimap_texture(world));
        }
        let mm = self.minimap_rect();
        draw_rectangle(mm.x - 3.0, mm.y - 3.0, mm.w + 6.0, mm.h + 6.0, Color::new(0.07, 0.07, 0.06, 0.92));
        if let Some(tex) = &self.minimap {
            draw_texture_ex(
                tex,
                mm.x,
                mm.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(mm.w, mm.h)),
                    ..Default::default()
                },
            );
        }
        // Live unit dots.
        for u in &world.units {
            let fx = u.pos.x / (MAP_W as f32 * TILE);
            let fy = u.pos.y / (MAP_H as f32 * TILE);
            let c = world.nations[u.nation].color;
            draw_rectangle(mm.x + fx * mm.w - 1.0, mm.y + fy * mm.h - 1.0, 2.0, 2.0, c);
        }
        // Camera view rectangle.
        let tl = camera.screen_to_world(vec2(0.0, 0.0));
        let br = camera.screen_to_world(vec2(screen_width(), screen_height()));
        let rx = mm.x + tl.x / (MAP_W as f32 * TILE) * mm.w;
        let ry = mm.y + tl.y / (MAP_H as f32 * TILE) * mm.h;
        let rw = (br.x - tl.x) / (MAP_W as f32 * TILE) * mm.w;
        let rh = (br.y - tl.y) / (MAP_H as f32 * TILE) * mm.h;
        draw_rectangle_lines(rx, ry, rw, rh, 1.0, Color::new(0.9, 0.9, 0.85, 0.8));
        draw_rectangle_lines(mm.x, mm.y, mm.w, mm.h, 1.5, Color::new(0.3, 0.29, 0.25, 1.0));
    }

    fn draw_capital_warnings(&self, world: &World) {
        let player = &world.nations[PLAYER];
        if let Some(t) = player.capital_timer {
            let msg = format!("CAPITAL LOST — nation falls in {:.0}s. Retake it!", t);
            let dims = measure_text(&msg, None, 26, 1.0);
            let x = (screen_width() - dims.width) / 2.0;
            let pulse = ((get_time() * 4.0).sin() * 0.25 + 0.75) as f32;
            draw_text(&msg, x, TOP_BAR_H + 40.0, 26.0, Color::new(0.9, 0.25, 0.2, pulse));
        }
        for n in world.nations.iter().skip(1) {
            if let Some(t) = n.capital_timer {
                let msg = format!("{} capital under your flag — {:.0}s to victory", n.name, t);
                let dims = measure_text(&msg, None, 20, 1.0);
                let x = (screen_width() - dims.width) / 2.0;
                draw_text(&msg, x, TOP_BAR_H + 70.0, 20.0, Color::new(0.65, 0.85, 0.55, 1.0));
            }
        }
    }
}

fn build_minimap_texture(world: &World) -> Texture2D {
    let mut image = Image::gen_image_color(MAP_W as u16, MAP_H as u16, BLACK);
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let base = match world.map.get(x, y) {
                Terrain::DeepWater => Color::from_rgba(26, 41, 58, 255),
                Terrain::Water => Color::from_rgba(43, 63, 82, 255),
                Terrain::Plains => Color::from_rgba(125, 116, 80, 255),
                Terrain::Grass => Color::from_rgba(96, 108, 66, 255),
                Terrain::Forest => Color::from_rgba(74, 88, 56, 255),
                Terrain::Hills => Color::from_rgba(118, 104, 82, 255),
                Terrain::Mountain => Color::from_rgba(104, 100, 96, 255),
            };
            let color = match world.tile_owner(x, y) {
                Some(o) => {
                    let n = world.nations[o as usize].color;
                    Color::new(
                        base.r * 0.55 + n.r * 0.45,
                        base.g * 0.55 + n.g * 0.45,
                        base.b * 0.55 + n.b * 0.45,
                        1.0,
                    )
                }
                None => base,
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }
    for b in &world.buildings {
        let (tx, ty) = b.tile;
        let c = world.nations[b.nation].color;
        for dy in 0..b.kind.footprint() {
            for dx in 0..b.kind.footprint() {
                let (px, py) = (tx + dx, ty + dy);
                if px >= 0 && py >= 0 && px < MAP_W && py < MAP_H {
                    image.set_pixel(px as u32, py as u32, c);
                }
            }
        }
    }
    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    tex
}

// ---------------------------------------------------------------------------
// Selection panel (free function to keep borrow scopes simple)
// ---------------------------------------------------------------------------

impl Interface {
    pub fn draw_selection_panel(&mut self, world: &mut World) {
        if self.selection.is_empty() && self.selected_building.is_none() {
            return;
        }
        let panel = self.panel_rect();
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, Color::new(0.07, 0.07, 0.06, 0.92));
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.5, Color::new(0.3, 0.29, 0.25, 1.0));

        if let Some(bid) = self.selected_building {
            self.panel_building(world, bid, panel);
            return;
        }
        self.panel_units(world, panel);
    }

    fn panel_building(&mut self, world: &mut World, bid: Id, panel: Rect) {
        let Some(b) = world.building(bid) else { return };
        let kind = b.kind;
        let nation = b.nation;
        let complete = b.is_complete();
        let hp = b.hp;
        let max_hp = b.max_hp;
        let queue: Vec<QueueItem> = b.queue.clone();
        let progress = b.queue_progress;
        let workers = world.workers_at(bid);
        let age = world.nations[nation].age;

        let title = if nation == PLAYER {
            kind.name().to_string()
        } else {
            format!("{} ({})", kind.name(), world.nations[nation].name)
        };
        draw_text(&title, panel.x + 12.0, panel.y + 22.0, 18.0, WHITE);
        draw_text(
            &format!("HP {:.0}/{:.0}", hp, max_hp),
            panel.x + 12.0,
            panel.y + 42.0,
            14.0,
            Color::from_rgba(170, 170, 160, 255),
        );
        if !complete {
            draw_text("Under construction...", panel.x + 12.0, panel.y + 60.0, 14.0, Color::from_rgba(200, 180, 120, 255));
            return;
        }
        if nation != PLAYER {
            return;
        }

        if let Some((resource, per_worker, slots)) = kind.output() {
            draw_text(
                &format!(
                    "Workers {}/{}  (+{:.1} {}/s) — right-click citizens here to staff",
                    workers.min(slots),
                    slots,
                    per_worker * workers.min(slots) as f32,
                    resource.display_name()
                ),
                panel.x + 12.0,
                panel.y + 62.0,
                13.0,
                Color::from_rgba(160, 170, 140, 255),
            );
        }

        // Production buttons.
        let bx = panel.x + 12.0;
        let by = panel.y + 76.0;
        let bw = 106.0;
        let bh = 40.0;
        match kind {
            BuildingKind::City => {
                let citizen_cost =
                    unit_ramped_cost(UnitKind::Citizen, world.count_units(PLAYER, UnitKind::Citizen));
                if ui_button_detail(
                    Rect::new(bx, by, bw, bh),
                    "Citizen",
                    &citizen_cost.describe(),
                    true,
                ) {
                    self.report(world.try_enqueue(bid, QueueItem::Unit(UnitKind::Citizen)));
                }
                if age < 7 {
                    let cost = age_up_cost(age + 1);
                    if ui_button_detail(
                        Rect::new(bx + bw + 8.0, by, bw + 30.0, bh),
                        "Advance Age",
                        &cost.describe(),
                        true,
                    ) {
                        self.report(world.try_enqueue(bid, QueueItem::AgeUp));
                    }
                }
            }
            BuildingKind::Barracks => {
                for (i, k) in UnitKind::MILITARY.iter().enumerate() {
                    let cost = unit_ramped_cost(*k, world.count_units(PLAYER, *k));
                    let rect = Rect::new(bx + i as f32 * (bw + 8.0), by, bw, bh);
                    if ui_button_detail(rect, unit_name(*k, age), &cost.describe(), true) {
                        self.report(world.try_enqueue(bid, QueueItem::Unit(*k)));
                    }
                }
            }
            _ => {}
        }

        // Queue readout.
        if !queue.is_empty() {
            let mut qx = bx;
            let qy = panel.y + 126.0;
            for (i, item) in queue.iter().enumerate() {
                let label = match item {
                    QueueItem::Unit(u) => unit_name(*u, age).to_string(),
                    QueueItem::AgeUp => "Age research".to_string(),
                };
                let text = if i == 0 {
                    let needed = match item {
                        QueueItem::Unit(u) => unit_stats(*u, age).train_time,
                        QueueItem::AgeUp => 25.0,
                    };
                    format!("{} {:.0}%", label, (progress / needed * 100.0).min(99.0))
                } else {
                    label
                };
                draw_text(&text, qx, qy, 13.0, Color::from_rgba(200, 200, 180, 255));
                qx += measure_text(&text, None, 13, 1.0).width + 16.0;
            }
        }
    }

    fn panel_units(&mut self, world: &mut World, panel: Rect) {
        let age = world.nations[PLAYER].age;
        let mut counts: Vec<(UnitKind, usize)> = Vec::new();
        for id in &self.selection {
            if let Some(u) = world.unit(*id) {
                if let Some(e) = counts.iter_mut().find(|(k, _)| *k == u.kind) {
                    e.1 += 1;
                } else {
                    counts.push((u.kind, 1));
                }
            }
        }
        let summary = counts
            .iter()
            .map(|(k, n)| format!("{} x{}", unit_name(*k, age), n))
            .collect::<Vec<_>>()
            .join("   ");
        draw_text(&summary, panel.x + 12.0, panel.y + 22.0, 16.0, WHITE);

        let has_citizen = counts.iter().any(|(k, _)| *k == UnitKind::Citizen);
        if !has_citizen {
            draw_text(
                "Right-click: move / attack. Enemy territory bleeds you — attrition.",
                panel.x + 12.0,
                panel.y + 44.0,
                13.0,
                Color::from_rgba(160, 160, 150, 255),
            );
            return;
        }

        draw_text(
            "Build (inside your borders):",
            panel.x + 12.0,
            panel.y + 44.0,
            13.0,
            Color::from_rgba(160, 170, 140, 255),
        );
        let bw = 106.0;
        let bh = 40.0;
        for (i, kind) in BuildingKind::BUILDABLE.iter().enumerate() {
            let col = i % 4;
            let row = i / 4;
            let rect = Rect::new(
                panel.x + 12.0 + col as f32 * (bw + 8.0),
                panel.y + 54.0 + row as f32 * (bh + 6.0),
                bw,
                bh,
            );
            let unlocked = age >= kind.min_age();
            let detail = if unlocked {
                kind.cost().describe()
            } else {
                format!("Age {}+", kind.min_age() + 1)
            };
            if ui_button_detail(rect, kind.name(), &detail, unlocked) {
                self.placing = Some(*kind);
            }
        }
    }

    fn report(&mut self, result: Result<(), String>) {
        if let Err(e) = result {
            self.toast(e);
        }
    }
}
