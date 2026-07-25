use bevy::prelude::*;
use factory_content::{
  ContentDatabase, ScenarioId, BUILDING_MATERIALS_SCENARIO, IRON_BARS_FLEET_SCENARIO,
  IRON_BARS_SCENARIO,
};
use factory_sim::{
  DispatchPhase, DispatchReceiverState, GameState, GridPosition, HaulerSnapshot, NodeId,
  TickSnapshot,
};
use std::collections::{BTreeMap, VecDeque};

const NORMAL_TICKS_PER_SECOND: f32 = 2.0;
const FAST_TICKS_PER_SECOND: f32 = 8.0;
const MAX_TICKS_PER_FRAME: u8 = 8;
const AUTO_ADVANCE_IDLE_TICKS: u16 = 8;
const MAX_RECENT_EVENTS: usize = 8;
const ROUTE_DASH_COUNT: usize = 5;
const ROUTE_DASH_SPEED: f32 = 0.42;
const CRAFT_GAUGE_WIDTH: f32 = 96.0;
const OUTPUT_CHIP_COUNT: usize = 5;
const MAX_OUTPUT_CHIPS: usize = 15;
const OUTPUT_CHIP_LIFETIME: f32 = 0.55;
const DEMO_SCENARIOS: [ScenarioId; 3] = [
  IRON_BARS_SCENARIO,
  IRON_BARS_FLEET_SCENARIO,
  BUILDING_MATERIALS_SCENARIO,
];
const GRID_X: f32 = 180.0;
const GRID_Y: f32 = 120.0;
const WORLD_LEFT: f32 = -410.0;
const BUTTON_NORMAL: Color = Color::srgb(0.14, 0.17, 0.22);
const BUTTON_HOVERED: Color = Color::srgb(0.24, 0.29, 0.36);
const BUTTON_ACTIVE: Color = Color::srgb(0.30, 0.66, 0.47);
const BUTTON_PRESSED: Color = Color::srgb(0.85, 0.52, 0.25);
const BUTTON_BORDER: Color = Color::srgb(0.34, 0.39, 0.48);
const NODE_SOURCE_IDLE: Color = Color::srgb(0.56, 0.36, 0.22);
const NODE_SOURCE_READY: Color = Color::srgb(0.96, 0.64, 0.24);
const NODE_ROAD: Color = Color::srgb(0.30, 0.34, 0.40);
const NODE_FACTORY_IDLE: Color = Color::srgb(0.23, 0.47, 0.36);
const NODE_FACTORY_DEMAND: Color = Color::srgb(0.24, 0.69, 0.65);
const NODE_FACTORY_CRAFTING: Color = Color::srgb(0.34, 0.83, 0.48);
const ROUTE_IDLE: Color = Color::srgb(0.20, 0.23, 0.28);
const ROUTE_ACTIVE: Color = Color::srgb(0.94, 0.67, 0.25);
const ROUTE_DASH: Color = Color::srgb(1.0, 0.88, 0.48);
const HAULER_IDLE: Color = Color::srgb(0.38, 0.45, 0.58);
const HAULER_COLLECTING: Color = Color::srgb(0.95, 0.60, 0.24);
const HAULER_DELIVERING: Color = Color::srgb(0.38, 0.72, 0.98);
const CARGO_EMPTY: Color = Color::srgb(0.10, 0.13, 0.18);
const CARGO_LOADED: Color = Color::srgb(0.98, 0.92, 0.58);
const CRAFT_GAUGE_BACKGROUND: Color = Color::srgb(0.10, 0.16, 0.14);
const CRAFT_GAUGE_FILL: Color = Color::srgb(0.58, 0.96, 0.62);
const OUTPUT_CHIP_STONE: Color = Color::srgb(0.62, 0.58, 0.48);
const OUTPUT_CHIP_STEEL: Color = Color::srgb(0.48, 0.55, 0.58);

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
    .init_resource::<ProjectionScene>()
    .init_resource::<ProductionFeedback>()
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        handle_controls,
        handle_control_buttons,
        advance_simulation,
        rebuild_projection,
        project_snapshot,
        project_activity,
        project_craft_gauge,
        emit_output_chips,
        animate_activity,
        animate_output_chips,
        animate_haulers,
        update_text,
        style_control_buttons,
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
  scenario_index: usize,
  auto_cycle: bool,
  idle_streak: u16,
  completed_scenarios: u32,
  scene_revision: u64,
  recent_events: VecDeque<String>,
}

impl SimHost {
  fn new() -> Self {
    let game = scenario_game(DEMO_SCENARIOS[0]);
    let snapshot = game.snapshot(Vec::new());
    let scenario_name = snapshot.scenario.name.clone();
    Self {
      game,
      snapshot,
      paused: false,
      ticks_per_second: NORMAL_TICKS_PER_SECOND,
      accumulated_seconds: 0.0,
      scenario_index: 0,
      auto_cycle: true,
      idle_streak: 0,
      completed_scenarios: 0,
      scene_revision: 0,
      recent_events: VecDeque::from([format!("t000 showcase started: {scenario_name}")]),
    }
  }

  fn step_once(&mut self) {
    self.snapshot = self.game.step();
    self.record_snapshot_events();
    self.idle_streak = if self.snapshot.events.is_empty() {
      self.idle_streak.saturating_add(1)
    } else {
      0
    };
    if self.auto_cycle && self.idle_streak >= AUTO_ADVANCE_IDLE_TICKS {
      self.completed_scenarios += 1;
      self.next_scenario("showcase advanced");
    }
  }

