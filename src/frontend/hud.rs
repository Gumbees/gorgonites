//! Bevy-UI HUD: main menu, the in-game resource bar, a context action bar
//! (train / build / advance age) driven by the current selection, transient
//! toasts, capital-loss warnings, and the win/lose screen.

use bevy::prelude::*;

use crate::game::{
    age_up_cost, unit_name, unit_ramped_cost, BuildingKind, Era, QueueItem, UnitKind,
};
use crate::systems::rts::Resource as GameResource;

use super::input::{PlacementMode, Selection, Toast};
use super::sim::Sim;
use super::AppState;

const PLAYER: usize = Sim::PLAYER;

// --- markers ---------------------------------------------------------------

#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct GameOverRoot;
#[derive(Component)]
struct TopBarText;
#[derive(Component)]
struct ToastText;
#[derive(Component)]
struct CapitalWarnText;

/// An action-bar button and the command it fires.
#[derive(Component, Clone, Copy)]
struct ActionButton(Action);

#[derive(Clone, Copy)]
enum Action {
    TrainCitizen,
    Train(UnitKind),
    AgeUp,
    Build(BuildingKind),
}

/// Live handles to the action-bar buttons so we can rebuild them on change.
#[derive(Resource, Default)]
struct ActionBar {
    buttons: Vec<Entity>,
    signature: String,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionBar>()
            .add_systems(Startup, spawn_persistent_hud)
            .add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn::<MenuRoot>)
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over)
            .add_systems(OnExit(AppState::GameOver), despawn::<GameOverRoot>)
            .add_systems(
                Update,
                (menu_start.run_if(in_state(AppState::Menu)),
                 game_over_restart.run_if(in_state(AppState::GameOver))),
            )
            .add_systems(
                Update,
                (
                    update_top_bar,
                    update_toast,
                    update_capital_warning,
                    rebuild_action_bar,
                    handle_action_clicks,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), clear_action_bar);
    }
}

fn despawn<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// --- menu ------------------------------------------------------------------

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.05, 0.75)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("GORGONITES"),
                TextFont { font_size: 76.0, ..Default::default() },
                TextColor(Color::srgb(0.92, 0.9, 0.84)),
            ));
            p.spawn((
                Text::new("Rise of Nations, in real 3D  \u{2014}  borders, attrition, eight ages"),
                TextFont { font_size: 22.0, ..Default::default() },
                TextColor(Color::srgb(0.6, 0.62, 0.6)),
                Node { margin: UiRect::top(Val::Px(14.0)), ..Default::default() },
            ));
            for line in [
                "Left-drag select   \u{2022}   Right-click move / attack / assign workers",
                "WASD pan   \u{2022}   Q/E rotate   \u{2022}   scroll zoom",
                "Build inside your borders. Enemy soil bleeds your troops.",
                "Capture and hold the enemy capital to win.",
            ] {
                p.spawn((
                    Text::new(line),
                    TextFont { font_size: 16.0, ..Default::default() },
                    TextColor(Color::srgb(0.68, 0.68, 0.64)),
                    Node { margin: UiRect::top(Val::Px(6.0)), ..Default::default() },
                ));
            }
            p.spawn((
                Text::new("Press SPACE to begin"),
                TextFont { font_size: 24.0, ..Default::default() },
                TextColor(Color::srgb(0.9, 0.85, 0.55)),
                Node { margin: UiRect::top(Val::Px(28.0)), ..Default::default() },
            ));
        });
}

fn menu_start(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next.set(AppState::Playing);
    }
}

// --- game over -------------------------------------------------------------

fn spawn_game_over(mut commands: Commands, sim: Res<Sim>) {
    let victory = sim.world.winner == Some(PLAYER);
    let (title, color) = if victory {
        ("VICTORY", Color::srgb(0.6, 0.85, 0.5))
    } else {
        ("DEFEAT", Color::srgb(0.85, 0.3, 0.25))
    };
    let summary = format!(
        "Reached the {} \u{2014} {} kills in {:.0} minutes",
        Era::from_index(sim.world.nations[PLAYER].age).display_name(),
        sim.world.nations[PLAYER].kills,
        sim.world.game_time / 60.0,
    );
    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.82)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(title),
                TextFont { font_size: 72.0, ..Default::default() },
                TextColor(color),
            ));
            p.spawn((
                Text::new(summary),
                TextFont { font_size: 22.0, ..Default::default() },
                TextColor(Color::srgb(0.8, 0.8, 0.75)),
                Node { margin: UiRect::top(Val::Px(12.0)), ..Default::default() },
            ));
            p.spawn((
                Text::new("Press SPACE for the main menu"),
                TextFont { font_size: 20.0, ..Default::default() },
                TextColor(Color::srgb(0.7, 0.7, 0.66)),
                Node { margin: UiRect::top(Val::Px(24.0)), ..Default::default() },
            ));
        });
}

