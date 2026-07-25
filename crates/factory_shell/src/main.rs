use bevy::prelude::*;
use factory_content::{ContentDatabase, IRON_BARS_SCENARIO};
use factory_sim::{
  DispatchPhase, DispatchReceiverState, GameState, GridPosition, NodeId, TickSnapshot,
};
use std::collections::BTreeMap;

const NORMAL_TICKS_PER_SECOND: f32 = 2.0;
const FAST_TICKS_PER_SECOND: f32 = 8.0;
const MAX_TICKS_PER_FRAME: u8 = 8;
const GRID_X: f32 = 180.0;
const GRID_Y: f32 = 120.0;
const WORLD_LEFT: f32 = -410.0;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "factory game".into(),
        resolution: (1180, 720).into(),
        fit_canvas_to_parent: true,
        ..default()
      }),
      ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
    .insert_resource(SimHost::new())
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        handle_controls,
        advance_simulation,
        project_snapshot,
        update_text,
      )
        .chain(),
    )
    .run();
}

#[derive(Resource)]
struct SimHost {
  game: GameState,
  snapshot: TickSnapshot,
  paused: bool,
  ticks_per_second: f32,
  accumulated_seconds: f32,
}

impl SimHost {
  fn new() -> Self {
    let game = starter_game();
    let snapshot = game.snapshot(Vec::new());
    Self {
      game,
      snapshot,
      paused: false,
      ticks_per_second: NORMAL_TICKS_PER_SECOND,
      accumulated_seconds: 0.0,
    }
  }

  fn step_once(&mut self) {
    self.snapshot = self.game.step();
  }

  fn reset(&mut self) {
    self.game = starter_game();
    self.snapshot = self.game.snapshot(Vec::new());
    self.accumulated_seconds = 0.0;
  }

  fn toggle_speed(&mut self) {
    self.ticks_per_second = if self.ticks_per_second == NORMAL_TICKS_PER_SECOND {
      FAST_TICKS_PER_SECOND
    } else {
      NORMAL_TICKS_PER_SECOND
    };
    self.accumulated_seconds = 0.0;
  }
}

fn starter_game() -> GameState {
  GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO)
    .expect("starter iron-bars scenario is valid")
}

#[derive(Component)]
struct NodeVisual(NodeId);

#[derive(Component)]
struct NodeLabel(NodeId);

#[derive(Component)]
struct HaulerVisual(u8);

#[derive(Component)]
struct HaulerLabel(u8);

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct EventText;

fn setup(mut commands: Commands, host: Res<SimHost>) {
  commands.spawn(Camera2d);

  spawn_connections(&mut commands, &host.snapshot);
  for node in &host.snapshot.topology.nodes {
    let position = grid_to_world(node.position);
    let (color, size) = match node.id {
      NodeId::Source(_) => (Color::srgb(0.85, 0.52, 0.25), Vec2::new(124.0, 74.0)),
      NodeId::Road => (Color::srgb(0.30, 0.34, 0.40), Vec2::new(100.0, 34.0)),
      NodeId::Factory => (Color::srgb(0.30, 0.66, 0.47), Vec2::new(132.0, 82.0)),
    };
    commands.spawn((
      Sprite::from_color(color, size),
      Transform::from_xyz(position.x, position.y, 1.0),
      NodeVisual(node.id),
    ));
    commands.spawn((
      Text2d::new(node.id.to_string()),
      TextFont {
        font_size: FontSize::Px(17.0),
        ..default()
      },
      TextColor(Color::srgb(0.96, 0.96, 0.94)),
      Transform::from_xyz(position.x, position.y + 58.0, 3.0),
      NodeLabel(node.id),
    ));
  }

  for hauler in &host.snapshot.haulers {
    let position = grid_to_world(hauler.position_grid);
    commands.spawn((
      Sprite::from_color(Color::srgb(0.50, 0.63, 0.88), Vec2::splat(28.0)),
      Transform::from_xyz(position.x, position.y - 54.0, 2.0),
      HaulerVisual(hauler.id),
    ));
    commands.spawn((
      Text2d::new(format!("hauler-{}", hauler.id)),
      TextFont {
        font_size: FontSize::Px(14.0),
        ..default()
      },
      TextColor(Color::srgb(0.72, 0.80, 0.96)),
      Transform::from_xyz(position.x, position.y - 82.0, 3.0),
      HaulerLabel(hauler.id),
    ));
  }

  commands.spawn((
    Text2d::new(""),
    TextFont {
      font_size: FontSize::Px(19.0),
      ..default()
    },
    TextColor(Color::srgb(0.91, 0.92, 0.94)),
    Transform::from_xyz(315.0, 190.0, 4.0),
    HudText,
  ));
  commands.spawn((
    Text2d::new(""),
    TextFont {
      font_size: FontSize::Px(15.0),
      ..default()
    },
    TextColor(Color::srgb(0.72, 0.76, 0.82)),
    Transform::from_xyz(315.0, -180.0, 4.0),
    EventText,
  ));
}