  fn reset(&mut self) {
    self.game = scenario_game(DEMO_SCENARIOS[self.scenario_index]);
    self.snapshot = self.game.snapshot(Vec::new());
    self
      .snapshot
      .events
      .push(format!("scenario reset: {}", self.snapshot.scenario.name));
    self.record_snapshot_events();
    self.accumulated_seconds = 0.0;
    self.idle_streak = 0;
  }

  fn toggle_speed(&mut self) {
    self.ticks_per_second = if self.ticks_per_second == NORMAL_TICKS_PER_SECOND {
      FAST_TICKS_PER_SECOND
    } else {
      NORMAL_TICKS_PER_SECOND
    };
    self.accumulated_seconds = 0.0;
  }

  fn toggle_auto_cycle(&mut self) {
    self.auto_cycle = !self.auto_cycle;
    self.idle_streak = 0;
  }

  fn apply_control(&mut self, action: ControlAction) {
    match action {
      ControlAction::TogglePause => {
        self.paused = !self.paused;
        self.accumulated_seconds = 0.0;
      }
      ControlAction::Step => {
        self.paused = true;
        self.step_once();
        self.accumulated_seconds = 0.0;
      }
      ControlAction::Reset => self.reset(),
      ControlAction::ToggleSpeed => self.toggle_speed(),
      ControlAction::ToggleAutoCycle => self.toggle_auto_cycle(),
      ControlAction::SelectScenario(index) => {
        self.select_scenario(index, "scenario selected");
      }
    }
  }

  fn select_scenario(&mut self, index: usize, reason: &str) {
    self.scenario_index = index % DEMO_SCENARIOS.len();
    self.game = scenario_game(DEMO_SCENARIOS[self.scenario_index]);
    self.snapshot = self.game.snapshot(Vec::new());
    let scenario_name = self.snapshot.scenario.name.clone();
    self
      .snapshot
      .events
      .push(format!("{reason}: {scenario_name}"));
    self.record_snapshot_events();
    self.accumulated_seconds = 0.0;
    self.idle_streak = 0;
    self.scene_revision += 1;
  }

  fn record_snapshot_events(&mut self) {
    let entries = self
      .snapshot
      .events
      .iter()
      .map(|event| format!("t{:03} {event}", self.snapshot.tick))
      .collect::<Vec<_>>();
    for entry in entries {
      self.recent_events.push_back(entry);
      if self.recent_events.len() > MAX_RECENT_EVENTS {
        self.recent_events.pop_front();
      }
    }
  }

  fn next_scenario(&mut self, reason: &str) {
    self.select_scenario(self.scenario_index + 1, reason);
  }
}

fn scenario_game(scenario: ScenarioId) -> GameState {
  GameState::new(ContentDatabase::starter(), scenario).expect("showcase scenario is valid")
}

#[derive(Resource, Default)]
struct ProjectionScene {
  revision: u64,
}

#[derive(Resource, Default)]
struct ProductionFeedback {
  scene_revision: u64,
  crafted: BTreeMap<String, u32>,
}

#[derive(Component)]
struct ProjectionEntity;

#[derive(Component)]
struct NodeVisual(NodeId);

#[derive(Component)]
struct NodeLabel(NodeId);

#[derive(Component)]
struct HaulerVisual(u8);

#[derive(Component)]
struct RouteVisual(NodeId);

#[derive(Component)]
struct RouteDash {
  node: NodeId,
  outer: Vec2,
  road: Vec2,
  offset: f32,
}

#[derive(Component)]
struct HaulerLabel(u8);

#[derive(Component)]
struct HaulerTarget(Vec2);

#[derive(Component)]
struct CargoBadge(u8);

#[derive(Component)]
struct CraftGaugeFill {
  left: f32,
  max_width: f32,
}

#[derive(Component)]
struct OutputChip {
  velocity: Vec2,
  remaining: f32,
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct EventText;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
enum ControlAction {
  TogglePause,
  Step,
  Reset,
  ToggleSpeed,
  ToggleAutoCycle,
  SelectScenario(usize),
}

impl ControlAction {
  fn is_selected(self, host: &SimHost) -> bool {
    match self {
      Self::TogglePause => host.paused,
      Self::ToggleSpeed => host.ticks_per_second == FAST_TICKS_PER_SECOND,
      Self::ToggleAutoCycle => host.auto_cycle,
      Self::SelectScenario(index) => host.scenario_index == index,
      Self::Step | Self::Reset => false,
    }
  }
}

#[derive(Component)]
struct ControlButton(ControlAction);

fn setup(
  mut commands: Commands,
  host: Res<SimHost>,
  mut projection_scene: ResMut<ProjectionScene>,
) {
  commands.spawn(Camera2d);

  spawn_projection(&mut commands, &host.snapshot);
  projection_scene.revision = host.scene_revision;

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
    Transform::from_xyz(315.0, -105.0, 4.0),
    EventText,
  ));
  spawn_control_deck(&mut commands);
}