fn game_over_restart(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next.set(AppState::Menu);
    }
}

// --- persistent in-game HUD ------------------------------------------------

fn spawn_persistent_hud(mut commands: Commands) {
    // Top resource bar.
    commands.spawn((
        TopBarText,
        Text::new(""),
        TextFont { font_size: 18.0, ..Default::default() },
        TextColor(Color::srgb(0.9, 0.9, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(12.0),
            ..Default::default()
        },
    ));
    // Capital warning (centred, below the bar).
    commands.spawn((
        CapitalWarnText,
        Text::new(""),
        TextFont { font_size: 24.0, ..Default::default() },
        TextColor(Color::srgb(0.9, 0.3, 0.25)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Percent(28.0),
            ..Default::default()
        },
    ));
    // Toast (bottom centre-left).
    commands.spawn((
        ToastText,
        Text::new(""),
        TextFont { font_size: 18.0, ..Default::default() },
        TextColor(Color::srgb(0.95, 0.85, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(96.0),
            left: Val::Px(16.0),
            ..Default::default()
        },
    ));
}

fn update_top_bar(sim: Res<Sim>, mut q: Query<&mut Text, With<TopBarText>>) {
    let Ok(mut text) = q.single_mut() else { return };
    let n = &sim.world.nations[PLAYER];
    let mut parts = Vec::new();
    for r in GameResource::ALL {
        let i = r.index();
        let flag = if n.capped[i] { "!" } else { "" };
        parts.push(format!(
            "{} {}({:+.1}{})",
            r.display_name().chars().next().unwrap(),
            n.stockpile.get(r) as i64,
            n.income[i],
            flag
        ));
    }
    let era = Era::from_index(n.age);
    text.0 = format!(
        "{}     Pop {}/{}     {} (Age {})     {:02}:{:02}     kills {} vs {}",
        parts.join("  "),
        n.pop,
        n.pop_cap,
        era.display_name(),
        n.age + 1,
        (sim.world.game_time / 60.0) as i32,
        (sim.world.game_time % 60.0) as i32,
        n.kills,
        sim.world.nations.get(1).map(|e| e.kills).unwrap_or(0),
    );
}

fn update_toast(toast: Res<Toast>, mut q: Query<&mut Text, With<ToastText>>) {
    if let Ok(mut text) = q.single_mut() {
        text.0 = if toast.timer > 0.0 {
            toast.text.clone()
        } else {
            String::new()
        };
    }
}

fn update_capital_warning(sim: Res<Sim>, mut q: Query<&mut Text, With<CapitalWarnText>>) {
    let Ok(mut text) = q.single_mut() else { return };
    if let Some(t) = sim.world.nations[PLAYER].capital_timer {
        text.0 = format!("CAPITAL LOST \u{2014} nation falls in {:.0}s. Retake it!", t);
    } else if let Some(t) = sim
        .world
        .nations
        .iter()
        .skip(1)
        .find_map(|e| e.capital_timer)
    {
        text.0 = format!("Enemy capital under your flag \u{2014} {:.0}s to victory", t);
    } else {
        text.0 = String::new();
    }
}

// --- context action bar ----------------------------------------------------

/// A short string describing what the action bar should show; when it changes
/// we rebuild the buttons.
fn action_signature(sim: &Sim, selection: &Selection) -> String {
    let age = sim.world.nations[PLAYER].age;
    if let Some(id) = selection.building {
        if let Some(b) = sim.world.building(id) {
            return format!("b:{:?}:{}:{}", b.kind, b.nation, age);
        }
    }
    let has_citizen = selection
        .units
        .iter()
        .filter_map(|id| sim.world.unit(*id))
        .any(|u| u.kind == UnitKind::Citizen);
    format!("u:{}:{}", has_citizen, age)
}

fn clear_action_bar(mut commands: Commands, mut bar: ResMut<ActionBar>) {
    for e in bar.buttons.drain(..) {
        commands.entity(e).despawn();
    }
    bar.signature.clear();
}

fn rebuild_action_bar(
    mut commands: Commands,
    sim: Res<Sim>,
    selection: Res<Selection>,
    mut bar: ResMut<ActionBar>,
) {
    let sig = action_signature(&sim, &selection);
    if sig == bar.signature {
        return;
    }
    bar.signature = sig;
    for e in bar.buttons.drain(..) {
        commands.entity(e).despawn();
    }

    let age = sim.world.nations[PLAYER].age;
    let mut specs: Vec<(String, String, Option<Action>)> = Vec::new();

    if let Some(id) = selection.building {
        if let Some(b) = sim.world.building(id).filter(|b| b.nation == PLAYER) {
            match b.kind {
                BuildingKind::City => {
                    let c = unit_ramped_cost(
                        UnitKind::Citizen,
                        sim.world.count_units(PLAYER, UnitKind::Citizen),
                    );
                    specs.push(("Citizen".into(), c.describe(), Some(Action::TrainCitizen)));
                    if age < 7 {
                        specs.push((
                            "Advance Age".into(),
                            age_up_cost(age + 1).describe(),
                            Some(Action::AgeUp),
                        ));
                    }
                }
                BuildingKind::Barracks => {
                    for k in UnitKind::MILITARY {
                        let c = unit_ramped_cost(k, sim.world.count_units(PLAYER, k));
                        specs.push((
                            unit_name(k, age).into(),
                            c.describe(),
                            Some(Action::Train(k)),
                        ));
                    }
                }
                _ => {}
            }
        }
    } else {
        let has_citizen = selection
            .units
            .iter()
            .filter_map(|id| sim.world.unit(*id))
            .any(|u| u.kind == UnitKind::Citizen);
        if has_citizen {
            for kind in BuildingKind::BUILDABLE {
                let unlocked = age >= kind.min_age();
                let detail = if unlocked {
                    kind.cost().describe()
                } else {
                    format!("Age {}+", kind.min_age() + 1)
                };
                specs.push((
                    kind.name().into(),
                    detail,
                    unlocked.then_some(Action::Build(kind)),
                ));
            }
        }
    }

    for (i, (label, detail, action)) in specs.into_iter().enumerate() {
        let x = 16.0 + i as f32 * 128.0;
        let enabled = action.is_some();
        let mut ent = commands.spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(x),
                width: Val::Px(120.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            BackgroundColor(if enabled {
                Color::srgba(0.16, 0.17, 0.15, 0.95)
            } else {
                Color::srgba(0.1, 0.1, 0.1, 0.9)
            }),
        ));
        if let Some(a) = action {
            ent.insert(ActionButton(a));
        }
        let text_color = if enabled {
            Color::srgb(0.9, 0.9, 0.85)
        } else {
            Color::srgb(0.5, 0.5, 0.5)
        };
        ent.with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont { font_size: 15.0, ..Default::default() },
                TextColor(text_color),
            ));
            p.spawn((
                Text::new(detail),
                TextFont { font_size: 12.0, ..Default::default() },
                TextColor(Color::srgb(0.72, 0.68, 0.5)),
            ));
        });
        bar.buttons.push(ent.id());
    }
}

