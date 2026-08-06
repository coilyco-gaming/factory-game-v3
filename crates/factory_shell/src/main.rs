use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use factory_content::{
  ContentDatabase, ItemId, ScenarioId, COAL, COAL_PLANT, COPPER_ORE, IRON_BARS, IRON_ORE,
  MINING_DRILL, STONE, STORAGE_WAREHOUSE, V2_WORLD_SCENARIO,
};
use factory_sim::{
  BatteryOwner, DispatchPhase, DispatchReceiverState, GameState, GridPosition, HaulerId,
  HaulerSnapshot, NodeId, TickSnapshot, TopologyNode,
};
use std::collections::{BTreeMap, BTreeSet};

const NORMAL_TICKS_PER_SECOND: f32 = 2.0;
const FAST_TICKS_PER_SECOND: f32 = 8.0;
const MAX_TICKS_PER_FRAME: u8 = 8;
const ROUTE_DASH_COUNT: usize = 5;
const ROUTE_DASH_SPEED: f32 = 0.42;
const CRAFT_GAUGE_WIDTH: f32 = 96.0;
const POWER_GAUGE_WIDTH: f32 = 96.0;
const OUTPUT_CHIP_COUNT: usize = 5;
const MAX_OUTPUT_CHIPS: usize = 15;
const OUTPUT_CHIP_LIFETIME: f32 = 0.55;
const WORLD_SCENARIOS: [ScenarioId; 1] = [V2_WORLD_SCENARIO];
const MIN_ZOOM_LEVEL: u8 = 1;
const MAX_ZOOM_LEVEL: u8 = 10;
const MAX_DETAIL_ZOOM_LEVEL: u8 = 3;
const MIN_VISIBLE_CELLS: f32 = 10.0;
const INPUT_REPEAT_DELAY_SECONDS: f32 = 0.25;
const INPUT_REPEAT_INTERVAL_SECONDS: f32 = 0.08;
const MAX_INPUT_REPEATS_PER_FRAME: u8 = 4;
const NODE_ART_SIZE: f32 = 100.0;
const DRILL_ART_SIZE: f32 = 76.0;
const TRUCK_ART_SIZE: f32 = 72.0;
const CARGO_ART_SIZE: f32 = 28.0;
const GRID_X: f32 = 180.0;
const GRID_Y: f32 = 120.0;
const WORLD_LEFT: f32 = -410.0;
const CAMERA_UI_OFFSET_Y: f32 = 64.0;
const CONTROL_DECK_WIDTH: f32 = 360.0;
const COLLAPSED_CONTROL_DECK_WIDTH: f32 = 104.0;
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
const CARGO_EMPTY: Color = Color::srgb(0.10, 0.13, 0.18);
const CARGO_LOADED: Color = Color::srgb(0.98, 0.92, 0.58);
const CRAFT_GAUGE_BACKGROUND: Color = Color::srgb(0.10, 0.16, 0.14);
const CRAFT_GAUGE_FILL: Color = Color::srgb(0.58, 0.96, 0.62);
const POWER_GAUGE_BACKGROUND: Color = Color::srgb(0.16, 0.10, 0.08);
const POWER_GAUGE_FILL: Color = Color::srgb(1.0, 0.72, 0.18);
const OUTPUT_CHIP_STONE: Color = Color::srgb(0.62, 0.58, 0.48);
const OUTPUT_CHIP_STEEL: Color = Color::srgb(0.48, 0.55, 0.58);
const GROUND_ART: &str = "factory/terrain/ground.png";
const ROAD_ART: &str = "factory/logistics/road-straight-ns.png";
const TRUCK_ART: &str = "factory/vehicles/truck.png";
const IRON_DEPOSIT_ART: &str = "factory/resources/iron-ore-deposit.png";
const COPPER_DEPOSIT_ART: &str = "factory/resources/copper-ore-deposit.png";
const COAL_DEPOSIT_ART: &str = "factory/resources/coal-deposit.png";
const STONE_DEPOSIT_ART: &str = "factory/resources/stone-deposit.png";
const FOUNDRY_ART: &str = "factory/machines/foundry.png";
const FACTORY_ART: &str = "factory/machines/factory.png";
const COAL_PLANT_ART: &str = "factory/machines/coal-plant.png";
const RADAR_ART: &str = "factory/machines/radar.png";
const MINING_DRILL_ART: &str = "factory/machines/mining-drill.png";
const WAREHOUSE_ART: &str = "factory/structures/warehouse.png";
const IRON_ORE_ART: &str = "factory/items/iron-ore.png";
const IRON_BARS_ART: &str = "factory/items/iron-bars.png";

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
    .init_resource::<FactoryArt>()
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
        project_deployed_art,
        project_craft_gauge,
        project_power_gauge,
        emit_output_chips,
        animate_activity,
        animate_output_chips,
        animate_haulers,
        update_text,
        sync_annotation_visibility,
        sync_world_detail_visibility,
        style_control_buttons,
      )
        .chain(),
    )
    .run();
}

#[derive(Resource)]
struct FactoryArt {
  ground: Handle<Image>,
  road_straight_ns: Handle<Image>,
  truck: Handle<Image>,
  iron_ore_deposit: Handle<Image>,
  copper_ore_deposit: Handle<Image>,
  coal_deposit: Handle<Image>,
  stone_deposit: Handle<Image>,
  foundry: Handle<Image>,
  factory: Handle<Image>,
  coal_plant: Handle<Image>,
  radar: Handle<Image>,
  mining_drill: Handle<Image>,
  warehouse: Handle<Image>,
  iron_ore: Handle<Image>,
  iron_bars: Handle<Image>,
}

impl FromWorld for FactoryArt {
  fn from_world(world: &mut World) -> Self {
    let assets = world.resource::<AssetServer>();
    Self {
      ground: assets.load(GROUND_ART),
      road_straight_ns: assets.load(ROAD_ART),
      truck: assets.load(TRUCK_ART),
      iron_ore_deposit: assets.load(IRON_DEPOSIT_ART),
      copper_ore_deposit: assets.load(COPPER_DEPOSIT_ART),
      coal_deposit: assets.load(COAL_DEPOSIT_ART),
      stone_deposit: assets.load(STONE_DEPOSIT_ART),
      foundry: assets.load(FOUNDRY_ART),
      factory: assets.load(FACTORY_ART),
      coal_plant: assets.load(COAL_PLANT_ART),
      radar: assets.load(RADAR_ART),
      mining_drill: assets.load(MINING_DRILL_ART),
      warehouse: assets.load(WAREHOUSE_ART),
      iron_ore: assets.load(IRON_ORE_ART),
      iron_bars: assets.load(IRON_BARS_ART),
    }
  }
}

impl FactoryArt {
  fn item(&self, item: ItemId) -> Option<&Handle<Image>> {
    match item {
      IRON_ORE => Some(&self.iron_ore),
      IRON_BARS => Some(&self.iron_bars),
      _ => None,
    }
  }

  fn node(&self, kind: NodeArtKind) -> (&Handle<Image>, Quat) {
    match kind {
      NodeArtKind::Deposit(IRON_ORE) => (&self.iron_ore_deposit, Quat::IDENTITY),
      NodeArtKind::Deposit(COPPER_ORE) => (&self.copper_ore_deposit, Quat::IDENTITY),
      NodeArtKind::Deposit(COAL) => (&self.coal_deposit, Quat::IDENTITY),
      NodeArtKind::Deposit(STONE) => (&self.stone_deposit, Quat::IDENTITY),
      NodeArtKind::Deposit(_) => unreachable!("node art selects only accepted deposit items"),
      NodeArtKind::Foundry => (&self.foundry, Quat::IDENTITY),
      NodeArtKind::Factory => (&self.factory, Quat::IDENTITY),
      NodeArtKind::CoalPlant => (&self.coal_plant, Quat::IDENTITY),
      NodeArtKind::Radar => (&self.radar, Quat::IDENTITY),
      NodeArtKind::Warehouse => (&self.warehouse, Quat::IDENTITY),
      NodeArtKind::Road(RoadOrientation::NorthSouth) => (&self.road_straight_ns, Quat::IDENTITY),
      NodeArtKind::Road(RoadOrientation::EastWest) => (
        &self.road_straight_ns,
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
      ),
    }
  }
}