fn spawn_control_deck(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        right: px(18),
        bottom: px(18),
        width: px(452),
        flex_direction: FlexDirection::Column,
        row_gap: px(7),
        padding: UiRect::all(px(10)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.94)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(100),
    ))
    .with_children(|panel| {
      panel.spawn((
        Text::new("CONTROL DECK"),
        TextFont {
          font_size: FontSize::Px(13.0),
          ..default()
        },
        TextColor(Color::srgb(0.72, 0.76, 0.82)),
      ));
      spawn_control_row(
        panel,
        &[
          (ControlAction::TogglePause, "PLAY / PAUSE"),
          (ControlAction::Step, "STEP"),
          (ControlAction::Reset, "RESET"),
          (ControlAction::ToggleSpeed, "SPEED"),
          (ControlAction::ToggleAutoCycle, "AUTO"),
        ],
      );
      spawn_control_row(
        panel,
        &[
          (ControlAction::SelectScenario(0), "IRON"),
          (ControlAction::SelectScenario(1), "FLEET"),
          (ControlAction::SelectScenario(2), "MATERIALS"),
        ],
      );
    });
}

fn spawn_control_row(
  parent: &mut ChildSpawnerCommands,
  buttons: &[(ControlAction, &'static str)],
) {
  parent
    .spawn(Node {
      width: percent(100),
      height: px(34),
      column_gap: px(6),
      ..default()
    })
    .with_children(|row| {
      for (action, label) in buttons {
        row.spawn((
          Button,
          ControlButton(*action),
          Node {
            height: percent(100),
            flex_grow: 1.0,
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(7), px(3)),
            ..default()
          },
          BackgroundColor(BUTTON_NORMAL),
          BorderColor::all(BUTTON_BORDER),
          children![(
            Text::new(*label),
            TextFont {
              font_size: FontSize::Px(12.0),
              ..default()
            },
            TextColor(Color::srgb(0.92, 0.94, 0.97)),
          )],
        ));
      }
    });
}

fn spawn_projection(commands: &mut Commands, snapshot: &TickSnapshot) {
  spawn_connections(commands, snapshot);
  for node in &snapshot.topology.nodes {
    let position = grid_to_world(node.position);
    let size = match node.id {
      NodeId::Source(_) => Vec2::new(124.0, 74.0),
      NodeId::Road => Vec2::new(100.0, 34.0),
      NodeId::Factory => Vec2::new(132.0, 82.0),
    };
    commands.spawn((
      Sprite::from_color(node_color(snapshot, node.id), size),
      Transform::from_xyz(position.x, position.y, 1.0),
      NodeVisual(node.id),
      ProjectionEntity,
    ));
    commands.spawn((
      Text2d::new(node_label_value(snapshot, node.id)),
      TextFont {
        font_size: FontSize::Px(17.0),
        ..default()
      },
      TextColor(Color::srgb(0.96, 0.96, 0.94)),
      Transform::from_xyz(position.x, position.y + 58.0, 3.0),
      NodeLabel(node.id),
      ProjectionEntity,
    ));
    if node.id == NodeId::Factory {
      spawn_craft_gauge(commands, snapshot, position);
    }
  }

  for hauler in &snapshot.haulers {
    let position = hauler_world_position(hauler);
    commands.spawn((
      Sprite::from_color(hauler_color(hauler), Vec2::splat(hauler_size(hauler))),
      Transform::from_xyz(position.x, position.y, 2.0),
      HaulerVisual(hauler.id),
      HaulerTarget(position),
      ProjectionEntity,
    ));
    commands.spawn((
      Sprite::from_color(
        cargo_badge_color(hauler),
        Vec2::splat(cargo_badge_size(hauler)),
      ),
      Transform::from_xyz(position.x, position.y, 2.5),
      CargoBadge(hauler.id),
      HaulerTarget(position),
      ProjectionEntity,
    ));
    commands.spawn((
      Text2d::new(hauler_label_value(hauler)),
      TextFont {
        font_size: FontSize::Px(14.0),
        ..default()
      },
      TextColor(Color::srgb(0.72, 0.80, 0.96)),
      Transform::from_xyz(position.x, position.y - 28.0, 3.0),
      HaulerLabel(hauler.id),
      HaulerTarget(Vec2::new(position.x, position.y - 28.0)),
      ProjectionEntity,
    ));
  }
}

fn spawn_craft_gauge(commands: &mut Commands, snapshot: &TickSnapshot, factory: Vec2) {
  let y = factory.y - 28.0;
  let left = factory.x - CRAFT_GAUGE_WIDTH / 2.0;
  let progress = craft_progress_fraction(
    snapshot.factory.craft.craft_progress,
    snapshot.factory.craft.craft_time,
  );
  let width = CRAFT_GAUGE_WIDTH * progress;
  commands.spawn((
    Sprite::from_color(
      CRAFT_GAUGE_BACKGROUND,
      Vec2::new(CRAFT_GAUGE_WIDTH + 4.0, 10.0),
    ),
    Transform::from_xyz(factory.x, y, 1.5),
    ProjectionEntity,
  ));
  commands.spawn((
    Sprite::from_color(CRAFT_GAUGE_FILL, Vec2::new(width, 6.0)),
    Transform::from_xyz(left + width / 2.0, y, 1.6),
    if width > 0.0 {
      Visibility::Visible
    } else {
      Visibility::Hidden
    },
    CraftGaugeFill {
      left,
      max_width: CRAFT_GAUGE_WIDTH,
    },
    ProjectionEntity,
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
      Sprite::from_color(route_color(snapshot, node.id), Vec2::new(delta.length(), 5.0)),
      Transform::from_xyz(midpoint.x, midpoint.y, 0.0)
        .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
      RouteVisual(node.id),
      ProjectionEntity,
    ));
    for index in 0..ROUTE_DASH_COUNT {
      let offset = index as f32 / ROUTE_DASH_COUNT as f32;
      let dash_position = position.lerp(road_position, offset);
      commands.spawn((
        Sprite::from_color(ROUTE_DASH, Vec2::new(20.0, 3.0)),
        Transform::from_xyz(dash_position.x, dash_position.y, 0.5)
          .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
        Visibility::Hidden,
        RouteDash {
          node: node.id,
          outer: position,
          road: road_position,
          offset,
        },
        ProjectionEntity,
      ));
    }
  }
}

