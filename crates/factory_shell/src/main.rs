use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use factory_content::{
  ContentDatabase, ScenarioId, BUILDING_DEPLOYMENT_SCENARIO, BUILDING_MATERIALS_SCENARIO,
  DEPLOYMENT_DEMO_SCENARIO, DISTRIBUTED_CHAIN_SCENARIO, HYBRID_GRID_SCENARIO,
  IRON_BARS_FLEET_SCENARIO, IRON_BARS_SCENARIO, PATHFINDING_DEMO_SCENARIO,
  POWERED_IRONWORKS_SCENARIO, POWER_LINE_SCENARIO, PRODUCTION_CHAIN_SCENARIO, V2_WORLD_SCENARIO,
};
use factory_sim::{
  AlertHistory, BatteryOwner, DispatchPhase, DispatchReceiverState, GameState, GridPosition,
  HaulerId, HaulerSnapshot, NodeId, TickSnapshot,
};
use std::collections::{BTreeMap, VecDeque};

const NORMAL_TICKS_PER_SECOND: f32 = 2.0;
const FAST_TICKS_PER_SECOND: f32 = 8.0;
const MAX_TICKS_PER_FRAME: u8 = 8;
const AUTO_ADVANCE_IDLE_TICKS: u16 = 8;
const MAX_RECENT_EVENTS: usize = 10;
const HUD_VALUE_MAX_CHARS: usize = 52;
const ACTIVITY_ENTRY_MAX_CHARS: usize = 52;
const ROUTE_DASH_COUNT: usize = 5;
const ROUTE_DASH_SPEED: f32 = 0.42;
const CRAFT_GAUGE_WIDTH: f32 = 96.0;
const POWER_GAUGE_WIDTH: f32 = 96.0;
const OUTPUT_CHIP_COUNT: usize = 5;
const MAX_OUTPUT_CHIPS: usize = 15;
const OUTPUT_CHIP_LIFETIME: f32 = 0.55;
const DEMO_SCENARIOS: [ScenarioId; 12] = [
  IRON_BARS_SCENARIO,
  IRON_BARS_FLEET_SCENARIO,
  BUILDING_MATERIALS_SCENARIO,
  POWERED_IRONWORKS_SCENARIO,
  DEPLOYMENT_DEMO_SCENARIO,
  PATHFINDING_DEMO_SCENARIO,
  PRODUCTION_CHAIN_SCENARIO,
  DISTRIBUTED_CHAIN_SCENARIO,
  POWER_LINE_SCENARIO,
  BUILDING_DEPLOYMENT_SCENARIO,
  HYBRID_GRID_SCENARIO,
  V2_WORLD_SCENARIO,
];
const GRID_X: f32 = 180.0;
const GRID_Y: f32 = 120.0;
const WORLD_LEFT: f32 = -410.0;
const CAMERA_UI_OFFSET_Y: f32 = 52.0;
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
const NODE_POWER_IDLE: Color = Color::srgb(0.35, 0.24, 0.25);
const NODE_POWER_ACTIVE: Color = Color::srgb(0.92, 0.43, 0.18);
const NODE_POWER_CHARGED: Color = Color::srgb(0.96, 0.76, 0.22);
const NODE_RADAR_IDLE: Color = Color::srgb(0.28, 0.31, 0.48);
const NODE_RADAR_CLAIMED: Color = Color::srgb(0.58, 0.49, 0.92);
const NODE_BUILD_SITE: Color = Color::srgb(0.42, 0.35, 0.56);
const NODE_STRUCTURE: Color = Color::srgb(0.64, 0.38, 0.72);
const GRID_OBSTACLE: Color = Color::srgb(0.42, 0.24, 0.18);
const GRID_POWER_LINE: Color = Color::srgb(0.26, 0.78, 0.92);
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
const POWER_GAUGE_BACKGROUND: Color = Color::srgb(0.16, 0.10, 0.08);
const POWER_GAUGE_FILL: Color = Color::srgb(1.0, 0.72, 0.18);
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
    .init_resource::<PlayerView>()
    .init_resource::<ProjectionScene>()
    .init_resource::<ProductionFeedback>()
    .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        handle_controls,
        handle_player_view,
        handle_control_buttons,
        advance_simulation,
        rebuild_projection,
        project_snapshot,
        project_activity,
        project_craft_gauge,
        project_power_gauge,
        emit_output_chips,
        animate_activity,
        animate_output_chips,
        animate_haulers,
        update_text,
        sync_annotation_visibility,
        update_focus_alert,
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
  annotations_visible: bool,
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
      annotations_visible: true,
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
      ControlAction::ToggleAnnotations => self.annotations_visible = !self.annotations_visible,
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

#[derive(Resource)]
struct PlayerView {
  position: GridPosition,
  zoom_level: u8,
  scene_revision: u64,
}

impl Default for PlayerView {
  fn default() -> Self {
    Self {
      position: GridPosition { x: 0, y: 0 },
      zoom_level: 1,
      scene_revision: u64::MAX,
    }
  }
}

#[derive(Component)]
struct ProjectionEntity;

#[derive(Component)]
struct NodeVisual(NodeId);

#[derive(Component)]
struct NodeLabel(NodeId);

#[derive(Component)]
struct HaulerVisual(HaulerId);

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
struct HaulerLabel(HaulerId);

#[derive(Component)]
struct HaulerTarget(Vec2);

#[derive(Component)]
struct CargoBadge(HaulerId);

#[derive(Component)]
struct CraftGaugeFill {
  node: NodeId,
  left: f32,
  max_width: f32,
}