fn handle_action_clicks(
    mut interactions: Query<
        (&Interaction, &ActionButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut sim: ResMut<Sim>,
    selection: Res<Selection>,
    mut placement: ResMut<PlacementMode>,
    mut toast: ResMut<Toast>,
) {
    for (interaction, button, mut bg) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.3, 0.34, 0.28, 0.98));
                apply_action(button.0, &mut sim, &selection, &mut placement, &mut toast);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.24, 0.26, 0.22, 0.97));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.16, 0.17, 0.15, 0.95));
            }
        }
    }
}

fn apply_action(
    action: Action,
    sim: &mut Sim,
    selection: &Selection,
    placement: &mut PlacementMode,
    toast: &mut Toast,
) {
    match action {
        Action::Build(kind) => placement.kind = Some(kind),
        Action::TrainCitizen => {
            if let Some(id) = selection.building {
                if let Err(e) = sim.world.try_enqueue(id, QueueItem::Unit(UnitKind::Citizen)) {
                    toast.show(e);
                }
            }
        }
        Action::Train(kind) => {
            if let Some(id) = selection.building {
                if let Err(e) = sim.world.try_enqueue(id, QueueItem::Unit(kind)) {
                    toast.show(e);
                }
            }
        }
        Action::AgeUp => {
            if let Some(id) = selection.building {
                if let Err(e) = sim.world.try_enqueue(id, QueueItem::AgeUp) {
                    toast.show(e);
                }
            }
        }
    }
}