fn handle_controls(keys: Res<ButtonInput<KeyCode>>, mut host: ResMut<SimHost>) {
  for (key, action) in [
    (KeyCode::Space, ControlAction::TogglePause),
    (KeyCode::KeyN, ControlAction::Step),
    (KeyCode::KeyR, ControlAction::Reset),
    (KeyCode::KeyF, ControlAction::ToggleSpeed),
    (KeyCode::KeyC, ControlAction::SelectScenario(
      (host.scenario_index + 1) % DEMO_SCENARIOS.len(),
    )),
    (KeyCode::KeyL, ControlAction::ToggleAutoCycle),
  ] {
    if keys.just_pressed(key) {
      host.apply_control(action);
    }
  }
}

fn handle_control_buttons(
  buttons: Query<(&Interaction, &ControlButton), (Changed<Interaction>, With<Button>)>,
  mut host: ResMut<SimHost>,
) {
  for (interaction, button) in &buttons {
    if *interaction == Interaction::Pressed {
      host.apply_control(button.0);
    }
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

fn rebuild_projection(
  mut commands: Commands,
  host: Res<SimHost>,
  mut projection_scene: ResMut<ProjectionScene>,
  entities: Query<Entity, With<ProjectionEntity>>,
) {
  if projection_scene.revision == host.scene_revision {
    return;
  }

  for entity in &entities {
    commands.entity(entity).despawn();
  }
  spawn_projection(&mut commands, &host.snapshot);
  projection_scene.revision = host.scene_revision;
}

fn project_snapshot(
  host: Res<SimHost>,
  mut hauler_visuals: Query<
    (&HaulerVisual, &mut HaulerTarget),
    (Without<HaulerLabel>, Without<CargoBadge>),
  >,
  mut hauler_labels: Query<
    (&HaulerLabel, &mut HaulerTarget),
    (Without<HaulerVisual>, Without<CargoBadge>),
  >,
  mut cargo_badges: Query<
    (&CargoBadge, &mut HaulerTarget),
    (Without<HaulerVisual>, Without<HaulerLabel>),
  >,
) {
  if !host.is_changed() {
    return;
  }

  for (visual, mut target) in &mut hauler_visuals {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == visual.0)
    {
      target.0 = hauler_world_position(hauler);
    }
  }

  for (label, mut target) in &mut hauler_labels {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == label.0)
    {
      let position = hauler_world_position(hauler);
      target.0 = Vec2::new(position.x, position.y - 28.0);
    }
  }

  for (badge, mut target) in &mut cargo_badges {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == badge.0)
    {
      target.0 = hauler_world_position(hauler);
    }
  }
}

fn project_activity(
  host: Res<SimHost>,
  mut nodes: Query<
    (&NodeVisual, &mut Sprite),
    (
      Without<HaulerVisual>,
      Without<RouteVisual>,
      Without<CargoBadge>,
    ),
  >,
  mut routes: Query<
    (&RouteVisual, &mut Sprite),
    (
      Without<NodeVisual>,
      Without<HaulerVisual>,
      Without<CargoBadge>,
    ),
  >,
  mut haulers: Query<
    (&HaulerVisual, &mut Sprite),
    (
      Without<NodeVisual>,
      Without<RouteVisual>,
      Without<CargoBadge>,
    ),
  >,
  mut cargo_badges: Query<
    (&CargoBadge, &mut Sprite),
    (
      Without<NodeVisual>,
      Without<RouteVisual>,
      Without<HaulerVisual>,
    ),
  >,
) {
  if !host.is_changed() {
    return;
  }

  for (visual, mut sprite) in &mut nodes {
    sprite.color = node_color(&host.snapshot, visual.0);
  }
  for (visual, mut sprite) in &mut routes {
    sprite.color = route_color(&host.snapshot, visual.0);
  }
  for (visual, mut sprite) in &mut haulers {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == visual.0)
    {
      sprite.color = hauler_color(hauler);
      sprite.custom_size = Some(Vec2::splat(hauler_size(hauler)));
    }
  }
  for (badge, mut sprite) in &mut cargo_badges {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == badge.0)
    {
      sprite.color = cargo_badge_color(hauler);
      sprite.custom_size = Some(Vec2::splat(cargo_badge_size(hauler)));
    }
  }
}