#[derive(Component)]
struct PowerGaugeFill {
  node: NodeId,
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
struct HudTitleText;

#[derive(Copy, Clone)]
enum HudField {
  Flow,
  Stock,
  Logistics,
  World,
  Power,
}

#[derive(Component)]
struct HudValueText(HudField);

#[derive(Component)]
struct EventText;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct PlayerCursor;

#[derive(Component)]
struct FocusAlertText;

#[derive(Component)]
struct Annotation;

#[derive(Component)]
struct ControlDeckContent;

#[derive(Component)]
struct ControlDeck;

#[derive(Component)]
struct DeckToggleLabel;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
enum ControlAction {
  TogglePause,
  Step,
  Reset,
  ToggleSpeed,
  ToggleAutoCycle,
  ToggleAnnotations,
  SelectScenario(usize),
}

impl ControlAction {
  fn is_selected(self, host: &SimHost) -> bool {
    match self {
      Self::TogglePause => host.paused,
      Self::ToggleSpeed => host.ticks_per_second == FAST_TICKS_PER_SECOND,
      Self::ToggleAutoCycle => host.auto_cycle,
      Self::ToggleAnnotations => !host.annotations_visible,
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
  commands.spawn((Camera2d, MainCamera));

  spawn_projection(&mut commands, &host.snapshot);
  projection_scene.revision = host.scene_revision;

  spawn_status_panels(&mut commands);
  commands.spawn((
    Sprite::from_color(
      Color::srgba(0.95, 0.86, 0.32, 0.28),
      Vec2::new(150.0, 100.0),
    ),
    Transform::from_xyz(0.0, 0.0, 2.7),
    PlayerCursor,
    Annotation,
  ));
  spawn_control_deck(&mut commands);
}

fn spawn_status_panels(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        left: px(18),
        top: px(18),
        width: px(540),
        flex_direction: FlexDirection::Column,
        row_gap: px(2),
        padding: UiRect::all(px(12)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.92)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(90),
      Annotation,
    ))
    .with_children(|panel| {
      panel.spawn((
        Text::new("FACTORY GAME"),
        TextFont {
          font_size: FontSize::Px(14.0),
          ..default()
        },
        TextColor(Color::srgb(0.91, 0.92, 0.94)),
        Node {
          width: percent(100),
          ..default()
        },
        HudText,
        HudTitleText,
      ));
      for (label, field) in [
        ("FLOW", HudField::Flow),
        ("STOCK", HudField::Stock),
        ("LOGISTICS", HudField::Logistics),
        ("WORLD", HudField::World),
        ("POWER", HudField::Power),
      ] {
        spawn_status_row(panel, label, field);
      }
    });

  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        right: px(18),
        top: px(18),
        width: px(420),
        padding: UiRect::all(px(12)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.92)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(90),
      Annotation,
    ))
    .with_children(|panel| {
      panel.spawn((
        Text::new(""),
        TextFont {
          font_size: FontSize::Px(12.0),
          ..default()
        },
        TextLayout::no_wrap(),
        TextColor(Color::srgb(0.72, 0.76, 0.82)),
        Node {
          width: percent(100),
          overflow: Overflow::clip_x(),
          ..default()
        },
        EventText,
      ));
    });

  commands.spawn((
    Text::new(""),
    TextFont {
      font_size: FontSize::Px(12.0),
      ..default()
    },
    TextLayout::no_wrap(),
    TextColor(Color::srgb(1.0, 0.72, 0.42)),
    Node {
      position_type: PositionType::Absolute,
      left: px(18),
      bottom: px(18),
      width: px(540),
      padding: UiRect::axes(px(12), px(8)),
      border: UiRect::all(px(1)),
      overflow: Overflow::clip_x(),
      ..default()
    },
    BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.92)),
    BorderColor::all(BUTTON_PRESSED),
    GlobalZIndex(90),
    Visibility::Hidden,
    FocusAlertText,
    Annotation,
  ));
}

fn spawn_status_row(parent: &mut ChildSpawnerCommands, label: &'static str, field: HudField) {
  parent
    .spawn(Node {
      width: percent(100),
      flex_direction: FlexDirection::Row,
      column_gap: px(10),
      align_items: AlignItems::FlexStart,
      ..default()
    })
    .with_children(|row| {
      row.spawn((
        Text::new(label),
        TextFont {
          font_size: FontSize::Px(12.0),
          ..default()
        },
        TextColor(Color::srgb(0.62, 0.68, 0.76)),
        Node {
          width: px(84),
          flex_shrink: 0.0,
          ..default()
        },
      ));
      row.spawn((
        Text::new(""),
        TextFont {
          font_size: FontSize::Px(12.0),
          ..default()
        },
        TextLayout::no_wrap(),
        TextColor(Color::srgb(0.91, 0.92, 0.94)),
        Node {
          flex_grow: 1.0,
          min_width: px(0),
          overflow: Overflow::clip_x(),
          ..default()
        },
        HudText,
        HudValueText(field),
      ));
    });
}

fn spawn_control_deck(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        right: px(18),
        bottom: px(18),
        width: px(408),
        flex_direction: FlexDirection::Column,
        row_gap: px(5),
        padding: UiRect::all(px(8)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.94)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(100),
      ControlDeck,
    ))
    .with_children(|panel| {
      panel.spawn((
        Button,
        ControlButton(ControlAction::ToggleAnnotations),
        Node {
          height: px(30),
          border: UiRect::all(px(1)),
          justify_content: JustifyContent::Center,
          align_items: AlignItems::Center,
          padding: UiRect::axes(px(5), px(2)),
          ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        BorderColor::all(BUTTON_BORDER),
        children![(
          Text::new("HIDE UI"),
          TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
          },
          TextColor(Color::srgb(0.92, 0.94, 0.97)),
          DeckToggleLabel,
        )],
      ));
      panel
        .spawn((
          Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
          },
          ControlDeckContent,
        ))
        .with_children(|content| {
          content.spawn((
            Text::new("CONTROL DECK"),
            TextFont {
              font_size: FontSize::Px(12.0),
              ..default()
            },
            TextColor(Color::srgb(0.72, 0.76, 0.82)),
          ));
          spawn_control_row(
            content,
            &[
              (ControlAction::TogglePause, "PLAY / PAUSE"),
              (ControlAction::Step, "STEP"),
              (ControlAction::Reset, "RESET"),
              (ControlAction::ToggleSpeed, "SPEED"),
              (ControlAction::ToggleAutoCycle, "AUTO"),
            ],
          );
          spawn_control_row(
            content,
            &[
              (ControlAction::SelectScenario(6), "DRILL CHAIN"),
              (ControlAction::SelectScenario(7), "FREIGHT LINE"),
              (ControlAction::SelectScenario(8), "GRID LINK"),
              (ControlAction::SelectScenario(9), "BUILD"),
              (ControlAction::SelectScenario(10), "HYBRID"),
              (ControlAction::SelectScenario(11), "V2 WORLD"),
            ],
          );
          spawn_control_row(
            content,
            &[
              (ControlAction::SelectScenario(0), "IRON"),
              (ControlAction::SelectScenario(1), "FLEET"),
              (ControlAction::SelectScenario(2), "MATERIALS"),
              (ControlAction::SelectScenario(3), "POWER"),
              (ControlAction::SelectScenario(4), "DEPLOY"),
              (ControlAction::SelectScenario(5), "DETOUR"),
            ],
          );
        });
    });
}