#[derive(Resource)]
struct SimHost {
  game: GameState,
  snapshot: TickSnapshot,
  paused: bool,
  ticks_per_second: f32,
  accumulated_seconds: f32,
  world_index: usize,
  annotations_visible: bool,
  scene_revision: u64,
  snapshot_revision: u64,
}

impl SimHost {
  fn new() -> Self {
    Self::for_scenario(WORLD_SCENARIOS[0])
  }

  fn for_scenario(scenario: ScenarioId) -> Self {
    let game = scenario_game(scenario);
    let snapshot = game.snapshot(Vec::new());
    Self {
      game,
      snapshot,
      paused: false,
      ticks_per_second: NORMAL_TICKS_PER_SECOND,
      accumulated_seconds: 0.0,
      world_index: 0,
      annotations_visible: true,
      scene_revision: 0,
      snapshot_revision: 1,
    }
  }

  fn step_once(&mut self) {
    self.snapshot = self.game.step();
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
  }

  fn reset(&mut self) {
    self.game = scenario_game(WORLD_SCENARIOS[self.world_index]);
    self.snapshot = self.game.snapshot(Vec::new());
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
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
      ControlAction::ToggleAnnotations => self.annotations_visible = !self.annotations_visible,
      ControlAction::SelectWorld(index) => {
        self.select_world(index);
      }
    }
  }

  fn select_world(&mut self, index: usize) {
    self.world_index = index % WORLD_SCENARIOS.len();
    self.game = scenario_game(WORLD_SCENARIOS[self.world_index]);
    self.snapshot = self.game.snapshot(Vec::new());
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
    self.accumulated_seconds = 0.0;
    self.scene_revision += 1;
  }
}

fn scenario_game(scenario: ScenarioId) -> GameState {
  GameState::new(ContentDatabase::starter(), scenario).expect("viewer scenario is valid")
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
      zoom_level: MAX_ZOOM_LEVEL,
      scene_revision: u64::MAX,
    }
  }
}

#[derive(Default)]
struct InputRepeatState<T> {
  direction: T,
  held_seconds: f32,
  next_repeat_seconds: f32,
}

#[derive(Component)]
struct ProjectionEntity;

#[derive(Component)]
struct NodeVisual(NodeId);

#[derive(Component)]
struct NodeActivityVisual(NodeActivity);

#[derive(Component)]
struct NodeFallback;

#[derive(Component)]
struct NodeLabel(NodeId);

#[derive(Component)]
struct DrillArt(NodeId);

#[derive(Component)]
struct HaulerArt(HaulerId);

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
struct CargoArt(HaulerId);

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HudField {
  Resources,
  Materials,
  Power,
}

#[derive(Component)]
struct HudValueText(HudField);

#[derive(Component)]
struct StatusBar;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct PlayerCursor;

#[derive(Component)]
struct Annotation;

#[derive(Component)]
struct WorldDetail;

#[derive(Component)]
struct ControlDeckContent;

#[derive(Component)]
struct ControlDeck;

#[derive(Component)]
struct DeckToggleLabel;

#[derive(Component)]
struct DeckTitle;

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
enum ControlAction {
  TogglePause,
  Step,
  Reset,
  ToggleSpeed,
  ToggleAnnotations,
  SelectWorld(usize),
}

impl ControlAction {
  fn is_selected(self, host: &SimHost) -> bool {
    match self {
      Self::TogglePause => host.paused,
      Self::ToggleSpeed => host.ticks_per_second == FAST_TICKS_PER_SECOND,
      Self::ToggleAnnotations => !host.annotations_visible,
      Self::SelectWorld(index) => host.world_index == index,
      Self::Step | Self::Reset => false,
    }
  }
}

#[derive(Component)]
struct ControlButton(ControlAction);

fn setup(
  mut commands: Commands,
  host: Res<SimHost>,
  art: Res<FactoryArt>,
  mut projection_scene: ResMut<ProjectionScene>,
) {
  commands.spawn((Camera2d, MainCamera));

  spawn_projection(&mut commands, &host.snapshot, &art);
  projection_scene.revision = host.scene_revision;

  spawn_status_bar(&mut commands);
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

fn spawn_status_bar(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: px(88),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Stretch,
        padding: UiRect::axes(px(18), px(10)),
        border: UiRect {
          bottom: px(1),
          ..default()
        },
        ..default()
      },
      BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.96)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(90),
      Annotation,
      StatusBar,
    ))
    .with_children(|panel| {
      panel
        .spawn(Node {
          width: px(220),
          height: percent(100),
          flex_shrink: 0.0,
          flex_direction: FlexDirection::Column,
          justify_content: JustifyContent::Center,
          row_gap: px(3),
          padding: UiRect {
            right: px(16),
            ..default()
          },
          ..default()
        })
        .with_children(|title| {
          title.spawn((
            Text::new("FACTORY GAME"),
            TextFont {
              font_size: FontSize::Px(16.0),
              ..default()
            },
            TextColor(Color::srgb(0.94, 0.96, 0.98)),
          ));
          title.spawn((
            Text::new(""),
            TextFont {
              font_size: FontSize::Px(10.0),
              ..default()
            },
            TextLayout::no_wrap(),
            TextColor(BUTTON_ACTIVE),
            Node {
              width: percent(100),
              overflow: Overflow::clip_x(),
              ..default()
            },
            HudText,
            HudTitleText,
          ));
        });
      panel
        .spawn((
          Node {
            height: percent(100),
            min_width: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            border: UiRect {
              left: px(1),
              ..default()
            },
            overflow: Overflow::clip(),
            ..default()
          },
          BorderColor::all(Color::srgba(0.34, 0.39, 0.48, 0.58)),
        ))
        .with_children(|stack| {
          for (index, (label, field, accent)) in status_metrics().into_iter().enumerate() {
            spawn_status_metric(stack, label, field, accent, index > 0);
          }
        });
    });
}