fn spawn_connections(commands: &mut Commands, snapshot: &TickSnapshot) {
  let road = snapshot
    .topology
    .nodes
    .iter()
    .find(|node| node.id == NodeId::Road)
    .expect("starter topology contains a road");
  let road_position = grid_to_world(road.position);

  for node in snapshot
    .topology
    .nodes
    .iter()
    .filter(|node| node.id != NodeId::Road)
  {
    let position = grid_to_world(node.position);
    let delta = road_position - position;
    let midpoint = position + delta / 2.0;
    commands.spawn((
      Sprite::from_color(Color::srgb(0.20, 0.23, 0.28), Vec2::new(delta.length(), 5.0)),
      Transform::from_xyz(midpoint.x, midpoint.y, 0.0)
        .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
    ));
  }
}

fn handle_controls(keys: Res<ButtonInput<KeyCode>>, mut host: ResMut<SimHost>) {
  if keys.just_pressed(KeyCode::Space) {
    host.paused = !host.paused;
    host.accumulated_seconds = 0.0;
  }
  if keys.just_pressed(KeyCode::KeyN) {
    host.paused = true;
    host.step_once();
    host.accumulated_seconds = 0.0;
  }
  if keys.just_pressed(KeyCode::KeyR) {
    host.reset();
  }
  if keys.just_pressed(KeyCode::KeyF) {
    host.toggle_speed();
  }
}

fn advance_simulation(time: Res<Time>, mut host: ResMut<SimHost>) {
  if host.paused {
    return;
  }

  host.accumulated_seconds += time.delta_secs();
  let tick_interval = 1.0 / host.ticks_per_second;
  let mut ticks = 0;
  while host.accumulated_seconds >= tick_interval && ticks < MAX_TICKS_PER_FRAME {
    host.accumulated_seconds -= tick_interval;
    host.step_once();
    ticks += 1;
  }
}

fn project_snapshot(
  host: Res<SimHost>,
  mut node_visuals: Query<(&NodeVisual, &mut Transform), Without<HaulerVisual>>,
  mut hauler_visuals: Query<(&HaulerVisual, &mut Transform), Without<NodeVisual>>,
) {
  if !host.is_changed() {
    return;
  }

  for (visual, mut transform) in &mut node_visuals {
    if let Some(node) = host
      .snapshot
      .topology
      .nodes
      .iter()
      .find(|node| node.id == visual.0)
    {
      let position = grid_to_world(node.position);
      transform.translation.x = position.x;
      transform.translation.y = position.y;
    }
  }

  for (visual, mut transform) in &mut hauler_visuals {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == visual.0)
    {
      let position = grid_to_world(hauler.position_grid);
      transform.translation.x = position.x;
      transform.translation.y = position.y - 54.0 - f32::from(hauler.id) * 30.0;
    }
  }
}