fn spawn_control_row(parent: &mut ChildSpawnerCommands, buttons: &[(ControlAction, &'static str)]) {
  parent
    .spawn(Node {
      width: percent(100),
      height: px(30),
      column_gap: px(4),
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
            padding: UiRect::axes(px(5), px(2)),
            ..default()
          },
          BackgroundColor(BUTTON_NORMAL),
          BorderColor::all(BUTTON_BORDER),
          children![(
            Text::new(*label),
            TextFont {
              font_size: FontSize::Px(11.0),
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
  for power_line in &snapshot.topology.power_lines {
    let position = grid_to_world(*power_line);
    commands.spawn((
      Sprite::from_color(GRID_POWER_LINE, Vec2::new(78.0, 16.0)),
      Transform::from_xyz(position.x, position.y, 0.9),
      ProjectionEntity,
    ));
    commands.spawn((
      Text2d::new("POWER"),
      TextFont {
        font_size: FontSize::Px(11.0),
        ..default()
      },
      TextColor(Color::srgb(0.82, 0.97, 1.0)),
      Transform::from_xyz(position.x, position.y, 1.0),
      Annotation,
      ProjectionEntity,
    ));
  }
  for obstacle in &snapshot.topology.obstacles {
    let position = grid_to_world(*obstacle);
    commands.spawn((
      Sprite::from_color(GRID_OBSTACLE, Vec2::splat(76.0)),
      Transform::from_xyz(position.x, position.y, 0.8)
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
      ProjectionEntity,
    ));
    commands.spawn((
      Text2d::new("BLOCKED"),
      TextFont {
        font_size: FontSize::Px(13.0),
        ..default()
      },
      TextColor(Color::srgb(1.0, 0.76, 0.52)),
      Transform::from_xyz(position.x, position.y, 1.0),
      Annotation,
      ProjectionEntity,
    ));
  }
  for node in &snapshot.topology.nodes {
    let position = grid_to_world(node.position);
    let size = match node.id {
      NodeId::Source(_) => Vec2::new(124.0, 74.0),
      NodeId::Road => Vec2::new(100.0, 34.0),
      NodeId::Factory(_) => Vec2::new(132.0, 82.0),
      NodeId::Generator(_) => Vec2::new(132.0, 82.0),
      NodeId::Radar(_) => Vec2::new(132.0, 82.0),
      NodeId::BuildSite(_) => Vec2::new(124.0, 74.0),
      NodeId::Structure(_) => Vec2::new(132.0, 82.0),
      NodeId::Transit(_) => Vec2::new(28.0, 28.0),
    };
    commands.spawn((
      Sprite::from_color(node_color(snapshot, node.id), size),
      Transform::from_xyz(position.x, position.y, 1.0),
      NodeVisual(node.id),
      ProjectionEntity,
    ));
    let label_anchor = if matches!(node.id, NodeId::Road | NodeId::Transit(_)) {
      Anchor::BOTTOM_CENTER
    } else {
      Anchor::BOTTOM_LEFT
    };
    let label_x = if matches!(node.id, NodeId::Road | NodeId::Transit(_)) {
      position.x
    } else {
      position.x - size.x / 2.0
    };
    commands.spawn((
      Text2d::new(node_label_value(snapshot, node.id)),
      TextFont {
        font_size: FontSize::Px(15.0),
        ..default()
      },
      TextColor(Color::srgb(0.96, 0.96, 0.94)),
      Transform::from_xyz(label_x, position.y + size.y / 2.0 + 8.0, 3.0),
      label_anchor,
      NodeLabel(node.id),
      Annotation,
      ProjectionEntity,
    ));
    if matches!(node.id, NodeId::Factory(_)) {
      spawn_craft_gauge(commands, snapshot, node.id, position);
    } else if matches!(node.id, NodeId::Generator(_)) {
      spawn_power_gauge(commands, snapshot, node.id, position);
    }
  }

  for hauler in &snapshot.haulers {
    let position = hauler_world_position(snapshot, hauler);
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
      Annotation,
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
      Annotation,
      HaulerTarget(Vec2::new(position.x, position.y - 28.0)),
      ProjectionEntity,
    ));
  }
}

fn spawn_craft_gauge(
  commands: &mut Commands,
  snapshot: &TickSnapshot,
  node: NodeId,
  factory: Vec2,
) {
  let Some(factory_snapshot) = snapshot
    .factories
    .iter()
    .find(|factory| factory.node == node)
  else {
    return;
  };
  let y = factory.y - 28.0;
  let left = factory.x - CRAFT_GAUGE_WIDTH / 2.0;
  let progress = craft_progress_fraction(
    factory_snapshot.craft.craft_progress,
    factory_snapshot.craft.craft_time,
  );
  let width = CRAFT_GAUGE_WIDTH * progress;
  commands.spawn((
    Sprite::from_color(
      CRAFT_GAUGE_BACKGROUND,
      Vec2::new(CRAFT_GAUGE_WIDTH + 4.0, 10.0),
    ),
    Transform::from_xyz(factory.x, y, 1.5),
    Annotation,
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
      node,
      left,
      max_width: CRAFT_GAUGE_WIDTH,
    },
    Annotation,
    ProjectionEntity,
  ));
}

fn spawn_power_gauge(
  commands: &mut Commands,
  snapshot: &TickSnapshot,
  node: NodeId,
  generator_position: Vec2,
) {
  let Some(generator) = snapshot.power.as_ref().and_then(|power| {
    power
      .generators
      .iter()
      .find(|generator| generator.node == node)
  }) else {
    return;
  };
  let y = generator_position.y - 28.0;
  let left = generator_position.x - POWER_GAUGE_WIDTH / 2.0;
  let width = POWER_GAUGE_WIDTH * power_fraction(generator.energy, generator.capacity);
  commands.spawn((
    Sprite::from_color(
      POWER_GAUGE_BACKGROUND,
      Vec2::new(POWER_GAUGE_WIDTH + 4.0, 10.0),
    ),
    Transform::from_xyz(generator_position.x, y, 1.5),
    Annotation,
    ProjectionEntity,
  ));
  commands.spawn((
    Sprite::from_color(POWER_GAUGE_FILL, Vec2::new(width, 6.0)),
    Transform::from_xyz(left + width / 2.0, y, 1.6),
    if width > 0.0 {
      Visibility::Visible
    } else {
      Visibility::Hidden
    },
    PowerGaugeFill {
      node,
      left,
      max_width: POWER_GAUGE_WIDTH,
    },
    Annotation,
    ProjectionEntity,
  ));
}

fn spawn_connections(commands: &mut Commands, snapshot: &TickSnapshot) {
  if snapshot.topology.width > 20 || snapshot.topology.height > 20 {
    return;
  }
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
      Sprite::from_color(
        route_color(snapshot, node.id),
        Vec2::new(delta.length(), 5.0),
      ),
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
    (
      KeyCode::KeyC,
      ControlAction::SelectScenario((host.scenario_index + 1) % DEMO_SCENARIOS.len()),
    ),
    (KeyCode::KeyL, ControlAction::ToggleAutoCycle),
  ] {
    if keys.just_pressed(key) {
      host.apply_control(action);
    }
  }
}

fn handle_player_view(
  keys: Res<ButtonInput<KeyCode>>,
  mut mouse_wheel: MessageReader<MouseWheel>,
  host: Res<SimHost>,
  mut view: ResMut<PlayerView>,
  mut camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
  mut cursor: Single<&mut Transform, (With<PlayerCursor>, Without<MainCamera>)>,
) {
  if view.scene_revision != host.scene_revision {
    view.position = initial_player_position(&host.snapshot);
    view.zoom_level = 1;
    view.scene_revision = host.scene_revision;
  }

  let pan_distance = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
    10
  } else {
    1
  };
  let move_right = keys.just_pressed(KeyCode::KeyD)
    || keys.just_pressed(KeyCode::ArrowRight);
  let move_left = keys.just_pressed(KeyCode::KeyA)
    || keys.just_pressed(KeyCode::ArrowLeft);
  let move_up = keys.just_pressed(KeyCode::KeyW)
    || keys.just_pressed(KeyCode::ArrowUp);
  let move_down = keys.just_pressed(KeyCode::KeyS)
    || keys.just_pressed(KeyCode::ArrowDown);
  let horizontal = pan_distance * (i32::from(move_right) - i32::from(move_left));
  let vertical = pan_distance * (i32::from(move_up) - i32::from(move_down));
  view.position = move_player_focus(
    view.position,
    horizontal,
    vertical,
    host.snapshot.topology.width,
    host.snapshot.topology.height,
  );

  let wheel_delta = mouse_wheel.read().map(|event| event.y).sum::<f32>();
  if keys.just_pressed(KeyCode::KeyE) || wheel_delta > 0.0 {
    view.zoom_level = view.zoom_level.saturating_sub(1).max(1);
  }
  if keys.just_pressed(KeyCode::KeyQ) || wheel_delta < 0.0 {
    view.zoom_level = view.zoom_level.saturating_add(1).min(10);
  }
  if keys.just_pressed(KeyCode::KeyO) {
    view.zoom_level = if view.zoom_level == 10 { 1 } else { 10 };
  }

  let world = grid_to_world(view.position);
  camera.0.translation.x = world.x;
  camera.0.translation.y = world.y - CAMERA_UI_OFFSET_Y;
  cursor.translation.x = world.x;
  cursor.translation.y = world.y;
  if let Projection::Orthographic(projection) = &mut *camera.1 {
    projection.scale = player_zoom_scale(
      view.zoom_level,
      host.snapshot.topology.width,
      host.snapshot.topology.height,
    );
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
      target.0 = hauler_world_position(&host.snapshot, hauler);
    }
  }

  for (label, mut target) in &mut hauler_labels {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == label.0)
    {
      let position = hauler_world_position(&host.snapshot, hauler);
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
      target.0 = hauler_world_position(&host.snapshot, hauler);
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

  for (gauge, mut sprite, mut transform, mut visibility) in &mut gauges {
    let Some(factory) = host
      .snapshot
      .factories
      .iter()
      .find(|factory| factory.node == gauge.node)
    else {
      continue;
    };
    let progress = craft_progress_fraction(factory.craft.craft_progress, factory.craft.craft_time);
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

fn project_power_gauge(
  host: Res<SimHost>,
  mut gauges: Query<(
    &PowerGaugeFill,
    &mut Sprite,
    &mut Transform,
    &mut Visibility,
  )>,
) {
  if !host.is_changed() {
    return;
  }
  let Some(power) = &host.snapshot.power else {
    return;
  };
  for (gauge, mut sprite, mut transform, mut visibility) in &mut gauges {
    let Some(generator) = power
      .generators
      .iter()
      .find(|generator| generator.node == gauge.node)
    else {
      continue;
    };
    let progress = power_fraction(generator.energy, generator.capacity);
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
    .filter(|node| matches!(node.id, NodeId::Factory(_)))
    .last()
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
  mut route_dashes: Query<(&RouteDash, &mut Transform, &mut Visibility), Without<NodeVisual>>,
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

fn sync_annotation_visibility(
  host: Res<SimHost>,
  mut annotations: Query<&mut Visibility, With<Annotation>>,
  mut deck: Single<&mut Node, (With<ControlDeck>, Without<ControlDeckContent>)>,
  mut content: Single<&mut Node, (With<ControlDeckContent>, Without<ControlDeck>)>,
  mut toggle_labels: Query<&mut Text, With<DeckToggleLabel>>,
) {
  let visibility = if host.annotations_visible {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
  for mut annotation in &mut annotations {
    *annotation = visibility;
  }

  deck.width = if host.annotations_visible {
    px(408)
  } else {
    px(104)
  };
  content.display = if host.annotations_visible {
    Display::Flex
  } else {
    Display::None
  };
  for mut label in &mut toggle_labels {
    *label = Text::new(if host.annotations_visible {
      "HIDE UI"
    } else {
      "SHOW UI"
    });
  }
}

fn update_focus_alert(
  host: Res<SimHost>,
  view: Res<PlayerView>,
  mut alert: Single<(&mut Text, &mut Visibility), With<FocusAlertText>>,
) {
  let value = focused_alert(&host.snapshot, view.position);
  let visible = host.annotations_visible && !value.is_empty();
  *alert.0 = Text::new(value);
  *alert.1 = if visible {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
}

fn update_text(
  host: Res<SimHost>,
  mut node_labels: Query<(&NodeLabel, &mut Text2d), (Without<HudText>, Without<EventText>)>,
  mut hauler_labels: Query<
    (&HaulerLabel, &mut Text2d),
    (Without<NodeLabel>, Without<HudText>, Without<EventText>),
  >,
  mut hud_title: Query<
    &mut Text,
    (
      With<HudTitleText>,
      Without<HudValueText>,
      Without<EventText>,
    ),
  >,
  mut hud_values: Query<(&HudValueText, &mut Text), (Without<HudTitleText>, Without<EventText>)>,
  mut events: Query<&mut Text, (With<EventText>, Without<HudText>)>,
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
  let totals = snapshot_inventory_totals(&host.snapshot);
  let power = host
    .snapshot
    .power
    .as_ref()
    .map(|power| format!("{}/{}", power.energy, power.capacity))
    .unwrap_or_else(|| "off-grid".into());
  for mut text in &mut hud_title {
    *text = Text::new(format!("FACTORY GAME  /  {}", host.snapshot.scenario.name));
  }
  for (field, mut text) in &mut hud_values {
    let value = match field.0 {
      HudField::Flow => format!(
        "mined {}  |  crafted {}",
        format_items(&metrics.mined),
        format_items(&metrics.crafted)
      ),
      HudField::Stock => format_items(&totals),
      HudField::Logistics => format!(
        "{} dispatched  |  {} idle  |  {} haulers",
        metrics.dispatches_assigned,
        metrics.idle_ticks,
        host.snapshot.haulers.len()
      ),
      HudField::World => format!(
        "{} sources  |  {} factories  |  {} radars  |  {} built",
        host.snapshot.sources.len(),
        host.snapshot.factories.len(),
        host.snapshot.radars.len(),
        host.snapshot.structures.len()
      ),
      HudField::Power => format!(
        "{}  |  {} used  |  {} starved",
        power, metrics.energy_consumed, metrics.power_starvations
      ),
    };
    *text = Text::new(truncate_for_display(&value, HUD_VALUE_MAX_CHARS));
  }

  let event_value = if host.recent_events.is_empty() {
    "RECENT ACTIVITY\nwaiting for first tick".into()
  } else {
    let mut visible_events = host
      .recent_events
      .iter()
      .rev()
      .take(MAX_RECENT_EVENTS)
      .map(|event| truncate_for_display(event, ACTIVITY_ENTRY_MAX_CHARS))
      .collect::<Vec<_>>();
    visible_events.reverse();
    format!("RECENT ACTIVITY\n{}", visible_events.join("\n"))
  };
  for mut text in &mut events {
    *text = Text::new(event_value.clone());
  }
}

fn truncate_for_display(value: &str, max_chars: usize) -> String {
  if value.chars().count() <= max_chars {
    return value.into();
  }
  if max_chars <= 3 {
    return ".".repeat(max_chars);
  }

  let visible_chars = max_chars - 3;
  format!(
    "{}...",
    value.chars().take(visible_chars).collect::<String>()
  )
}

fn move_player_focus(
  position: GridPosition,
  x: i32,
  y: i32,
  width: i32,
  height: i32,
) -> GridPosition {
  GridPosition {
    x: (position.x + x).clamp(0, width.saturating_sub(1)),
    y: (position.y + y).clamp(0, height.saturating_sub(1)),
  }
}

fn initial_player_position(snapshot: &TickSnapshot) -> GridPosition {
  if snapshot.topology.width <= 20 && snapshot.topology.height <= 20 {
    return GridPosition {
      x: snapshot.topology.width / 2,
      y: snapshot.topology.height / 2,
    };
  }
  let factory_positions = snapshot
    .topology
    .nodes
    .iter()
    .filter(|node| matches!(node.id, NodeId::Factory(_)))
    .map(|node| node.position)
    .collect::<Vec<_>>();
  let min_x = factory_positions.iter().map(|position| position.x).min();
  let max_x = factory_positions.iter().map(|position| position.x).max();
  let min_y = factory_positions.iter().map(|position| position.y).min();
  let max_y = factory_positions.iter().map(|position| position.y).max();
  match (min_x, max_x, min_y, max_y) {
    (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) => GridPosition {
      x: (min_x + max_x) / 2,
      y: (min_y + max_y) / 2,
    },
    _ => GridPosition {
      x: snapshot.topology.width / 2,
      y: snapshot.topology.height / 2,
    },
  }
}

fn player_zoom_scale(level: u8, width: i32, height: i32) -> f32 {
  let level = level.clamp(1, 10);
  if width <= 10 && height <= 10 {
    return 1.0 + f32::from(level - 1) * 0.18;
  }
  let overview = (width as f32 / 7.0).max(height as f32 / 6.0).max(2.62);
  overview.powf(f32::from(level - 1) / 9.0)
}

fn node_alerts(snapshot: &TickSnapshot, node: NodeId) -> Option<&AlertHistory> {
  match node {
    NodeId::Source(_) => snapshot
      .sources
      .iter()
      .find(|source| source.node == node)
      .map(|source| &source.alerts),
    NodeId::Factory(_) => snapshot
      .factories
      .iter()
      .find(|factory| factory.node == node)
      .map(|factory| &factory.alerts),
    NodeId::Generator(_) => snapshot.power.as_ref().and_then(|power| {
      power
        .generators
        .iter()
        .find(|generator| generator.node == node)
        .map(|generator| &generator.alerts)
    }),
    NodeId::Radar(_) => snapshot
      .radars
      .iter()
      .find(|radar| radar.node == node)
      .map(|radar| &radar.alerts),
    NodeId::Structure(_) => snapshot
      .structures
      .iter()
      .find(|structure| structure.node == node)
      .map(|structure| &structure.alerts),
    NodeId::Road | NodeId::BuildSite(_) | NodeId::Transit(_) => None,
  }
}

fn focused_alert(snapshot: &TickSnapshot, position: GridPosition) -> String {
  let node_alerts = snapshot
    .topology
    .nodes
    .iter()
    .filter(|node| node.position == position)
    .filter_map(|node| {
      node_alerts(snapshot, node.id)
        .and_then(AlertHistory::latest)
        .map(|alert| (alert.tick, format!("{}: {}", node.id, alert.message)))
    });
  let hauler_alerts = snapshot
    .haulers
    .iter()
    .filter(|hauler| hauler.position_grid == position)
    .filter_map(|hauler| {
      hauler.alerts.latest().map(|alert| {
        (
          alert.tick,
          format!("hauler-{}: {}", hauler.id, alert.message),
        )
      })
    });
  node_alerts
    .chain(hauler_alerts)
    .max_by_key(|(tick, _)| *tick)
    .map(|(tick, message)| {
      truncate_for_display(&format!("ALERT t{tick:03}  {message}"), HUD_VALUE_MAX_CHARS)
    })
    .unwrap_or_default()
}

#[cfg(test)]
fn focused_status(snapshot: &TickSnapshot, position: GridPosition) -> String {
  let mut details = snapshot
    .topology
    .nodes
    .iter()
    .filter(|node| node.position == position)
    .map(|node| node_label_value(snapshot, node.id).replace('\n', " | "))
    .collect::<Vec<_>>();
  details.extend(
    snapshot
      .haulers
      .iter()
      .filter(|hauler| hauler.position_grid == position)
      .map(hauler_label_value),
  );
  if details.is_empty() {
    if snapshot.topology.obstacles.contains(&position) {
      "inspect: blocked".into()
    } else {
      "inspect: empty".into()
    }
  } else {
    format!("inspect: {}", details.join(" || "))
  }
}

fn snapshot_inventory_totals(snapshot: &TickSnapshot) -> BTreeMap<String, u32> {
  let mut totals = BTreeMap::new();
  for items in snapshot
    .sources
    .iter()
    .map(|source| &source.stockpile.items)
    .chain(snapshot.haulers.iter().map(|hauler| &hauler.cargo.items))
    .chain(
      snapshot
        .factories
        .iter()
        .map(|factory| &factory.inventory.items),
    )
    .chain(snapshot.power.iter().flat_map(|power| {
      power
        .generators
        .iter()
        .map(|generator| &generator.fuel.items)
    }))
  {
    for (item, quantity) in items {
      *totals.entry(item.clone()).or_default() += quantity;
    }
  }
  totals
}

fn grid_to_world(position: GridPosition) -> Vec2 {
  Vec2::new(
    WORLD_LEFT + position.x as f32 * GRID_X,
    position.y as f32 * GRID_Y,
  )
}

fn hauler_world_position(snapshot: &TickSnapshot, hauler: &HaulerSnapshot) -> Vec2 {
  let position = grid_to_world(hauler.position_grid);
  let stack_index = snapshot
    .haulers
    .iter()
    .filter(|candidate| candidate.position_grid == hauler.position_grid && candidate.id < hauler.id)
    .count();
  Vec2::new(position.x, position.y - 54.0 - stack_index as f32 * 30.0)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodeActivity {
  Idle,
  Ready,
  Demanding,
  Crafting,
  Powering,
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

fn power_fraction(energy: u32, capacity: u32) -> f32 {
  if capacity == 0 {
    0.0
  } else {
    (energy as f32 / capacity as f32).clamp(0.0, 1.0)
  }
}

fn crafted_output_delta(previous: &BTreeMap<String, u32>, current: &BTreeMap<String, u32>) -> u32 {
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
        source.deployed
          && (!source.stockpile.items.is_empty() || !source.dispatch.intents.is_empty())
      })
      .map_or(NodeActivity::Idle, |_| NodeActivity::Ready),
    NodeId::Road => NodeActivity::Idle,
    NodeId::Factory(_) => snapshot
      .factories
      .iter()
      .find(|factory| factory.node == node)
      .map_or(NodeActivity::Idle, |factory| {
        if factory.craft.crafting {
          NodeActivity::Crafting
        } else if !factory.dispatch.intents.is_empty() {
          NodeActivity::Demanding
        } else {
          NodeActivity::Idle
        }
      }),
    NodeId::Generator(_) => snapshot
      .power
      .as_ref()
      .and_then(|power| {
        power
          .generators
          .iter()
          .find(|generator| generator.node == node)
      })
      .map_or(NodeActivity::Idle, |generator| {
        if snapshot
          .events
          .iter()
          .any(|event| event.starts_with(&format!("power generate {} ", generator.node)))
        {
          NodeActivity::Powering
        } else if generator.energy > 0 {
          NodeActivity::Ready
        } else {
          NodeActivity::Demanding
        }
      }),
    NodeId::Radar(_) => snapshot
      .radars
      .iter()
      .find(|radar| radar.node == node)
      .map_or(NodeActivity::Idle, |radar| {
        if radar.claimed_target.is_some() {
          NodeActivity::Ready
        } else {
          NodeActivity::Idle
        }
      }),
    NodeId::BuildSite(_) => NodeActivity::Demanding,
    NodeId::Structure(_) => NodeActivity::Ready,
    NodeId::Transit(_) => NodeActivity::Idle,
  }
}

fn hauler_activity(hauler: &HaulerSnapshot) -> HaulerActivity {
  match &hauler.dispatch {
    DispatchReceiverState::Assigned(assignment)
      if matches!(
        assignment.phase,
        DispatchPhase::Collect | DispatchPhase::Retrieve
      ) =>
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
    (NodeId::Factory(_), NodeActivity::Crafting) => NODE_FACTORY_CRAFTING,
    (NodeId::Factory(_), NodeActivity::Demanding) => NODE_FACTORY_DEMAND,
    (NodeId::Factory(_), _) => NODE_FACTORY_IDLE,
    (NodeId::Generator(_), NodeActivity::Powering) => NODE_POWER_ACTIVE,
    (NodeId::Generator(_), NodeActivity::Ready) => NODE_POWER_CHARGED,
    (NodeId::Generator(_), _) => NODE_POWER_IDLE,
    (NodeId::Radar(_), NodeActivity::Ready) => NODE_RADAR_CLAIMED,
    (NodeId::Radar(_), _) => NODE_RADAR_IDLE,
    (NodeId::BuildSite(_), _) => NODE_BUILD_SITE,
    (NodeId::Structure(_), _) => NODE_STRUCTURE,
    (NodeId::Transit(_), _) => NODE_ROAD,
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
      DispatchReceiverState::Assigned(assignment)
        if assignment.phase == DispatchPhase::Retrieve && assignment.source == node =>
      {
        Some(RouteDirection::TowardRoad)
      }
      DispatchReceiverState::Assigned(assignment)
        if assignment.phase == DispatchPhase::Deploy && assignment.destination == node =>
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
  let label = match node {
    NodeId::Source(_) => snapshot
      .sources
      .iter()
      .find(|source| source.node == node)
      .map(|source| {
        if source.exhausted && !source.deployed {
          format!("{}\nore site: exhausted", source.node)
        } else if source.exhausted {
          format!(
            "{}\ndraining: {}",
            source.node,
            format_items(&source.stockpile.items)
          )
        } else if source.deployed {
          format!(
            "{}\nstock: {}",
            source.node,
            format_items(&source.stockpile.items)
          )
        } else {
          format!("{}\nore site: awaiting drill", source.node)
        }
      })
      .unwrap_or_else(|| node.to_string()),
    NodeId::Road => "road".into(),
    NodeId::Factory(_) => snapshot
      .factories
      .iter()
      .find(|factory| factory.node == node)
      .map(|factory| {
        format!(
          "{}\noutput: {}\nstock: {}\ncraft: {}/{} {}",
          node,
          factory.craft.output_item,
          format_items(&factory.inventory.items),
          factory.craft.craft_progress,
          factory.craft.craft_time,
          if factory.craft.crafting {
            "active"
          } else {
            "idle"
          }
        )
      })
      .unwrap_or_else(|| node.to_string()),
    NodeId::Generator(_) => snapshot
      .power
      .as_ref()
      .and_then(|power| {
        power
          .generators
          .iter()
          .find(|generator| generator.node == node)
      })
      .map(|generator| {
        format!(
          "{}\nmode: {}\nfuel: {}",
          generator.node,
          if generator.fuel_item.is_some() {
            "fuel"
          } else {
            "fuel-free"
          },
          format_items(&generator.fuel.items)
        )
      })
      .unwrap_or_else(|| node.to_string()),
    NodeId::Radar(_) => snapshot
      .radars
      .iter()
      .find(|radar| radar.node == node)
      .map(|radar| {
        format!(
          "{}\ndeploy: {}\ntarget: {}\nclaim: {}",
          radar.node,
          radar.deployment_item,
          radar.target_item,
          radar
            .claimed_target
            .map_or_else(|| "none".into(), |target| target.to_string())
        )
      })
      .unwrap_or_else(|| node.to_string()),
    NodeId::BuildSite(index) => format!("build-site-{index}\nawaiting structure"),
    NodeId::Structure(_) => snapshot
      .structures
      .iter()
      .find(|structure| structure.node == node)
      .map(|structure| format!("{}\nspawned: {}", structure.node, structure.item))
      .unwrap_or_else(|| node.to_string()),
    NodeId::Transit(position) => format!("transit\n{}, {}", position.x, position.y),
  };
  let battery = snapshot.power.as_ref().and_then(|power| {
    power
      .batteries
      .iter()
      .find(|battery| battery.owner == BatteryOwner::Node(node))
  });
  battery.map_or(label.clone(), |battery| {
    format!("{label}\nbattery: {}/{}", battery.energy, battery.capacity)
  })
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
        DispatchPhase::Retrieve => "retrieve",
        DispatchPhase::Deploy => "deploy",
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

    host.apply_control(ControlAction::ToggleAnnotations);
    assert!(!host.annotations_visible);
    assert!(ControlAction::ToggleAnnotations.is_selected(&host));

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
  fn focus_mode_hides_annotations_and_collapses_the_control_deck() {
    let mut host = SimHost::new();
    host.annotations_visible = false;
    let mut app = App::new();
    app.insert_resource(host);
    app.add_systems(Update, sync_annotation_visibility);
    let annotation = app
      .world_mut()
      .spawn((Annotation, Visibility::Visible))
      .id();
    let deck = app.world_mut().spawn((Node::default(), ControlDeck)).id();
    let content = app
      .world_mut()
      .spawn((Node::default(), ControlDeckContent))
      .id();
    let label = app
      .world_mut()
      .spawn((Text::new("HIDE UI"), DeckToggleLabel))
      .id();

    app.update();

    assert_eq!(
      Visibility::Hidden,
      *app.world().get::<Visibility>(annotation).unwrap()
    );
    assert_eq!(px(104), app.world().get::<Node>(deck).unwrap().width);
    assert_eq!(
      Display::None,
      app.world().get::<Node>(content).unwrap().display
    );
    assert_eq!("SHOW UI", app.world().get::<Text>(label).unwrap().as_str());
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
    assert!(host
      .recent_events
      .back()
      .expect("scenario event is recorded")
      .contains("test selected"));
  }

  #[test]
  fn display_truncation_uses_three_dots_and_stays_within_the_limit() {
    assert_eq!("short", truncate_for_display("short", 8));
    assert_eq!("abcde...", truncate_for_display("abcdefghijk", 8));
    assert_eq!("..", truncate_for_display("long", 2));
    assert_eq!(8, truncate_for_display("abcdefghijk", 8).chars().count());
  }

  #[test]
  fn presentation_state_tracks_authoritative_material_flow() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    host.select_scenario(2, "test selected");
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
      saw_crafting |= node_activity(&host.snapshot, NodeId::Factory(0)) == NodeActivity::Crafting;
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
        route_direction(&host.snapshot, NodeId::Factory(0)) == Some(RouteDirection::AwayFromRoad);
    }

    assert!(saw_toward_road);
    assert!(saw_away_from_road);
  }

  #[test]
  fn deployment_projection_tracks_dormant_source_and_drill_route() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    host.select_scenario(4, "test selected");
    let source = host.snapshot.sources[0].node;

    assert!(node_label_value(&host.snapshot, source).contains("awaiting drill"));
    host.step_once();
    assert_eq!(
      Some(RouteDirection::TowardRoad),
      route_direction(&host.snapshot, NodeId::Factory(0))
    );
    let mut saw_deploy_route = false;
    for _ in 0..6 {
      host.step_once();
      saw_deploy_route |=
        route_direction(&host.snapshot, source) == Some(RouteDirection::AwayFromRoad);
    }
    assert!(saw_deploy_route);
  }

  #[test]
  fn v2_radar_projection_exposes_claimed_targets() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    host.select_scenario(11, "test selected");
    host.step_once();

    let radar = NodeId::Radar(0);
    assert!(host
      .snapshot
      .topology
      .nodes
      .iter()
      .any(|node| node.id == radar));
    assert_eq!(NodeActivity::Ready, node_activity(&host.snapshot, radar));
    let label = node_label_value(&host.snapshot, radar);
    assert!(label.contains("deploy: mining_drill"));
    assert!(label.contains("target: iron_ore"));
    assert!(label.contains("claim: source-"));
  }

  #[test]
  fn power_line_projection_tracks_generated_grid_cells() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    host.select_scenario(8, "test selected");

    assert!(host.snapshot.topology.power_lines.is_empty());
    host.step_once();

    assert_eq!(3, host.snapshot.topology.power_lines.len());
    assert!(node_label_value(&host.snapshot, NodeId::Factory(0)).contains("battery:"));
    assert!(host
      .snapshot
      .events
      .iter()
      .any(|event| event.starts_with("power line generator-0 built")));
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
  fn player_focus_moves_one_cell_and_clamps_to_the_world() {
    assert_eq!(
      GridPosition { x: 2, y: 1 },
      move_player_focus(GridPosition { x: 1, y: 1 }, 1, 0, 3, 2)
    );
    assert_eq!(
      GridPosition { x: 0, y: 1 },
      move_player_focus(GridPosition { x: 0, y: 1 }, -1, 1, 3, 2)
    );
  }

  #[test]
  fn player_zoom_preserves_the_unity_one_to_ten_bounds() {
    assert_eq!(1.0, player_zoom_scale(1, 100, 100));
    assert_eq!(
      player_zoom_scale(1, 100, 100),
      player_zoom_scale(0, 100, 100)
    );
    assert_eq!(
      player_zoom_scale(10, 100, 100),
      player_zoom_scale(11, 100, 100)
    );
    assert!(player_zoom_scale(10, 100, 100) > 16.0);
  }

  #[test]
  fn large_world_focus_starts_on_the_factory_district() {
    let game = scenario_game(V2_WORLD_SCENARIO);
    let snapshot = game.snapshot(Vec::new());

    assert_eq!(GridPosition { x: 54, y: 51 }, initial_player_position(&snapshot));
  }

  #[test]
  fn focused_status_and_totals_follow_the_authoritative_snapshot() {
    let game = scenario_game(BUILDING_DEPLOYMENT_SCENARIO);
    let snapshot = game.snapshot(Vec::new());

    assert!(focused_status(&snapshot, GridPosition { x: 4, y: 1 }).contains("awaiting structure"));
    assert_eq!(
      Some(&1),
      snapshot_inventory_totals(&snapshot).get("storage_warehouse")
    );
  }

  #[test]
  fn focused_alert_uses_the_latest_authoritative_object_history() {
    let mut game = scenario_game(IRON_BARS_SCENARIO);
    game.world.factories[0]
      .alerts
      .record(7, "product output full");
    let snapshot = game.snapshot(Vec::new());
    let position = snapshot
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Factory(0))
      .unwrap()
      .position;

    assert_eq!(
      "ALERT t007  factory-0: product output full",
      focused_alert(&snapshot, position)
    );
  }

  #[test]
  fn focus_alert_overlay_is_contextual_and_honors_hidden_ui() {
    let mut host = SimHost::new();
    host.auto_cycle = false;
    host.game.world.factories[0]
      .alerts
      .record(7, "product output full");
    host.snapshot = host.game.snapshot(Vec::new());
    let position = host
      .snapshot
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Factory(0))
      .unwrap()
      .position;
    let view = PlayerView {
      position,
      zoom_level: 1,
      scene_revision: 0,
    };
    let mut app = App::new();
    app.insert_resource(host);
    app.insert_resource(view);
    app.add_systems(Update, update_focus_alert);
    let overlay = app
      .world_mut()
      .spawn((Text::new(""), Visibility::Hidden, FocusAlertText))
      .id();

    app.update();
    assert_eq!(
      "ALERT t007  factory-0: product output full",
      app.world().get::<Text>(overlay).unwrap().as_str()
    );
    assert_eq!(
      Visibility::Visible,
      *app.world().get::<Visibility>(overlay).unwrap()
    );

    app
      .world_mut()
      .resource_mut::<SimHost>()
      .annotations_visible = false;
    app.update();
    assert_eq!(
      Visibility::Hidden,
      *app.world().get::<Visibility>(overlay).unwrap()
    );
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
    assert_eq!(POWERED_IRONWORKS_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(DEPLOYMENT_DEMO_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(PATHFINDING_DEMO_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(PRODUCTION_CHAIN_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(DISTRIBUTED_CHAIN_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(POWER_LINE_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(BUILDING_DEPLOYMENT_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(HYBRID_GRID_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
    host.next_scenario("test");
    assert_eq!(IRON_BARS_SCENARIO, host.snapshot.scenario.id);
    assert_eq!(12, host.scene_revision);
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