fn status_metrics() -> [(&'static str, HudField, Color); 3] {
  [
    (
      "RESOURCES",
      HudField::Resources,
      Color::srgb(0.96, 0.58, 0.28),
    ),
    (
      "MATERIALS",
      HudField::Materials,
      Color::srgb(0.95, 0.78, 0.36),
    ),
    ("POWER", HudField::Power, Color::srgb(0.48, 0.88, 0.62)),
  ]
}

fn spawn_status_metric(
  parent: &mut ChildSpawnerCommands,
  label: &'static str,
  field: HudField,
  accent: Color,
  divided: bool,
) {
  parent
    .spawn((
      status_metric_node(divided),
      BorderColor::all(Color::srgba(0.34, 0.39, 0.48, 0.58)),
    ))
    .with_children(|metric| {
      metric.spawn((
        Text::new(label),
        TextFont {
          font_size: FontSize::Px(10.0),
          ..default()
        },
        TextColor(accent),
        Node {
          width: px(64),
          flex_shrink: 0.0,
          ..default()
        },
      ));
      metric.spawn((
        Text::new(""),
        TextFont {
          font_size: FontSize::Px(10.5),
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

fn status_metric_node(divided: bool) -> Node {
  Node {
    width: percent(100),
    min_width: px(0),
    min_height: px(0),
    flex_grow: 1.0,
    flex_direction: FlexDirection::Row,
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    column_gap: px(12),
    padding: UiRect::axes(px(12), px(0)),
    border: UiRect {
      top: px(if divided { 1 } else { 0 }),
      ..default()
    },
    overflow: Overflow::clip(),
    ..default()
  }
}

fn spawn_control_deck(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        right: px(18),
        bottom: px(18),
        width: px(CONTROL_DECK_WIDTH),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        padding: UiRect::all(px(10)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.96)),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(100),
      ControlDeck,
    ))
    .with_children(|panel| {
      panel
        .spawn(Node {
          width: percent(100),
          height: px(30),
          flex_direction: FlexDirection::Row,
          align_items: AlignItems::Center,
          column_gap: px(8),
          ..default()
        })
        .with_children(|header| {
          header.spawn((
            Text::new("FACTORY CONTROL"),
            TextFont {
              font_size: FontSize::Px(11.0),
              ..default()
            },
            TextColor(BUTTON_ACTIVE),
            Node {
              flex_grow: 1.0,
              ..default()
            },
            DeckTitle,
          ));
          header.spawn((
            Button,
            ControlButton(ControlAction::ToggleAnnotations),
            Node {
              width: px(92),
              height: percent(100),
              flex_shrink: 0.0,
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
                font_size: FontSize::Px(11.0),
                ..default()
              },
              TextColor(Color::srgb(0.92, 0.94, 0.97)),
              DeckToggleLabel,
            )],
          ));
        });
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
          spawn_control_row(
            content,
            &[
              (ControlAction::TogglePause, "PLAY / PAUSE"),
              (ControlAction::Step, "STEP"),
              (ControlAction::Reset, "RESET"),
              (ControlAction::ToggleSpeed, "SPEED"),
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

fn spawn_projection(commands: &mut Commands, snapshot: &TickSnapshot, art: &FactoryArt) {
  spawn_ground(commands, snapshot, art);
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
      WorldDetail,
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
      WorldDetail,
      ProjectionEntity,
    ));
  }
  for node in &snapshot.topology.nodes {
    let position = grid_to_world(node.position);
    let activity = node_activity(snapshot, node.id);
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
    match node_presentation(snapshot, node) {
      NodePresentation::Art(kind) => spawn_node_art(
        commands,
        art,
        node.id,
        activity,
        kind,
        position,
        NODE_ART_SIZE,
        1.2,
      ),
      NodePresentation::Fallback => {
        commands.spawn((
          Sprite::from_color(node_color_for_activity(node.id, activity), size),
          Transform::from_xyz(position.x, position.y, 1.0),
          NodeVisual(node.id),
          NodeActivityVisual(activity),
          NodeFallback,
          ProjectionEntity,
        ));
      }
    }
    if drill_art_candidate(snapshot, node) {
      spawn_drill_art(commands, art, snapshot, node, position);
    }
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
      WorldDetail,
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
    let mut truck = Sprite::from_image(art.truck.clone());
    truck.custom_size = Some(Vec2::splat(TRUCK_ART_SIZE));
    commands.spawn((
      truck,
      Transform::from_xyz(position.x, position.y, 2.2),
      HaulerArt(hauler.id),
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
      WorldDetail,
      HaulerTarget(position),
      ProjectionEntity,
    ));
    commands.spawn((
      cargo_art_sprite(art, hauler),
      Transform::from_xyz(position.x, position.y, 2.7),
      CargoArt(hauler.id),
      Annotation,
      WorldDetail,
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
      WorldDetail,
      HaulerTarget(Vec2::new(position.x, position.y - 28.0)),
      ProjectionEntity,
    ));
  }
}

fn spawn_node_art(
  commands: &mut Commands,
  art: &FactoryArt,
  node: NodeId,
  activity: NodeActivity,
  kind: NodeArtKind,
  position: Vec2,
  size: f32,
  z: f32,
) {
  let (image, rotation) = art.node(kind);
  let mut sprite = Sprite::from_image(image.clone());
  sprite.custom_size = Some(Vec2::splat(size));
  commands.spawn((
    sprite,
    Transform::from_xyz(position.x, position.y, z).with_rotation(rotation),
    NodeVisual(node),
    NodeActivityVisual(activity),
    ProjectionEntity,
  ));
}

fn spawn_drill_art(
  commands: &mut Commands,
  art: &FactoryArt,
  snapshot: &TickSnapshot,
  node: &TopologyNode,
  position: Vec2,
) {
  let mut sprite = Sprite::from_image(art.mining_drill.clone());
  configure_drill_art(&mut sprite, snapshot, node.id);
  commands.spawn((
    sprite,
    Transform::from_xyz(position.x, position.y, 1.3),
    DrillArt(node.id),
    ProjectionEntity,
  ));
}

fn spawn_ground(commands: &mut Commands, snapshot: &TickSnapshot, art: &FactoryArt) {
  let mut ground = Sprite::from_image(art.ground.clone());
  ground.custom_size = Some(world_art_size(
    snapshot.topology.width,
    snapshot.topology.height,
  ));
  let center = world_center(snapshot.topology.width, snapshot.topology.height);
  commands.spawn((
    ground,
    Transform::from_xyz(center.x, center.y, -2.0),
    ProjectionEntity,
  ));
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
    WorldDetail,
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
    WorldDetail,
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
    WorldDetail,
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
    WorldDetail,
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
      ControlAction::SelectWorld((host.world_index + 1) % WORLD_SCENARIOS.len()),
    ),
  ] {
    if keys.just_pressed(key) {
      host.apply_control(action);
    }
  }
}

fn handle_player_view(
  keys: Res<ButtonInput<KeyCode>>,
  time: Res<Time>,
  mut mouse_wheel: MessageReader<MouseWheel>,
  host: Res<SimHost>,
  mut view: ResMut<PlayerView>,
  mut pan_repeat: Local<InputRepeatState<IVec2>>,
  mut zoom_repeat: Local<InputRepeatState<i8>>,
  window: Single<&Window>,
  mut camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
  mut cursor: Single<&mut Transform, (With<PlayerCursor>, Without<MainCamera>)>,
) {
  if view.scene_revision != host.scene_revision {
    view.position = initial_player_position(&host.snapshot);
    view.zoom_level = MAX_ZOOM_LEVEL;
    view.scene_revision = host.scene_revision;
    *pan_repeat = InputRepeatState::default();
    *zoom_repeat = InputRepeatState::default();
  }

  let pan_distance = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
    10
  } else {
    1
  };
  let move_right = keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight);
  let move_left = keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft);
  let move_up = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
  let move_down = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
  let direction = IVec2::new(
    i32::from(move_right) - i32::from(move_left),
    i32::from(move_up) - i32::from(move_down),
  );
  let repeat_steps = repeated_input_steps(&mut pan_repeat, direction, time.delta_secs());
  let horizontal = pan_distance * direction.x * i32::from(repeat_steps);
  let vertical = pan_distance * direction.y * i32::from(repeat_steps);
  let next_position = move_player_focus(
    view.position,
    horizontal,
    vertical,
    host.snapshot.topology.width,
    host.snapshot.topology.height,
  );
  if next_position != view.position {
    view.position = next_position;
  }

  let wheel_delta = mouse_wheel.read().map(|event| event.y).sum::<f32>();
  let zoom_direction =
    i8::from(keys.pressed(KeyCode::KeyQ)) - i8::from(keys.pressed(KeyCode::KeyE));
  let zoom_steps = repeated_input_steps(&mut zoom_repeat, zoom_direction, time.delta_secs());
  let mut next_zoom = move_zoom_level(view.zoom_level, zoom_direction, zoom_steps);
  let wheel_direction = i8::from(wheel_delta < 0.0) - i8::from(wheel_delta > 0.0);
  next_zoom = move_zoom_level(next_zoom, wheel_direction, u8::from(wheel_direction != 0));
  if keys.just_pressed(KeyCode::KeyO) {
    next_zoom = if next_zoom == MAX_ZOOM_LEVEL {
      MIN_ZOOM_LEVEL
    } else {
      MAX_ZOOM_LEVEL
    };
  }
  if next_zoom != view.zoom_level {
    view.zoom_level = next_zoom;
  }

  let focused_world = grid_to_world(view.position);
  let camera_world = if view.zoom_level == MAX_ZOOM_LEVEL {
    world_center(host.snapshot.topology.width, host.snapshot.topology.height)
  } else {
    Vec2::new(focused_world.x, focused_world.y - CAMERA_UI_OFFSET_Y)
  };
  camera.0.translation.x = camera_world.x;
  camera.0.translation.y = camera_world.y;
  cursor.translation.x = focused_world.x;
  cursor.translation.y = focused_world.y;
  if let Projection::Orthographic(projection) = &mut *camera.1 {
    projection.scale = player_zoom_scale(
      view.zoom_level,
      host.snapshot.topology.width,
      host.snapshot.topology.height,
      window.width(),
      window.height(),
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
  art: Res<FactoryArt>,
  mut projection_scene: ResMut<ProjectionScene>,
  entities: Query<Entity, With<ProjectionEntity>>,
) {
  if projection_scene.revision == host.scene_revision {
    return;
  }

  for entity in &entities {
    commands.entity(entity).despawn();
  }
  spawn_projection(&mut commands, &host.snapshot, &art);
  projection_scene.revision = host.scene_revision;
}

fn project_snapshot(
  host: Res<SimHost>,
  mut last_snapshot_revision: Local<u64>,
  mut hauler_art: Query<
    (&HaulerArt, &mut HaulerTarget),
    (Without<HaulerLabel>, Without<CargoBadge>, Without<CargoArt>),
  >,
  mut hauler_labels: Query<
    (&HaulerLabel, &mut HaulerTarget),
    (Without<HaulerArt>, Without<CargoBadge>, Without<CargoArt>),
  >,
  mut cargo_badges: Query<
    (&CargoBadge, &mut HaulerTarget),
    (Without<HaulerArt>, Without<HaulerLabel>, Without<CargoArt>),
  >,
  mut cargo_art: Query<
    (&CargoArt, &mut HaulerTarget),
    (
      Without<HaulerArt>,
      Without<HaulerLabel>,
      Without<CargoBadge>,
    ),
  >,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
    return;
  }

  for (visual, mut target) in &mut hauler_art {
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

  for (cargo, mut target) in &mut cargo_art {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == cargo.0)
    {
      target.0 = hauler_world_position(&host.snapshot, hauler);
    }
  }
}