fn update_text(
  host: Res<SimHost>,
  mut node_labels: Query<(&NodeLabel, &mut Text2d), (Without<HudText>, Without<EventText>)>,
  mut hauler_labels: Query<
    (&HaulerLabel, &mut Text2d, &mut Transform),
    (Without<NodeLabel>, Without<HudText>, Without<EventText>),
  >,
  mut hud: Query<&mut Text2d, (With<HudText>, Without<EventText>)>,
  mut events: Query<&mut Text2d, (With<EventText>, Without<HudText>)>,
) {
  if !host.is_changed() {
    return;
  }

  for (label, mut text) in &mut node_labels {
    let value = match label.0 {
      NodeId::Source(_) => host
        .snapshot
        .sources
        .iter()
        .find(|source| source.node == label.0)
        .map(|source| {
          format!(
            "{}\nstock: {}",
            source.node,
            format_items(&source.stockpile.items)
          )
        })
        .unwrap_or_else(|| label.0.to_string()),
      NodeId::Road => "road".into(),
      NodeId::Factory => format!(
        "factory\ninventory: {}\ncraft: {}/{} {}",
        format_items(&host.snapshot.factory.inventory.items),
        host.snapshot.factory.craft.craft_progress,
        host.snapshot.factory.craft.craft_time,
        if host.snapshot.factory.craft.crafting {
          "active"
        } else {
          "idle"
        }
      ),
    };
    *text = Text2d::new(value);
  }

  for (label, mut text, mut transform) in &mut hauler_labels {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == label.0)
    {
      let position = grid_to_world(hauler.position_grid);
      transform.translation.x = position.x;
      transform.translation.y = position.y - 82.0 - f32::from(hauler.id) * 30.0;
      *text = Text2d::new(format!(
        "hauler-{} | {} | {}",
        hauler.id,
        dispatch_text(&hauler.dispatch),
        format_items(&hauler.cargo.items)
      ));
    }
  }

  let metrics = host.game.metrics();
  let status = if host.paused { "paused" } else { "running" };
  let hud_value = format!(
    "FACTORY GAME\n{}\n\ntick: {}\nstatus: {}\nspeed: {:.0} ticks/sec\n\n\
     mined: {}\ncrafted: {}\ndispatches: {}\nidle ticks: {}\n\n\
     CONTROLS\nSpace  play / pause\nN      single step\nR      reset\nF      2x / 8x speed",
    host.snapshot.scenario.name,
    host.snapshot.tick,
    status,
    host.ticks_per_second,
    format_items(&metrics.mined),
    format_items(&metrics.crafted),
    metrics.dispatches_assigned,
    metrics.idle_ticks,
  );
  for mut text in &mut hud {
    *text = Text2d::new(hud_value.clone());
  }

  let event_value = if host.snapshot.events.is_empty() {
    "EVENTS\nwaiting for first tick".into()
  } else {
    format!(
      "EVENTS\n{}",
      host
        .snapshot
        .events
        .iter()
        .rev()
        .take(6)
        .rev()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
    )
  };
  for mut text in &mut events {
    *text = Text2d::new(event_value.clone());
  }
}

fn grid_to_world(position: GridPosition) -> Vec2 {
  Vec2::new(
    WORLD_LEFT + position.x as f32 * GRID_X,
    position.y as f32 * GRID_Y,
  )
}

fn format_items(items: &BTreeMap<String, u32>) -> String {
  if items.is_empty() {
    return "empty".into();
  }
  items
    .iter()
    .map(|(item, quantity)| format!("{item}={quantity}"))
    .collect::<Vec<_>>()
    .join(", ")
}

fn dispatch_text(state: &DispatchReceiverState) -> String {
  match state {
    DispatchReceiverState::Unassigned => "idle".into(),
    DispatchReceiverState::Assigned(assignment) => {
      let phase = match assignment.phase {
        DispatchPhase::Collect => "collect",
        DispatchPhase::Deliver => "deliver",
      };
      format!("{phase} {}", assignment.item)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sim_host_steps_match_direct_simulation_bytes() {
    let mut host = SimHost::new();
    let mut direct = starter_game();

    for _ in 0..6 {
      host.step_once();
      let direct_snapshot = direct.step();
      assert_eq!(
        serde_json::to_vec(&direct_snapshot).expect("direct snapshot serializes"),
        serde_json::to_vec(&host.snapshot).expect("viewer snapshot serializes")
      );
    }
  }

  #[test]
  fn reset_restores_the_initial_snapshot_and_preserves_speed() {
    let mut host = SimHost::new();
    host.step_once();
    host.toggle_speed();
    host.reset();

    assert_eq!(0, host.snapshot.tick);
    assert_eq!(FAST_TICKS_PER_SECOND, host.ticks_per_second);
    assert_eq!(0.0, host.accumulated_seconds);
  }
}