fn project_craft_gauge(
  host: Res<SimHost>,
  mut gauges: Query<(
    &CraftGaugeFill,
    &mut Sprite,
    &mut Transform,
    &mut Visibility,
  )>,
) {
  if !host.is_changed() {
    return;
  }

  let progress = craft_progress_fraction(
    host.snapshot.factory.craft.craft_progress,
    host.snapshot.factory.craft.craft_time,
  );
  for (gauge, mut sprite, mut transform, mut visibility) in &mut gauges {
    let width = gauge.max_width * progress;
    sprite.custom_size = Some(Vec2::new(width, 6.0));
    transform.translation.x = gauge.left + width / 2.0;
    *visibility = if width > 0.0 {
      Visibility::Visible
    } else {
      Visibility::Hidden
    };
  }
}

fn emit_output_chips(
  mut commands: Commands,
  host: Res<SimHost>,
  mut feedback: ResMut<ProductionFeedback>,
  chips: Query<(), With<OutputChip>>,
) {
  if !host.is_changed() {
    return;
  }

  let crafted = host.game.metrics().crafted;
  let produced = if feedback.scene_revision == host.scene_revision {
    crafted_output_delta(&feedback.crafted, &crafted)
  } else {
    0
  };
  feedback.scene_revision = host.scene_revision;
  feedback.crafted = crafted;
  if produced == 0 {
    return;
  }

  let Some(factory) = host
    .snapshot
    .topology
    .nodes
    .iter()
    .find(|node| node.id == NodeId::Factory)
    .map(|node| grid_to_world(node.position))
  else {
    return;
  };
  let chip_count = OUTPUT_CHIP_COUNT.min(MAX_OUTPUT_CHIPS.saturating_sub(chips.iter().count()));
  for index in 0..chip_count {
    let offset = output_chip_offset(index);
    let color = if index % 2 == 0 {
      OUTPUT_CHIP_STONE
    } else {
      OUTPUT_CHIP_STEEL
    };
    commands.spawn((
      Sprite::from_color(color, Vec2::new(12.0, 5.0)),
      Transform::from_xyz(factory.x + offset.x, factory.y + offset.y, 4.0),
      OutputChip {
        velocity: output_chip_velocity(index),
        remaining: OUTPUT_CHIP_LIFETIME,
      },
      ProjectionEntity,
    ));
  }
}

fn animate_activity(
  time: Res<Time>,
  host: Res<SimHost>,
  mut nodes: Query<(&NodeVisual, &mut Transform), Without<RouteDash>>,
  mut route_dashes: Query<
    (&RouteDash, &mut Transform, &mut Visibility),
    Without<NodeVisual>,
  >,
) {
  let elapsed = time.elapsed_secs();
  let pulse = 1.0 + 0.045 * (elapsed * 5.0).sin().max(0.0);
  for (visual, mut transform) in &mut nodes {
    transform.scale = if node_activity(&host.snapshot, visual.0) == NodeActivity::Idle {
      Vec3::ONE
    } else {
      Vec3::splat(pulse)
    };
  }

  for (dash, mut transform, mut visibility) in &mut route_dashes {
    let Some(direction) = route_direction(&host.snapshot, dash.node) else {
      *visibility = Visibility::Hidden;
      continue;
    };
    *visibility = Visibility::Visible;
    let phase = (elapsed * ROUTE_DASH_SPEED + dash.offset) % 1.0;
    let position = match direction {
      RouteDirection::TowardRoad => dash.outer.lerp(dash.road, phase),
      RouteDirection::AwayFromRoad => dash.road.lerp(dash.outer, phase),
    };
    transform.translation.x = position.x;
    transform.translation.y = position.y;
  }
}

fn animate_output_chips(
  mut commands: Commands,
  time: Res<Time>,
  mut chips: Query<(Entity, &mut OutputChip, &mut Transform)>,
) {
  let delta = time.delta_secs();
  for (entity, mut chip, mut transform) in &mut chips {
    chip.remaining -= delta;
    if chip.remaining <= 0.0 {
      commands.entity(entity).despawn();
      continue;
    }
    transform.translation.x += chip.velocity.x * delta;
    transform.translation.y += chip.velocity.y * delta;
    transform.scale = Vec3::splat((chip.remaining / OUTPUT_CHIP_LIFETIME).max(0.25));
  }
}

fn animate_haulers(time: Res<Time>, mut haulers: Query<(&HaulerTarget, &mut Transform)>) {
  let blend = 1.0 - (-10.0 * time.delta_secs()).exp();
  for (target, mut transform) in &mut haulers {
    let position = transform.translation.truncate().lerp(target.0, blend);
    transform.translation.x = position.x;
    transform.translation.y = position.y;
  }
}

fn style_control_buttons(
  host: Res<SimHost>,
  mut buttons: Query<(
    &Interaction,
    &ControlButton,
    &mut BackgroundColor,
    &mut BorderColor,
  )>,
) {
  for (interaction, button, mut background, mut border) in &mut buttons {
    let selected = button.0.is_selected(&host);
    background.0 = match interaction {
      Interaction::Pressed => BUTTON_PRESSED,
      Interaction::Hovered => BUTTON_HOVERED,
      Interaction::None if selected => BUTTON_ACTIVE,
      Interaction::None => BUTTON_NORMAL,
    };
    *border = BorderColor::all(if selected {
      BUTTON_ACTIVE
    } else {
      BUTTON_BORDER
    });
  }
}