fn project_activity(
  host: Res<SimHost>,
  art: Res<FactoryArt>,
  mut last_snapshot_revision: Local<u64>,
  mut nodes: Query<
    (
      &NodeVisual,
      &mut NodeActivityVisual,
      &mut Sprite,
      Option<&NodeFallback>,
    ),
    (Without<RouteVisual>, Without<CargoBadge>, Without<CargoArt>),
  >,
  mut routes: Query<
    (&RouteVisual, &mut Sprite),
    (Without<NodeVisual>, Without<CargoBadge>, Without<CargoArt>),
  >,
  mut cargo_badges: Query<
    (&CargoBadge, &mut Sprite),
    (Without<NodeVisual>, Without<RouteVisual>, Without<CargoArt>),
  >,
  mut cargo_art: Query<
    (&CargoArt, &mut Sprite),
    (
      Without<NodeVisual>,
      Without<RouteVisual>,
      Without<CargoBadge>,
    ),
  >,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
    return;
  }

  for (visual, mut activity, mut sprite, fallback) in &mut nodes {
    activity.0 = node_activity(&host.snapshot, visual.0);
    if fallback.is_some() {
      let color = node_color_for_activity(visual.0, activity.0);
      if sprite.color != color {
        sprite.color = color;
      }
    }
  }
  for (visual, mut sprite) in &mut routes {
    sprite.color = route_color(&host.snapshot, visual.0);
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
  for (cargo, mut sprite) in &mut cargo_art {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == cargo.0)
    {
      configure_cargo_art(&mut sprite, &art, hauler);
    }
  }
}

fn project_deployed_art(
  host: Res<SimHost>,
  mut last_snapshot_revision: Local<u64>,
  mut drills: Query<(&DrillArt, &mut Sprite)>,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
    return;
  }

  for (drill, mut sprite) in &mut drills {
    configure_drill_art(&mut sprite, &host.snapshot, drill.0);
  }
}

fn project_craft_gauge(
  host: Res<SimHost>,
  mut last_snapshot_revision: Local<u64>,
  mut gauges: Query<(
    &CraftGaugeFill,
    &mut Sprite,
    &mut Transform,
    &mut Visibility,
  )>,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
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
  mut last_snapshot_revision: Local<u64>,
  mut gauges: Query<(
    &PowerGaugeFill,
    &mut Sprite,
    &mut Transform,
    &mut Visibility,
  )>,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
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
  mut last_snapshot_revision: Local<u64>,
  mut feedback: ResMut<ProductionFeedback>,
  chips: Query<(), With<OutputChip>>,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
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
  mut nodes: Query<(&NodeActivityVisual, &mut Transform), Without<RouteDash>>,
  mut route_dashes: Query<(&RouteDash, &mut Transform, &mut Visibility), Without<NodeVisual>>,
) {
  let elapsed = time.elapsed_secs();
  let pulse = 1.0 + 0.045 * (elapsed * 5.0).sin().max(0.0);
  for (activity, mut transform) in &mut nodes {
    let scale = if activity.0 == NodeActivity::Idle {
      Vec3::ONE
    } else {
      Vec3::splat(pulse)
    };
    if transform.scale != scale {
      transform.scale = scale;
    }
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
  mut last_visibility: Local<Option<bool>>,
  mut annotations: Query<
    &mut Visibility,
    (With<Annotation>, Without<WorldDetail>, Without<DeckTitle>),
  >,
  mut deck: Single<&mut Node, (With<ControlDeck>, Without<ControlDeckContent>)>,
  mut content: Single<&mut Node, (With<ControlDeckContent>, Without<ControlDeck>)>,
  mut deck_titles: Query<&mut Visibility, (With<DeckTitle>, Without<Annotation>)>,
  mut toggle_labels: Query<&mut Text, With<DeckToggleLabel>>,
) {
  if *last_visibility == Some(host.annotations_visible) {
    return;
  }
  *last_visibility = Some(host.annotations_visible);

  let visibility = if host.annotations_visible {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
  for mut annotation in &mut annotations {
    *annotation = visibility;
  }

  deck.width = if host.annotations_visible {
    px(CONTROL_DECK_WIDTH)
  } else {
    px(COLLAPSED_CONTROL_DECK_WIDTH)
  };
  content.display = if host.annotations_visible {
    Display::Flex
  } else {
    Display::None
  };
  for mut title in &mut deck_titles {
    *title = if host.annotations_visible {
      Visibility::Visible
    } else {
      Visibility::Hidden
    };
  }
  for mut label in &mut toggle_labels {
    *label = Text::new(if host.annotations_visible {
      "HIDE UI"
    } else {
      "SHOW UI"
    });
  }
}

fn sync_world_detail_visibility(
  host: Res<SimHost>,
  view: Res<PlayerView>,
  mut last_state: Local<Option<(bool, u8, u64)>>,
  mut details: Query<&mut Visibility, With<WorldDetail>>,
) {
  let state = (
    host.annotations_visible,
    view.zoom_level,
    host.scene_revision,
  );
  if *last_state == Some(state) {
    return;
  }
  *last_state = Some(state);

  let visibility = if world_detail_visible(host.annotations_visible, view.zoom_level) {
    Visibility::Visible
  } else {
    Visibility::Hidden
  };
  for mut detail in &mut details {
    *detail = visibility;
  }
}

fn update_text(
  host: Res<SimHost>,
  mut last_snapshot_revision: Local<u64>,
  mut node_labels: Query<(&NodeLabel, &mut Text2d), Without<HaulerLabel>>,
  mut hauler_labels: Query<(&HaulerLabel, &mut Text2d), Without<NodeLabel>>,
  mut hud_title: Query<&mut Text, (With<HudTitleText>, Without<HudValueText>)>,
  mut hud_values: Query<(&HudValueText, &mut Text), Without<HudTitleText>>,
) {
  if !claim_snapshot_revision(host.snapshot_revision, &mut last_snapshot_revision) {
    return;
  }

  for (label, mut text) in &mut node_labels {
    let value = node_label_value(&host.snapshot, label.0);
    if text.as_str() != value {
      *text = Text2d::new(value);
    }
  }

  for (label, mut text) in &mut hauler_labels {
    if let Some(hauler) = host
      .snapshot
      .haulers
      .iter()
      .find(|hauler| hauler.id == label.0)
    {
      let value = hauler_label_value(hauler);
      if text.as_str() != value {
        *text = Text2d::new(value);
      }
    }
  }

  let metrics = host.game.metrics();
  let totals = snapshot_inventory_totals(&host.snapshot);
  let (resources, materials) = split_stockpile_totals(host.game.content(), totals);
  let power = host
    .snapshot
    .power
    .as_ref()
    .map(|power| format!("{}/{}", power.energy, power.capacity))
    .unwrap_or_else(|| "off-grid".into());
  for mut text in &mut hud_title {
    *text = Text::new(host.snapshot.scenario.name.to_uppercase());
  }
  for (field, mut text) in &mut hud_values {
    let value = match field.0 {
      HudField::Resources => format_items(&resources),
      HudField::Materials => format_items(&materials),
      HudField::Power => format!(
        "{}  |  {} made  |  {} moved  |  {} used  |  {} starved",
        power,
        metrics.energy_generated,
        metrics.energy_balanced,
        metrics.energy_consumed,
        metrics.power_starvations
      ),
    };
    *text = Text::new(value);
  }
}

fn claim_snapshot_revision(current: u64, previous: &mut u64) -> bool {
  if *previous == current {
    return false;
  }
  *previous = current;
  true
}

fn world_detail_visible(annotations_visible: bool, zoom_level: u8) -> bool {
  annotations_visible && zoom_level <= MAX_DETAIL_ZOOM_LEVEL
}

fn repeated_input_steps<T>(state: &mut InputRepeatState<T>, direction: T, delta_seconds: f32) -> u8
where
  T: Copy + Default + PartialEq,
{
  if direction == T::default() {
    *state = InputRepeatState::default();
    return 0;
  }
  if direction != state.direction {
    state.direction = direction;
    state.held_seconds = 0.0;
    state.next_repeat_seconds = INPUT_REPEAT_DELAY_SECONDS;
    return 1;
  }

  state.held_seconds += delta_seconds.max(0.0);
  let mut repeats = 0;
  while state.held_seconds >= state.next_repeat_seconds && repeats < MAX_INPUT_REPEATS_PER_FRAME {
    repeats += 1;
    state.next_repeat_seconds += INPUT_REPEAT_INTERVAL_SECONDS;
  }
  if repeats == MAX_INPUT_REPEATS_PER_FRAME && state.held_seconds >= state.next_repeat_seconds {
    state.next_repeat_seconds = state.held_seconds + INPUT_REPEAT_INTERVAL_SECONDS;
  }
  repeats
}

fn move_zoom_level(level: u8, direction: i8, steps: u8) -> u8 {
  if direction < 0 {
    level.saturating_sub(steps).max(MIN_ZOOM_LEVEL)
  } else if direction > 0 {
    level.saturating_add(steps).min(MAX_ZOOM_LEVEL)
  } else {
    level
  }
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
  GridPosition {
    x: snapshot.topology.width / 2,
    y: snapshot.topology.height / 2,
  }
}

fn world_center(width: i32, height: i32) -> Vec2 {
  Vec2::new(
    WORLD_LEFT + width.saturating_sub(1) as f32 * GRID_X / 2.0,
    height.saturating_sub(1) as f32 * GRID_Y / 2.0,
  )
}

fn world_art_size(width: i32, height: i32) -> Vec2 {
  Vec2::new(width.max(1) as f32 * GRID_X, height.max(1) as f32 * GRID_Y)
}

fn player_zoom_scale(
  level: u8,
  width: i32,
  height: i32,
  viewport_width: f32,
  viewport_height: f32,
) -> f32 {
  let viewport_width = viewport_width.max(1.0);
  let viewport_height = viewport_height.max(1.0);
  let overview = (width.max(1) as f32 * GRID_X / viewport_width)
    .max(height.max(1) as f32 * GRID_Y / viewport_height);
  let detail = (MIN_VISIBLE_CELLS * GRID_X / viewport_width)
    .max(MIN_VISIBLE_CELLS * GRID_Y / viewport_height)
    .min(overview);
  let level = level.clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
  let progress = f32::from(level - MIN_ZOOM_LEVEL) / f32::from(MAX_ZOOM_LEVEL - MIN_ZOOM_LEVEL);
  detail * (overview / detail).powf(progress)
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

fn split_stockpile_totals(
  content: &ContentDatabase,
  totals: BTreeMap<String, u32>,
) -> (BTreeMap<String, u32>, BTreeMap<String, u32>) {
  let resource_items = content
    .items
    .values()
    .filter(|item| item.ingredients.is_empty())
    .map(|item| item.id.as_str())
    .collect::<BTreeSet<_>>();
  totals
    .into_iter()
    .partition(|(item, _)| resource_items.contains(item.as_str()))
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
enum RouteDirection {
  TowardRoad,
  AwayFromRoad,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RoadOrientation {
  NorthSouth,
  EastWest,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodeArtKind {
  Deposit(ItemId),
  Foundry,
  Factory,
  CoalPlant,
  Radar,
  Warehouse,
  Road(RoadOrientation),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NodePresentation {
  Art(NodeArtKind),
  Fallback,
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
        source.occupied_by.is_some()
          || (source.deployed
            && (!source.stockpile.items.is_empty() || !source.dispatch.intents.is_empty()))
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

fn node_color_for_activity(node: NodeId, activity: NodeActivity) -> Color {
  match (node, activity) {
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

fn cargo_art_item(hauler: &HaulerSnapshot) -> Option<ItemId> {
  [IRON_ORE, IRON_BARS].into_iter().find(|item| {
    hauler
      .cargo
      .items
      .get(item.as_str())
      .is_some_and(|quantity| *quantity > 0)
  })
}

fn cargo_art_sprite(art: &FactoryArt, hauler: &HaulerSnapshot) -> Sprite {
  let mut sprite = Sprite::default();
  configure_cargo_art(&mut sprite, art, hauler);
  sprite
}

fn configure_cargo_art(sprite: &mut Sprite, art: &FactoryArt, hauler: &HaulerSnapshot) {
  if let Some(image) = cargo_art_item(hauler).and_then(|item| art.item(item)) {
    sprite.image = image.clone();
    sprite.custom_size = Some(Vec2::splat(CARGO_ART_SIZE));
  } else {
    sprite.image = Handle::default();
    sprite.custom_size = Some(Vec2::ZERO);
  }
  sprite.color = Color::WHITE;
}

fn node_art_kind(snapshot: &TickSnapshot, node: &TopologyNode) -> Option<NodeArtKind> {
  match node.id {
    NodeId::Source(_) => snapshot
      .sources
      .iter()
      .find(|source| source.node == node.id)
      .filter(|source| matches!(source.item, IRON_ORE | COPPER_ORE | COAL | STONE))
      .map(|source| NodeArtKind::Deposit(source.item)),
    NodeId::Factory(_) => snapshot
      .factories
      .iter()
      .find(|factory| factory.node == node.id)
      .map(|factory| {
        if factory.craft.output_item == IRON_BARS {
          NodeArtKind::Foundry
        } else {
          NodeArtKind::Factory
        }
      }),
    NodeId::Generator(_) => snapshot.power.as_ref().and_then(|power| {
      power
        .generators
        .iter()
        .find(|generator| generator.node == node.id && generator.item == Some(COAL_PLANT))
        .map(|_| NodeArtKind::CoalPlant)
    }),
    NodeId::Radar(_) => snapshot
      .radars
      .iter()
      .any(|radar| radar.node == node.id)
      .then_some(NodeArtKind::Radar),
    NodeId::Structure(_) => snapshot
      .structures
      .iter()
      .find(|structure| structure.node == node.id && structure.item == STORAGE_WAREHOUSE)
      .map(|_| NodeArtKind::Warehouse),
    NodeId::Road => straight_road_orientation(snapshot, node.position).map(NodeArtKind::Road),
    _ => None,
  }
}

fn node_presentation(snapshot: &TickSnapshot, node: &TopologyNode) -> NodePresentation {
  node_art_kind(snapshot, node).map_or(NodePresentation::Fallback, NodePresentation::Art)
}

fn drill_art_candidate(snapshot: &TickSnapshot, node: &TopologyNode) -> bool {
  let Some(source) = snapshot
    .sources
    .iter()
    .find(|source| source.node == node.id)
  else {
    return false;
  };
  snapshot
    .radars
    .iter()
    .any(|radar| radar.deployment_item == MINING_DRILL && radar.target_item == source.item)
}

fn drill_art_visible(snapshot: &TickSnapshot, node: NodeId) -> bool {
  let Some(source) = snapshot.sources.iter().find(|source| source.node == node) else {
    return false;
  };
  source.deployed
    && source.occupied_by.is_none()
    && snapshot
      .radars
      .iter()
      .any(|radar| radar.deployment_item == MINING_DRILL && radar.target_item == source.item)
}

fn configure_drill_art(sprite: &mut Sprite, snapshot: &TickSnapshot, node: NodeId) {
  sprite.custom_size = Some(if drill_art_visible(snapshot, node) {
    Vec2::splat(DRILL_ART_SIZE)
  } else {
    Vec2::ZERO
  });
}

fn straight_road_orientation(
  snapshot: &TickSnapshot,
  position: GridPosition,
) -> Option<RoadOrientation> {
  if snapshot
    .topology
    .nodes
    .iter()
    .any(|node| node.position == position && node.id != NodeId::Road)
  {
    return None;
  }

  let occupied = |x: i32, y: i32| {
    snapshot
      .topology
      .nodes
      .iter()
      .any(|node| node.position == GridPosition { x, y })
  };
  let north = occupied(position.x, position.y + 1);
  let south = occupied(position.x, position.y - 1);
  let east = occupied(position.x + 1, position.y);
  let west = occupied(position.x - 1, position.y);

  match (north, south, east, west) {
    (true, true, false, false) => Some(RoadOrientation::NorthSouth),
    (false, false, true, true) => Some(RoadOrientation::EastWest),
    _ => None,
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
        if let Some(occupant) = source.occupied_by {
          format!("{}\nore site: occupied by {}", source.node, occupant)
        } else if source.exhausted && !source.deployed {
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
        let lines = snapshot
          .topology
          .generator_power_lines
          .iter()
          .filter(|line| line.generator == generator.node)
          .collect::<Vec<_>>();
        let link = match lines.as_slice() {
          [] => "pending".into(),
          [line] => format!("{} via {} cells", line.target, line.cells.len()),
          lines => format!(
            "{} links via {} cells",
            lines.len(),
            lines.iter().map(|line| line.cells.len()).sum::<usize>()
          ),
        };
        format!(
          "{}\ntype: {}\nmode: {}\nfuel: {}\nlink: {}",
          generator.node,
          generator.item.map_or("generator", |item| item.as_str()),
          if generator.fuel_item.is_some() {
            "fuel"
          } else {
            "fuel-free"
          },
          format_items(&generator.fuel.items),
          link
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
  use factory_content::{
    BUILDING_DEPLOYMENT_SCENARIO, BUILDING_MATERIALS_SCENARIO, DEPLOYMENT_DEMO_SCENARIO,
    IRON_BARS_SCENARIO, POWER_LINE_SCENARIO,
  };
  use factory_sim::GeneratorPowerLine;

  #[test]
  fn sim_host_steps_match_direct_simulation_bytes() {
    let mut host = SimHost::new();
    let mut direct = scenario_game(V2_WORLD_SCENARIO);

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
  fn snapshot_revision_gate_projects_each_authoritative_snapshot_once() {
    let mut host = SimHost::new();
    let mut projected = 0;

    assert!(claim_snapshot_revision(
      host.snapshot_revision,
      &mut projected
    ));
    assert!(!claim_snapshot_revision(
      host.snapshot_revision,
      &mut projected
    ));

    host.step_once();
    assert!(claim_snapshot_revision(
      host.snapshot_revision,
      &mut projected
    ));
    assert!(!claim_snapshot_revision(
      host.snapshot_revision,
      &mut projected
    ));
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

    host.apply_control(ControlAction::ToggleAnnotations);
    assert!(!host.annotations_visible);
    assert!(ControlAction::ToggleAnnotations.is_selected(&host));

    host.apply_control(ControlAction::SelectWorld(0));
    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
    assert!(ControlAction::SelectWorld(0).is_selected(&host));
    assert_eq!(1, host.scene_revision);

    host.apply_control(ControlAction::Reset);
    assert_eq!(0, host.snapshot.tick);
    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
  }

  #[test]
  fn pressed_button_routes_through_the_shared_control_action() {
    let mut app = App::new();
    app.insert_resource(SimHost::new());
    app.add_systems(Update, handle_control_buttons);
    app.world_mut().spawn((
      Button,
      Interaction::Pressed,
      ControlButton(ControlAction::SelectWorld(0)),
    ));

    app.update();

    let host = app.world().resource::<SimHost>();
    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
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
    let title = app
      .world_mut()
      .spawn((Text::new("FACTORY CONTROL"), Visibility::Visible, DeckTitle))
      .id();

    app.update();

    assert_eq!(
      Visibility::Hidden,
      *app.world().get::<Visibility>(annotation).unwrap()
    );
    assert_eq!(
      px(COLLAPSED_CONTROL_DECK_WIDTH),
      app.world().get::<Node>(deck).unwrap().width
    );
    assert_eq!(
      Display::None,
      app.world().get::<Node>(content).unwrap().display
    );
    assert_eq!("SHOW UI", app.world().get::<Text>(label).unwrap().as_str());
    assert_eq!(
      Visibility::Hidden,
      *app.world().get::<Visibility>(title).unwrap()
    );
  }

  #[test]
  fn status_bar_contains_only_resources_materials_and_power() {
    assert_eq!(
      ["RESOURCES", "MATERIALS", "POWER"],
      status_metrics().map(|(label, _, _)| label)
    );
    assert_eq!(
      [HudField::Resources, HudField::Materials, HudField::Power],
      status_metrics().map(|(_, field, _)| field)
    );

    let row = status_metric_node(false);
    assert_eq!(percent(100), row.width);
    assert_eq!(FlexDirection::Row, row.flex_direction);
    assert_eq!(1.0, row.flex_grow);
  }

  #[test]
  fn stockpile_totals_split_resources_from_recipe_materials() {
    let content = ContentDatabase::starter();
    let totals = BTreeMap::from([
      ("iron_ore".to_string(), 12),
      ("coal".to_string(), 7),
      ("iron_bars".to_string(), 5),
      ("storage_warehouse".to_string(), 1),
    ]);

    let (resources, materials) = split_stockpile_totals(&content, totals);

    assert_eq!(
      BTreeMap::from([("coal".to_string(), 7), ("iron_ore".to_string(), 12)]),
      resources
    );
    assert_eq!(
      BTreeMap::from([
        ("iron_bars".to_string(), 5),
        ("storage_warehouse".to_string(), 1),
      ]),
      materials
    );
  }

  #[test]
  fn presentation_state_tracks_authoritative_material_flow() {
    let mut host = SimHost::for_scenario(BUILDING_MATERIALS_SCENARIO);
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
      saw_collecting |= host.snapshot.haulers.iter().any(|hauler| {
        matches!(
          &hauler.dispatch,
          DispatchReceiverState::Assigned(assignment)
            if matches!(assignment.phase, DispatchPhase::Collect | DispatchPhase::Retrieve)
        )
      });
      saw_delivering |= host.snapshot.haulers.iter().any(|hauler| {
        matches!(
          &hauler.dispatch,
          DispatchReceiverState::Assigned(assignment)
            if matches!(assignment.phase, DispatchPhase::Deliver | DispatchPhase::Deploy)
        ) || !hauler.cargo.items.is_empty()
      });
      saw_crafting |= node_activity(&host.snapshot, NodeId::Factory(0)) == NodeActivity::Crafting;
    }

    assert!(saw_ready_source);
    assert!(saw_collecting);
    assert!(saw_delivering);
    assert!(saw_crafting);
  }

  #[test]
  fn route_direction_tracks_collect_and_delivery_phases() {
    let mut host = SimHost::for_scenario(IRON_BARS_SCENARIO);
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
    let mut host = SimHost::for_scenario(DEPLOYMENT_DEMO_SCENARIO);
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
  fn v2_remote_coal_plant_projection_exposes_generator_and_occupied_source() {
    let mut content = ContentDatabase::starter();
    let recipe_inputs = content.item(COAL_PLANT).ingredients.clone();
    let scenario = content
      .scenarios
      .get_mut(&V2_WORLD_SCENARIO)
      .expect("v2 scenario exists");
    for factory in &mut scenario.factories {
      factory.input_buffer = 0;
    }
    scenario.factories[10].starting_items = recipe_inputs;
    let mut game = GameState::new(content, V2_WORLD_SCENARIO).unwrap();

    let deployed = (0..150)
      .find_map(|_| {
        let snapshot = game.step();
        snapshot
          .power
          .as_ref()
          .is_some_and(|power| power.generators.len() == 2)
          .then_some(snapshot)
      })
      .expect("remote coal plant deploys");
    let mut snapshot = game.step();
    let generator = NodeId::Generator(1);
    let source = snapshot
      .sources
      .iter()
      .find(|source| source.occupied_by == Some(generator))
      .expect("deployed generator occupies its coal source");

    assert_eq!(NodeActivity::Ready, node_activity(&snapshot, generator));
    assert!(node_label_value(&snapshot, generator).contains("type: coal_plant"));
    let line = snapshot
      .topology
      .generator_power_lines
      .iter()
      .find(|line| line.generator == generator)
      .expect("remote generator exposes its complete line path");
    assert_eq!(NodeId::Generator(0), line.target);
    assert!(line
      .cells
      .iter()
      .all(|cell| snapshot.topology.power_lines.contains(cell)));
    assert!(node_label_value(&snapshot, generator)
      .contains(&format!("link: generator-0 via {} cells", line.cells.len())));
    assert_eq!(NodeActivity::Ready, node_activity(&snapshot, source.node));
    assert!(node_label_value(&snapshot, source.node).contains("occupied by generator-1"));
    assert!(deployed
      .topology
      .generator_power_lines
      .iter()
      .all(|line| line.generator != generator));

    let primary_cells = line.cells.len();
    snapshot
      .topology
      .generator_power_lines
      .push(GeneratorPowerLine {
        generator,
        target: NodeId::Source(0),
        cells: vec![GridPosition { x: 1, y: 1 }],
      });
    assert!(node_label_value(&snapshot, generator)
      .contains(&format!("link: 2 links via {} cells", primary_cells + 1)));
  }

  #[test]
  fn power_line_projection_tracks_generated_grid_cells() {
    let mut host = SimHost::for_scenario(POWER_LINE_SCENARIO);

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
    let mut host = SimHost::for_scenario(IRON_BARS_SCENARIO);
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
  fn accepted_art_maps_only_to_matching_simulation_identities() {
    let starter = scenario_game(IRON_BARS_SCENARIO).snapshot(Vec::new());
    let source = starter
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Source(0))
      .unwrap();
    let road = starter
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Road)
      .unwrap();
    let factory = starter
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Factory(0))
      .unwrap();

    assert_eq!(
      Some(NodeArtKind::Deposit(IRON_ORE)),
      node_art_kind(&starter, source)
    );
    assert_eq!(
      Some(NodeArtKind::Road(RoadOrientation::EastWest)),
      node_art_kind(&starter, road)
    );
    assert_eq!(Some(NodeArtKind::Foundry), node_art_kind(&starter, factory));

    let v2 = scenario_game(V2_WORLD_SCENARIO).snapshot(Vec::new());
    for item in [IRON_ORE, COPPER_ORE, COAL, STONE] {
      let source = v2
        .sources
        .iter()
        .find(|source| source.item == item)
        .unwrap();
      let node = v2
        .topology
        .nodes
        .iter()
        .find(|node| node.id == source.node)
        .unwrap();
      assert_eq!(Some(NodeArtKind::Deposit(item)), node_art_kind(&v2, node));
    }
    let junction = v2
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Road)
      .unwrap();
    assert_eq!(None, node_art_kind(&v2, junction));

    let generic_factory = v2
      .factories
      .iter()
      .find(|factory| factory.craft.output_item != IRON_BARS)
      .unwrap();
    let generic_factory_node = v2
      .topology
      .nodes
      .iter()
      .find(|node| node.id == generic_factory.node)
      .unwrap();
    assert_eq!(
      Some(NodeArtKind::Factory),
      node_art_kind(&v2, generic_factory_node)
    );
    let coal_plant = v2
      .power
      .as_ref()
      .unwrap()
      .generators
      .iter()
      .find(|generator| generator.item == Some(COAL_PLANT))
      .unwrap();
    let coal_plant_node = v2
      .topology
      .nodes
      .iter()
      .find(|node| node.id == coal_plant.node)
      .unwrap();
    assert_eq!(
      Some(NodeArtKind::CoalPlant),
      node_art_kind(&v2, coal_plant_node)
    );
    let radar_node = v2
      .topology
      .nodes
      .iter()
      .find(|node| matches!(node.id, NodeId::Radar(_)))
      .unwrap();
    assert_eq!(Some(NodeArtKind::Radar), node_art_kind(&v2, radar_node));

    let materials = scenario_game(BUILDING_MATERIALS_SCENARIO).snapshot(Vec::new());
    let stone = materials
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Source(1))
      .unwrap();
    assert_eq!(STONE, materials.sources[1].item);
    assert_eq!(
      Some(NodeArtKind::Deposit(STONE)),
      node_art_kind(&materials, stone)
    );

    let mut construction = scenario_game(BUILDING_DEPLOYMENT_SCENARIO);
    let initial_construction = construction.snapshot(Vec::new());
    let build_site = initial_construction
      .topology
      .nodes
      .iter()
      .find(|node| matches!(node.id, NodeId::BuildSite(_)))
      .unwrap();
    assert_eq!(None, node_art_kind(&initial_construction, build_site));
    let built = (0..16)
      .find_map(|_| {
        let snapshot = construction.step();
        (!snapshot.structures.is_empty()).then_some(snapshot)
      })
      .expect("warehouse construction completes");
    let warehouse = built
      .topology
      .nodes
      .iter()
      .find(|node| matches!(node.id, NodeId::Structure(_)))
      .unwrap();
    assert_eq!(
      Some(NodeArtKind::Warehouse),
      node_art_kind(&built, warehouse)
    );

    let mut deployment = scenario_game(DEPLOYMENT_DEMO_SCENARIO);
    let initial = deployment.snapshot(Vec::new());
    let initial_node = initial
      .topology
      .nodes
      .iter()
      .find(|node| node.id == initial.sources[0].node)
      .unwrap();
    assert!(drill_art_candidate(&initial, initial_node));
    assert!(!drill_art_visible(&initial, initial_node.id));
    let mut drill_sprite = Sprite::default();
    configure_drill_art(&mut drill_sprite, &initial, initial_node.id);
    assert_eq!(Some(Vec2::ZERO), drill_sprite.custom_size);
    let deployed = (0..64)
      .find_map(|_| {
        let snapshot = deployment.step();
        snapshot.sources[0].deployed.then_some(snapshot)
      })
      .expect("mining drill deploys");
    let deployed_node = deployed
      .topology
      .nodes
      .iter()
      .find(|node| node.id == deployed.sources[0].node)
      .unwrap();
    assert!(drill_art_visible(&deployed, deployed_node.id));
    configure_drill_art(&mut drill_sprite, &deployed, deployed_node.id);
    assert_eq!(Some(Vec2::splat(DRILL_ART_SIZE)), drill_sprite.custom_size);

    let mut occupied = deployed.clone();
    occupied.sources[0].occupied_by = Some(NodeId::Generator(0));
    configure_drill_art(&mut drill_sprite, &occupied, deployed_node.id);
    assert_eq!(Some(Vec2::ZERO), drill_sprite.custom_size);
  }

  #[test]
  fn node_backplates_are_limited_to_unmapped_identities() {
    let snapshot = scenario_game(V2_WORLD_SCENARIO).snapshot(Vec::new());
    let source = snapshot
      .topology
      .nodes
      .iter()
      .find(|node| matches!(node.id, NodeId::Source(_)))
      .unwrap();
    let junction = snapshot
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Road)
      .unwrap();

    assert!(matches!(
      node_presentation(&snapshot, source),
      NodePresentation::Art(NodeArtKind::Deposit(_))
    ));
    assert_eq!(
      NodePresentation::Fallback,
      node_presentation(&snapshot, junction)
    );
  }

  #[test]
  fn cargo_art_tracks_supported_authoritative_items() {
    let mut hauler = scenario_game(IRON_BARS_SCENARIO)
      .snapshot(Vec::new())
      .haulers[0]
      .clone();
    assert_eq!(None, cargo_art_item(&hauler));

    hauler.cargo.items.insert(IRON_ORE.as_str().into(), 2);
    assert_eq!(Some(IRON_ORE), cargo_art_item(&hauler));

    hauler.cargo.items.clear();
    hauler.cargo.items.insert(IRON_BARS.as_str().into(), 1);
    assert_eq!(Some(IRON_BARS), cargo_art_item(&hauler));

    hauler.cargo.items.clear();
    hauler.cargo.items.insert(STONE.as_str().into(), 3);
    assert_eq!(None, cargo_art_item(&hauler));
  }

  #[test]
  fn runtime_art_paths_are_stable_and_relative_to_the_asset_root() {
    assert_eq!("factory/terrain/ground.png", GROUND_ART);
    assert_eq!("factory/logistics/road-straight-ns.png", ROAD_ART);
    assert_eq!("factory/vehicles/truck.png", TRUCK_ART);
    assert_eq!("factory/resources/iron-ore-deposit.png", IRON_DEPOSIT_ART);
    assert_eq!(
      "factory/resources/copper-ore-deposit.png",
      COPPER_DEPOSIT_ART
    );
    assert_eq!("factory/resources/coal-deposit.png", COAL_DEPOSIT_ART);
    assert_eq!("factory/resources/stone-deposit.png", STONE_DEPOSIT_ART);
    assert_eq!("factory/machines/foundry.png", FOUNDRY_ART);
    assert_eq!("factory/machines/factory.png", FACTORY_ART);
    assert_eq!("factory/machines/coal-plant.png", COAL_PLANT_ART);
    assert_eq!("factory/machines/radar.png", RADAR_ART);
    assert_eq!("factory/machines/mining-drill.png", MINING_DRILL_ART);
    assert_eq!("factory/structures/warehouse.png", WAREHOUSE_ART);
    assert_eq!("factory/items/iron-ore.png", IRON_ORE_ART);
    assert_eq!("factory/items/iron-bars.png", IRON_BARS_ART);
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
  fn held_navigation_repeats_after_delay_independent_of_frame_slicing() {
    let direction = IVec2::new(1, -1);
    let mut one_frame = InputRepeatState::default();
    assert_eq!(1, repeated_input_steps(&mut one_frame, direction, 0.0));
    let one_frame_repeats = repeated_input_steps(&mut one_frame, direction, 0.4);

    let mut split_frames = InputRepeatState::default();
    assert_eq!(1, repeated_input_steps(&mut split_frames, direction, 0.0));
    let split_repeats = (0..5)
      .map(|_| repeated_input_steps(&mut split_frames, direction, 0.08))
      .sum::<u8>();

    assert_eq!(2, one_frame_repeats);
    assert_eq!(one_frame_repeats, split_repeats);
    assert_eq!(
      0,
      repeated_input_steps(&mut split_frames, IVec2::ZERO, 0.08)
    );
    assert_eq!(1, repeated_input_steps(&mut split_frames, -direction, 0.0));
  }

  #[test]
  fn held_zoom_repeats_resets_and_remains_bounded() {
    let mut repeat = InputRepeatState::default();

    assert_eq!(1, repeated_input_steps(&mut repeat, -1_i8, 0.0));
    assert_eq!(0, repeated_input_steps(&mut repeat, -1, 0.24));
    assert_eq!(1, repeated_input_steps(&mut repeat, -1, 0.01));
    assert_eq!(0, repeated_input_steps(&mut repeat, 0, 0.08));
    assert_eq!(1, repeated_input_steps(&mut repeat, 1, 0.0));
    assert_eq!(MIN_ZOOM_LEVEL, move_zoom_level(MIN_ZOOM_LEVEL, -1, 4));
    assert_eq!(MAX_ZOOM_LEVEL, move_zoom_level(MAX_ZOOM_LEVEL, 1, 4));
  }

  #[test]
  fn player_zoom_fits_the_world_and_keeps_a_ten_cell_detail_view() {
    let detail = player_zoom_scale(MIN_ZOOM_LEVEL, 50, 50, 1180.0, 720.0);
    let overview = player_zoom_scale(MAX_ZOOM_LEVEL, 50, 50, 1180.0, 720.0);

    assert!((detail - 5.0 / 3.0).abs() < f32::EPSILON);
    assert!((overview - 25.0 / 3.0).abs() < f32::EPSILON);
    assert_eq!(detail, player_zoom_scale(0, 50, 50, 1180.0, 720.0));
    assert_eq!(overview, player_zoom_scale(11, 50, 50, 1180.0, 720.0));
    assert_eq!(Vec2::new(4_000.0, 2_940.0), world_center(50, 50));
    assert_eq!(Vec2::new(9_000.0, 6_000.0), world_art_size(50, 50));
  }

  #[test]
  fn world_detail_is_limited_to_close_zoom() {
    assert!(world_detail_visible(true, MIN_ZOOM_LEVEL));
    assert!(world_detail_visible(true, MAX_DETAIL_ZOOM_LEVEL));
    assert!(!world_detail_visible(true, MAX_DETAIL_ZOOM_LEVEL + 1));
    assert!(!world_detail_visible(false, MIN_ZOOM_LEVEL));
  }

  #[test]
  fn world_focus_starts_at_the_map_center() {
    let game = scenario_game(V2_WORLD_SCENARIO);
    let snapshot = game.snapshot(Vec::new());

    assert_eq!(
      GridPosition { x: 25, y: 25 },
      initial_player_position(&snapshot)
    );
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
  fn viewer_world_roster_excludes_fixture_scenarios_and_wraps() {
    let mut host = SimHost::new();

    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
    assert_eq!([V2_WORLD_SCENARIO], WORLD_SCENARIOS);
    host.select_world(1);
    assert_eq!(V2_WORLD_SCENARIO, host.snapshot.scenario.id);
    assert_eq!(1, host.scene_revision);
    assert_eq!(0, host.snapshot.tick);
  }
}