fn update_text(
  host: Res<SimHost>,
  mut node_labels: Query<(&NodeLabel, &mut Text2d), (Without<HudText>, Without<EventText>)>,
  mut hauler_labels: Query<
    (&HaulerLabel, &mut Text2d),
    (Without<NodeLabel>, Without<HudText>, Without<EventText>),
  >,
  mut hud: Query<&mut Text2d, (With<HudText>, Without<EventText>)>,
  mut events: Query<&mut Text2d, (With<EventText>, Without<HudText>)>,
) {
  if !host.is_changed() {
    return;
  }

  for (label, mut text) in &mut node_labels {
    *text = Text2d::new(node_label_value(&host.snapshot, label.0));
  }

  for (label, mut text) in &mut hauler_labels {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == label.0)
    {
      *text = Text2d::new(hauler_label_value(hauler));
    }
  }

  let metrics = host.game.metrics();
  let status = if host.paused { "paused" } else { "running" };
  let cycle_status = if host.auto_cycle { "auto" } else { "locked" };
  let hud_value = format!(
    "FACTORY GAME\n{}\n\ntick: {}\nstatus: {}\nspeed: {:.0} ticks/sec\n\n\
     mined: {}\ncrafted: {}\ndispatches: {}\nidle ticks: {}\n\n\
     showcase: {}\nquiet: {}/{}\ncompleted: {}\n\n\
     click the control deck below\nkeyboard: Space N R F C L",
    host.snapshot.scenario.name,
    host.snapshot.tick,
    status,
    host.ticks_per_second,
    format_items(&metrics.mined),
    format_items(&metrics.crafted),
    metrics.dispatches_assigned,
    metrics.idle_ticks,
    cycle_status,
    host.idle_streak,
    AUTO_ADVANCE_IDLE_TICKS,
    host.completed_scenarios,
  );
  for mut text in &mut hud {
    *text = Text2d::new(hud_value.clone());
  }

  let event_value = if host.recent_events.is_empty() {
    "EVENTS\nwaiting for first tick".into()
  } else {
    format!(
      "EVENTS\n{}",
      host
        .recent_events
        .iter()
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

fn hauler_world_position(hauler: &HaulerSnapshot) -> Vec2 {
  let position = grid_to_world(hauler.position_grid);
  Vec2::new(
    position.x,
    position.y - 54.0 - f32::from(hauler.id) * 30.0,
  )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodeActivity {
  Idle,
  Ready,
  Demanding,
  Crafting,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HaulerActivity {
  Idle,
  Collecting,
  Delivering,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RouteDirection {
  TowardRoad,
  AwayFromRoad,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CargoBadgeState {
  Empty,
  Loaded(u32),
}

fn craft_progress_fraction(progress: u32, craft_time: u32) -> f32 {
  if craft_time == 0 {
    0.0
  } else {
    (progress as f32 / craft_time as f32).clamp(0.0, 1.0)
  }
}

fn crafted_output_delta(
  previous: &BTreeMap<String, u32>,
  current: &BTreeMap<String, u32>,
) -> u32 {
  current
    .iter()
    .map(|(item, quantity)| quantity.saturating_sub(previous.get(item).copied().unwrap_or(0)))
    .sum()
}

fn output_chip_offset(index: usize) -> Vec2 {
  let centered = index as f32 - (OUTPUT_CHIP_COUNT - 1) as f32 / 2.0;
  Vec2::new(centered * 10.0, -8.0 + (index % 2) as f32 * 5.0)
}

fn output_chip_velocity(index: usize) -> Vec2 {
  let centered = index as f32 - (OUTPUT_CHIP_COUNT - 1) as f32 / 2.0;
  Vec2::new(centered * 2.0, 26.0 + (index % 2) as f32 * 6.0)
}

fn node_activity(snapshot: &TickSnapshot, node: NodeId) -> NodeActivity {
  match node {
    NodeId::Source(_) => snapshot
      .sources
      .iter()
      .find(|source| source.node == node)
      .filter(|source| {
        !source.stockpile.items.is_empty() || !source.dispatch.intents.is_empty()
      })
      .map_or(NodeActivity::Idle, |_| NodeActivity::Ready),
    NodeId::Road => NodeActivity::Idle,
    NodeId::Factory if snapshot.factory.craft.crafting => NodeActivity::Crafting,
    NodeId::Factory if !snapshot.factory.dispatch.intents.is_empty() => NodeActivity::Demanding,
    NodeId::Factory => NodeActivity::Idle,
  }
}

fn hauler_activity(hauler: &HaulerSnapshot) -> HaulerActivity {
  match &hauler.dispatch {
    DispatchReceiverState::Assigned(assignment)
      if assignment.phase == DispatchPhase::Collect =>
    {
      HaulerActivity::Collecting
    }
    DispatchReceiverState::Assigned(_) => HaulerActivity::Delivering,
    DispatchReceiverState::Unassigned if !hauler.cargo.items.is_empty() => {
      HaulerActivity::Delivering
    }
    DispatchReceiverState::Unassigned => HaulerActivity::Idle,
  }
}

fn node_color(snapshot: &TickSnapshot, node: NodeId) -> Color {
  match (node, node_activity(snapshot, node)) {
    (NodeId::Source(_), NodeActivity::Ready) => NODE_SOURCE_READY,
    (NodeId::Source(_), _) => NODE_SOURCE_IDLE,
    (NodeId::Road, _) => NODE_ROAD,
    (NodeId::Factory, NodeActivity::Crafting) => NODE_FACTORY_CRAFTING,
    (NodeId::Factory, NodeActivity::Demanding) => NODE_FACTORY_DEMAND,
    (NodeId::Factory, _) => NODE_FACTORY_IDLE,
  }
}

fn hauler_color(hauler: &HaulerSnapshot) -> Color {
  match hauler_activity(hauler) {
    HaulerActivity::Idle => HAULER_IDLE,
    HaulerActivity::Collecting => HAULER_COLLECTING,
    HaulerActivity::Delivering => HAULER_DELIVERING,
  }
}

fn hauler_size(hauler: &HaulerSnapshot) -> f32 {
  if hauler.cargo.items.is_empty() {
    25.0
  } else {
    33.0
  }
}

fn cargo_badge_state(hauler: &HaulerSnapshot) -> CargoBadgeState {
  let units = hauler.cargo.items.values().sum();
  if units == 0 {
    CargoBadgeState::Empty
  } else {
    CargoBadgeState::Loaded(units)
  }
}

fn cargo_badge_color(hauler: &HaulerSnapshot) -> Color {
  match cargo_badge_state(hauler) {
    CargoBadgeState::Empty => CARGO_EMPTY,
    CargoBadgeState::Loaded(_) => CARGO_LOADED,
  }
}

fn cargo_badge_size(hauler: &HaulerSnapshot) -> f32 {
  match cargo_badge_state(hauler) {
    CargoBadgeState::Empty => 7.0,
    CargoBadgeState::Loaded(units) => 9.0 + units.min(4) as f32,
  }
}

fn route_direction(snapshot: &TickSnapshot, node: NodeId) -> Option<RouteDirection> {
  snapshot
    .haulers
    .iter()
    .find_map(|hauler| match &hauler.dispatch {
      DispatchReceiverState::Assigned(assignment)
        if assignment.phase == DispatchPhase::Collect && assignment.source == node =>
      {
        Some(RouteDirection::TowardRoad)
      }
      DispatchReceiverState::Assigned(assignment)
        if assignment.phase == DispatchPhase::Deliver && assignment.destination == node =>
      {
        Some(RouteDirection::AwayFromRoad)
      }
      _ => None,
    })
}

fn route_color(snapshot: &TickSnapshot, node: NodeId) -> Color {
  if route_direction(snapshot, node).is_some() {
    ROUTE_ACTIVE
  } else {
    ROUTE_IDLE
  }
}

fn node_label_value(snapshot: &TickSnapshot, node: NodeId) -> String {
  match node {
    NodeId::Source(_) => snapshot
      .sources
      .iter()
      .find(|source| source.node == node)
      .map(|source| {
        format!(
          "{}\nstock: {}",
          source.node,
          format_items(&source.stockpile.items)
        )
      })
      .unwrap_or_else(|| node.to_string()),
    NodeId::Road => "road".into(),
    NodeId::Factory => format!(
      "factory\ninventory: {}\ncraft: {}/{} {}",
      format_items(&snapshot.factory.inventory.items),
      snapshot.factory.craft.craft_progress,
      snapshot.factory.craft.craft_time,
      if snapshot.factory.craft.crafting {
        "active"
      } else {
        "idle"
      }
    ),
  }
}

fn hauler_label_value(hauler: &HaulerSnapshot) -> String {
  format!(
    "hauler-{} | {} | {}",
    hauler.id,
    dispatch_text(&hauler.dispatch),
    format_items(&hauler.cargo.items)
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
    let mut direct = scenario_game(IRON_BARS_SCENARIO);

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

  #[test]
  fn control_actions_share_one_host_state_machine() {
    let mut host = SimHost::new();

    host.apply_control(ControlAction::TogglePause);
    assert!(host.paused);
    assert!(ControlAction::TogglePause.is_selected(&host));

    host.apply_control(ControlAction::Step);
    assert_eq!(1, host.snapshot.tick);
    assert!(host.paused);

    host.apply_control(ControlAction::ToggleSpeed);
    assert_eq!(FAST_TICKS_PER_SECOND, host.ticks_per_second);
    assert!(ControlAction::ToggleSpeed.is_selected(&host));

    host.apply_control(ControlAction::ToggleAutoCycle);
    assert!(!host.auto_cycle);
    assert!(!ControlAction::ToggleAutoCycle.is_selected(&host));

    host.apply_control(ControlAction::SelectScenario(2));
    assert_eq!(BUILDING_MATERIALS_SCENARIO, host.snapshot.scenario.id);
    assert!(ControlAction::SelectScenario(2).is_selected(&host));
    assert_eq!(1, host.scene_revision);

    host.apply_control(ControlAction::Reset);
    assert_eq!(0, host.snapshot.tick);
    assert_eq!(BUILDING_MATERIALS_SCENARIO, host.snapshot.scenario.id);
  }

  #[test]
  fn pressed_button_routes_through_the_shared_control_action() {
    let mut app = App::new();
    app.insert_resource(SimHost::new());
    app.add_systems(Update, handle_control_buttons);
    app.world_mut().spawn((
      Button,
      Interaction::Pressed,
      ControlButton(ControlAction::SelectScenario(1)),
    ));

    app.update();

    let host = app.world().resource::<SimHost>();
    assert_eq!(IRON_BARS_FLEET_SCENARIO, host.snapshot.scenario.id);
    assert_eq!(1, host.scene_revision);
  }

  #[test]
  fn recent_activity_is_bounded_and_survives_scenario_changes() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    for _ in 0..32 {
      host.step_once();
    }

    assert_eq!(MAX_RECENT_EVENTS, host.recent_events.len());
    let latest_before_change = host
      .recent_events
      .back()
      .expect("activity history has an event")
      .clone();

    host.select_scenario(1, "test selected");

    assert_eq!(MAX_RECENT_EVENTS, host.recent_events.len());
    assert!(host.recent_events.contains(&latest_before_change));
    assert!(
      host
        .recent_events
        .back()
        .expect("scenario event is recorded")
        .contains("test selected")
    );
  }

  #[test]
  fn presentation_state_tracks_authoritative_material_flow() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    let mut saw_ready_source = false;
    let mut saw_collecting = false;
    let mut saw_delivering = false;
    let mut saw_crafting = false;

    for _ in 0..64 {
      host.step_once();
      saw_ready_source |= host
        .snapshot
        .sources
        .iter()
        .any(|source| node_activity(&host.snapshot, source.node) == NodeActivity::Ready);
      saw_collecting |= host
        .snapshot
        .haulers
        .iter()
        .any(|hauler| hauler_activity(hauler) == HaulerActivity::Collecting);
      saw_delivering |= host
        .snapshot
        .haulers
        .iter()
        .any(|hauler| hauler_activity(hauler) == HaulerActivity::Delivering);
      saw_crafting |=
        node_activity(&host.snapshot, NodeId::Factory) == NodeActivity::Crafting;
    }

    assert!(saw_ready_source);
    assert!(saw_collecting);
    assert!(saw_delivering);
    assert!(saw_crafting);
  }

  #[test]
  fn route_direction_tracks_collect_and_delivery_phases() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    let source = host.snapshot.sources[0].node;
    let mut saw_toward_road = false;
    let mut saw_away_from_road = false;

    for _ in 0..64 {
      host.step_once();
      saw_toward_road |=
        route_direction(&host.snapshot, source) == Some(RouteDirection::TowardRoad);
      saw_away_from_road |=
        route_direction(&host.snapshot, NodeId::Factory) == Some(RouteDirection::AwayFromRoad);
    }

    assert!(saw_toward_road);
    assert!(saw_away_from_road);
  }

  #[test]
  fn cargo_badge_state_tracks_authoritative_inventory() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    assert_eq!(
      CargoBadgeState::Empty,
      cargo_badge_state(&host.snapshot.haulers[0])
    );

    let loaded = (0..64).find_map(|_| {
      host.step_once();
      host.snapshot.haulers.iter().find_map(|hauler| {
        let state = cargo_badge_state(hauler);
        (state != CargoBadgeState::Empty).then_some(state)
      })
    });

    assert!(matches!(loaded, Some(CargoBadgeState::Loaded(units)) if units > 0));
  }

  #[test]
  fn craft_progress_fraction_is_bounded() {
    assert_eq!(0.0, craft_progress_fraction(0, 0));
    assert_eq!(0.0, craft_progress_fraction(0, 4));
    assert_eq!(0.5, craft_progress_fraction(2, 4));
    assert_eq!(1.0, craft_progress_fraction(4, 4));
    assert_eq!(1.0, craft_progress_fraction(8, 4));
  }

  #[test]
  fn crafted_output_delta_detects_growth_without_false_reset_output() {
    let previous = BTreeMap::from([("IronBars".to_string(), 2)]);
    let current = BTreeMap::from([
      ("BuildingMaterials".to_string(), 1),
      ("IronBars".to_string(), 5),
    ]);

    assert_eq!(4, crafted_output_delta(&previous, &current));
    assert_eq!(0, crafted_output_delta(&current, &BTreeMap::new()));
  }

  #[test]
  fn output_chip_pattern_is_compact_and_directional() {
    let velocities = (0..OUTPUT_CHIP_COUNT)
      .map(output_chip_velocity)
      .collect::<Vec<_>>();
    let offsets = (0..OUTPUT_CHIP_COUNT)
      .map(output_chip_offset)
      .collect::<Vec<_>>();

    assert!(velocities.iter().all(|velocity| velocity.y > 0.0));
    assert!(velocities.iter().all(|velocity| velocity.x.abs() <= 4.0));
    assert!(offsets.iter().all(|offset| offset.x.abs() <= 20.0));
  }

  #[test]
  fn scenario_selection_follows_the_showcase_order_and_wraps() {
    let mut host = SimHost::new();

    assert_eq!(IRON_BARS_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(IRON_BARS_FLEET_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(BUILDING_MATERIALS_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(IRON_BARS_SCENARIO, host.snapshot.scenario.id);
    assert_eq!(3, host.scene_revision);
  }

  #[test]
  fn completed_scenario_advances_automatically() {
    let mut host = SimHost::new();

    for _ in 0..256 {
      host.step_once();
      if host.snapshot.scenario.id != IRON_BARS_SCENARIO {
        break;
      }
    }

    assert_eq!(IRON_BARS_FLEET_SCENARIO, host.snapshot.scenario.id);
    assert_eq!(1, host.completed_scenarios);
    assert_eq!(0, host.snapshot.tick);
    assert_eq!(0, host.idle_streak);
  }
}
