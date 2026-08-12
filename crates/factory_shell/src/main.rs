mod a11y;
mod storage;

use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;
use factory_content::{ItemId, COPPER_BARS, COPPER_ORE, IRON_BARS, IRON_ORE};
use factory_sim::{
  CompactGame, CompactRecipe, CompactSnapshot, GridPosition, COMPACT_SCENARIO_NAME,
  COMPACT_WORLD_HEIGHT, COMPACT_WORLD_WIDTH,
};

const NORMAL_TICKS_PER_SECOND: f32 = 2.0;
const FAST_TICKS_PER_SECOND: f32 = 8.0;
const MAX_TICKS_PER_FRAME: u8 = 8;
const CELL_SIZE: f32 = 100.0;
const GROUND_SIZE: f32 = 96.0;
const TOP_BAR_HEIGHT: f32 = 132.0;
const PANEL_WIDTH: f32 = 380.0;
const PANEL_HEIGHT: f32 = 326.0;
const MIN_VISIBLE_CELLS: f32 = 10.0;
const MIN_ZOOM_LEVEL: u8 = 1;
const MAX_ZOOM_LEVEL: u8 = 10;
const INLINE_SEPARATOR: &str = " // ";
const AUTOSAVE_SECONDS: f32 = 5.0;

const GROUND_ART: &str = "factory/terrain/ground.png";
const ROAD_ART: &str = "factory/logistics/road-straight-ns.png";
const TRUCK_ART: &str = "factory/vehicles/truck.png";
const IRON_DEPOSIT_ART: &str = "factory/resources/iron-ore-deposit.png";
const COPPER_DEPOSIT_ART: &str = "factory/resources/copper-ore-deposit.png";
const FOUNDRY_ART: &str = "factory/machines/foundry.png";
const FACTORY_ART: &str = "factory/machines/factory.png";
const WAREHOUSE_ART: &str = "factory/structures/warehouse.png";
const IRON_ORE_ART: &str = "factory/items/iron-ore.png";
const IRON_BARS_ART: &str = "factory/items/iron-bars.png";

const PANEL_BACKGROUND: Color = Color::srgba(0.035, 0.045, 0.065, 0.97);
const TOP_BACKGROUND: Color = Color::srgba(0.025, 0.032, 0.047, 0.98);
const BUTTON_NORMAL: Color = Color::srgb(0.14, 0.17, 0.22);
const BUTTON_HOVERED: Color = Color::srgb(0.24, 0.29, 0.36);
const BUTTON_ACTIVE: Color = Color::srgb(0.30, 0.66, 0.47);
const BUTTON_PRESSED: Color = Color::srgb(0.85, 0.52, 0.25);
const BUTTON_BORDER: Color = Color::srgb(0.34, 0.39, 0.48);
const TEXT_PRIMARY: Color = Color::srgb(0.92, 0.94, 0.97);
const TEXT_MUTED: Color = Color::srgb(0.56, 0.61, 0.69);
const ACCENT_ORANGE: Color = Color::srgb(0.96, 0.58, 0.28);
const ACCENT_GOLD: Color = Color::srgb(0.95, 0.78, 0.36);
const ACCENT_GREEN: Color = Color::srgb(0.48, 0.88, 0.62);

fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "factory game".into(),
            resolution: (1180, 720).into(),
            fit_canvas_to_parent: true,
            // The page owns the layout so the accessible panel can sit beside
            // the canvas instead of under it. See docs/accessible-play.md.
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#fg-canvas".into()),
            ..default()
          }),
          ..default()
        })
        // The shell ships no .meta sidecars. See docs/factory-art.md for why
        // the probe is skipped rather than answered.
        .set(AssetPlugin {
          meta_check: AssetMetaCheck::Never,
          ..default()
        }),
    )
    .insert_resource(ClearColor(Color::srgb(0.055, 0.063, 0.075)))
    // The shell opens paused, so it opens in the reactive mode too.
    .insert_resource(WinitSettings::desktop_app())
    .insert_resource(SimHost::new())
    .init_resource::<PlayerView>()
    .init_resource::<HoverCell>()
    .init_resource::<FactoryArt>()
    .init_resource::<A11yFocus>()
    .init_resource::<A11yLog>()
    .add_systems(Startup, (setup, install_accessible_surface))
    .add_systems(
      Update,
      (
        handle_keyboard,
        handle_control_buttons,
        update_camera,
        handle_pointer_edits,
        apply_accessible_commands,
        sync_frame_pacing,
        advance_simulation,
        autosave,
        rebuild_dynamic_projection,
        animate_trucks,
        update_ui_text,
        update_hover_cursor,
        style_control_buttons,
        publish_accessible_surface,
      )
        .chain(),
    )
    .run();
}

#[derive(Resource)]
struct FactoryArt {
  ground: Handle<Image>,
  road: Handle<Image>,
  truck: Handle<Image>,
  iron_deposit: Handle<Image>,
  copper_deposit: Handle<Image>,
  foundry: Handle<Image>,
  factory: Handle<Image>,
  warehouse: Handle<Image>,
  iron_ore: Handle<Image>,
  iron_bars: Handle<Image>,
}

impl FromWorld for FactoryArt {
  fn from_world(world: &mut World) -> Self {
    let assets = world.resource::<AssetServer>();
    Self {
      ground: assets.load(GROUND_ART),
      road: assets.load(ROAD_ART),
      truck: assets.load(TRUCK_ART),
      iron_deposit: assets.load(IRON_DEPOSIT_ART),
      copper_deposit: assets.load(COPPER_DEPOSIT_ART),
      foundry: assets.load(FOUNDRY_ART),
      factory: assets.load(FACTORY_ART),
      warehouse: assets.load(WAREHOUSE_ART),
      iron_ore: assets.load(IRON_ORE_ART),
      iron_bars: assets.load(IRON_BARS_ART),
    }
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum ToolMode {
  #[default]
  Inspect,
  Road,
  Erase,
  Building,
}

impl ToolMode {
  const fn label(self) -> &'static str {
    match self {
      Self::Inspect => "INSPECT",
      Self::Road => "DRAW ROAD",
      Self::Erase => "ERASE ROAD",
      Self::Building => "PLACE FACTORY",
    }
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ControlAction {
  SetTool(ToolMode),
  Configure(CompactRecipe),
  TogglePause,
  Step,
  Reset,
  ToggleSpeed,
  ZoomIn,
  ZoomOut,
}

#[derive(Resource)]
struct SimHost {
  game: CompactGame,
  snapshot: CompactSnapshot,
  paused: bool,
  ticks_per_second: f32,
  accumulated_seconds: f32,
  tool: ToolMode,
  selected_building: Option<u16>,
  feedback: String,
  snapshot_revision: u64,
  seconds_since_save: f32,
  saved_revision: u64,
}

impl SimHost {
  fn new() -> Self {
    let (mut game, feedback) = Self::restore_or_start();
    let snapshot = game.snapshot();
    Self {
      game,
      snapshot,
      paused: true,
      ticks_per_second: NORMAL_TICKS_PER_SECOND,
      accumulated_seconds: 0.0,
      tool: ToolMode::Inspect,
      selected_building: None,
      feedback,
      snapshot_revision: 1,
      seconds_since_save: 0.0,
      saved_revision: 0,
    }
  }

  fn restore_or_start() -> (CompactGame, String) {
    // An unreadable save never blocks play, and its slot is left intact.
    // See docs/compact-persistence.md.
    match storage::load().map(|raw| CompactGame::from_save_string(&raw)) {
      Some(Ok(game)) => (game, "Restored your last session.".into()),
      Some(Err(error)) => (
        CompactGame::new(),
        format!("Could not read the saved session ({error}). Started fresh."),
      ),
      None => (
        CompactGame::new(),
        "Draw a road from the warehouse apron toward a deposit.".into(),
      ),
    }
  }

  fn save_now(&self) {
    match self.game.to_save_string() {
      Ok(raw) => {
        if let Err(error) = storage::store(&raw) {
          warn!("could not persist the session: {error}");
        }
      }
      Err(error) => warn!("could not serialize the session: {error}"),
    }
  }

  fn refresh(&mut self) {
    self.snapshot = self.game.snapshot();
    if let Some(event) = self.snapshot.events.last() {
      self.feedback = event.clone();
    }
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
  }

  fn step_once(&mut self) {
    self.snapshot = self.game.step();
    if let Some(event) = self.snapshot.events.last() {
      self.feedback = event.clone();
    }
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
  }

  fn reset(&mut self) {
    storage::clear();
    self.game = CompactGame::new();
    self.snapshot = self.game.snapshot();
    self.paused = true;
    self.accumulated_seconds = 0.0;
    self.tool = ToolMode::Inspect;
    self.selected_building = None;
    self.feedback = "World reset. Draw a road from the warehouse apron.".into();
    self.snapshot_revision = self.snapshot_revision.wrapping_add(1);
    self.seconds_since_save = 0.0;
    self.saved_revision = self.snapshot_revision;
  }

  fn edit_cell(&mut self, cell: GridPosition) {
    match self.tool {
      ToolMode::Road => match self.game.place_road(cell) {
        Ok(true) => self.refresh(),
        Ok(false) => self.feedback = "Road already exists here.".into(),
        Err(error) => self.feedback = error.to_string(),
      },
      ToolMode::Erase => match self.game.remove_road(cell) {
        Ok(true) => self.refresh(),
        Ok(false) => self.feedback = "There is no road here.".into(),
        Err(error) => self.feedback = error.to_string(),
      },
      ToolMode::Building => match self.game.place_building(cell) {
        Ok(building) => {
          self.selected_building = Some(building);
          self.tool = ToolMode::Inspect;
          self.refresh();
          self.feedback = format!("Factory {building} placed. Choose its recipe.");
        }
        Err(error) => self.feedback = error.to_string(),
      },
      ToolMode::Inspect => {
        self.selected_building = self
          .snapshot
          .buildings
          .iter()
          .find(|building| building.position == cell)
          .map(|building| building.id);
        self.feedback = inspect_cell(&self.snapshot, cell);
      }
    }
  }

  fn configure_selected(&mut self, recipe: CompactRecipe) {
    let Some(building) = self.selected_building else {
      self.feedback = "Select a factory before assigning a recipe.".into();
      return;
    };
    match self.game.configure_building(building, recipe) {
      Ok(()) => {
        self.refresh();
        self.feedback = format!("Factory {building} now makes {}.", recipe.name());
      }
      Err(error) => self.feedback = error.to_string(),
    }
  }
}

#[derive(Resource)]
struct PlayerView {
  center: Vec2,
  zoom_level: u8,
}

impl Default for PlayerView {
  fn default() -> Self {
    Self {
      center: Vec2::ZERO,
      zoom_level: MAX_ZOOM_LEVEL,
    }
  }
}

#[derive(Resource, Default)]
struct HoverCell(Option<GridPosition>);

/// The cell the accessible surface currently describes. A player who cannot
/// see the grid needs a cursor that does not depend on a mouse.
#[derive(Resource)]
struct A11yFocus(GridPosition);

impl Default for A11yFocus {
  fn default() -> Self {
    Self(CompactGame::WAREHOUSE_POSITION)
  }
}

/// A rolling window of recent events, keyed on the snapshot revision so a
/// quiet tick does not blank the region a screen reader just read.
#[derive(Resource, Default)]
struct A11yLog {
  entries: Vec<String>,
  revision: u64,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct DynamicProjection;

#[derive(Component)]
struct TruckVisual {
  id: u16,
  target: Vec2,
}

#[derive(Component)]
struct HoverCursor;

#[derive(Component)]
struct ControlButton(ControlAction);

#[derive(Component)]
struct StatusCount(ItemId);

#[derive(Component)]
struct MarketStatusText;

#[derive(Component)]
struct PermitText;

#[derive(Component)]
struct SelectionText;

#[derive(Component)]
struct FeedbackText;

fn setup(mut commands: Commands, art: Res<FactoryArt>, host: Res<SimHost>) {
  commands.spawn((Camera2d, MainCamera));
  spawn_ground(&mut commands, &art);
  spawn_trucks(&mut commands, &host, &art);
  commands.spawn((
    Sprite::from_color(
      Color::srgba(0.35, 0.85, 0.62, 0.24),
      Vec2::splat(GROUND_SIZE),
    ),
    Transform::from_xyz(0.0, 0.0, 8.0),
    Visibility::Hidden,
    HoverCursor,
  ));
  spawn_top_bar(&mut commands, &art);
  spawn_control_panel(&mut commands);
}

fn spawn_ground(commands: &mut Commands, art: &FactoryArt) {
  for y in 0..COMPACT_WORLD_HEIGHT {
    for x in 0..COMPACT_WORLD_WIDTH {
      let cell = GridPosition { x, y };
      let position = grid_to_world(cell);
      commands.spawn((
        Sprite {
          image: art.ground.clone(),
          custom_size: Some(Vec2::splat(GROUND_SIZE)),
          ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0),
      ));
    }
  }
}

fn spawn_trucks(commands: &mut Commands, host: &SimHost, art: &FactoryArt) {
  for truck in &host.snapshot.trucks {
    let position = grid_to_world(truck.position);
    commands.spawn((
      Sprite {
        image: art.truck.clone(),
        custom_size: Some(Vec2::splat(66.0)),
        ..default()
      },
      Transform::from_xyz(position.x, position.y, 4.0),
      TruckVisual {
        id: truck.id,
        target: position,
      },
    ));
  }
}

fn spawn_top_bar(commands: &mut Commands, art: &FactoryArt) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        top: px(0),
        left: px(0),
        right: px(0),
        height: px(TOP_BAR_HEIGHT),
        padding: UiRect::axes(px(18), px(10)),
        border: UiRect {
          bottom: px(1),
          ..default()
        },
        flex_direction: FlexDirection::Row,
        column_gap: px(18),
        ..default()
      },
      BackgroundColor(TOP_BACKGROUND),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(100),
    ))
    .with_children(|bar| {
      bar
        .spawn(Node {
          width: px(250),
          flex_shrink: 0.0,
          flex_direction: FlexDirection::Column,
          justify_content: JustifyContent::Center,
          row_gap: px(4),
          ..default()
        })
        .with_children(|title| {
          title.spawn((
            Text::new("FACTORY GAME"),
            TextFont {
              font_size: FontSize::Px(24.0),
              ..default()
            },
            TextColor(TEXT_PRIMARY),
          ));
          title.spawn((
            Text::new(COMPACT_SCENARIO_NAME.to_uppercase()),
            TextFont {
              font_size: FontSize::Px(15.0),
              ..default()
            },
            TextColor(ACCENT_GREEN),
          ));
        });
      bar
        .spawn(Node {
          flex_grow: 1.0,
          min_width: px(0),
          flex_direction: FlexDirection::Column,
          justify_content: JustifyContent::Center,
          border: UiRect {
            left: px(1),
            ..default()
          },
          ..default()
        })
        .with_children(|rows| {
          spawn_stock_row(
            rows,
            "RESOURCES",
            ACCENT_ORANGE,
            &[
              (IRON_ORE, Some(art.iron_ore.clone())),
              (COPPER_ORE, Some(art.copper_deposit.clone())),
            ],
            false,
          );
          spawn_stock_row(
            rows,
            "MATERIALS",
            ACCENT_GOLD,
            &[
              (IRON_BARS, Some(art.iron_bars.clone())),
              (COPPER_BARS, None),
            ],
            true,
          );
          rows
            .spawn((status_row_node(true), BorderColor::all(BUTTON_BORDER)))
            .with_children(|row| {
              spawn_status_label(row, "MARKET", ACCENT_GREEN);
              row.spawn((
                Text::new("DEMAND 4 // SOLD 0 // $0"),
                status_value_font(),
                TextColor(TEXT_PRIMARY),
                MarketStatusText,
              ));
            });
        });
    });
}

fn spawn_stock_row(
  parent: &mut ChildSpawnerCommands,
  label: &'static str,
  accent: Color,
  items: &[(ItemId, Option<Handle<Image>>)],
  divided: bool,
) {
  parent
    .spawn((status_row_node(divided), BorderColor::all(BUTTON_BORDER)))
    .with_children(|row| {
      spawn_status_label(row, label, accent);
      for (index, (item, image)) in items.iter().enumerate() {
        if index > 0 {
          row.spawn((
            Text::new(INLINE_SEPARATOR.trim()),
            TextFont {
              font_size: FontSize::Px(14.0),
              ..default()
            },
            TextColor(TEXT_MUTED),
          ));
        }
        if let Some(image) = image {
          row.spawn((
            ImageNode::new(image.clone()),
            Node {
              width: px(19),
              height: px(19),
              flex_shrink: 0.0,
              ..default()
            },
          ));
        } else {
          row.spawn((
            Text::new(item_name(*item).to_uppercase()),
            TextFont {
              font_size: FontSize::Px(13.0),
              ..default()
            },
            TextColor(TEXT_MUTED),
          ));
        }
        row.spawn((
          Text::new("0"),
          status_value_font(),
          TextColor(TEXT_PRIMARY),
          StatusCount(*item),
        ));
      }
    });
}

fn status_row_node(divided: bool) -> Node {
  Node {
    width: percent(100),
    min_height: px(0),
    flex_grow: 1.0,
    flex_direction: FlexDirection::Row,
    align_items: AlignItems::Center,
    column_gap: px(7),
    padding: UiRect::axes(px(14), px(0)),
    border: UiRect {
      top: px(if divided { 1 } else { 0 }),
      ..default()
    },
    overflow: Overflow::clip_x(),
    ..default()
  }
}

fn spawn_status_label(parent: &mut ChildSpawnerCommands, label: &'static str, color: Color) {
  parent.spawn((
    Text::new(label),
    TextFont {
      font_size: FontSize::Px(15.0),
      ..default()
    },
    TextColor(color),
    Node {
      width: px(104),
      flex_shrink: 0.0,
      ..default()
    },
  ));
}

fn status_value_font() -> TextFont {
  TextFont {
    font_size: FontSize::Px(16.0),
    ..default()
  }
}

fn spawn_control_panel(commands: &mut Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        right: px(18),
        bottom: px(18),
        width: px(PANEL_WIDTH),
        height: px(PANEL_HEIGHT),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        padding: UiRect::all(px(12)),
        border: UiRect::all(px(1)),
        ..default()
      },
      BackgroundColor(PANEL_BACKGROUND),
      BorderColor::all(BUTTON_BORDER),
      GlobalZIndex(100),
    ))
    .with_children(|panel| {
      panel.spawn((
        Text::new("NETWORK PLANNER"),
        TextFont {
          font_size: FontSize::Px(13.0),
          ..default()
        },
        TextColor(ACCENT_GREEN),
      ));
      panel.spawn((
        Text::new("BUILDINGS 0 / 2 // NEXT AT 20 SALES"),
        TextFont {
          font_size: FontSize::Px(14.0),
          ..default()
        },
        TextColor(TEXT_PRIMARY),
        PermitText,
      ));
      spawn_button_row(
        panel,
        &[
          (ControlAction::SetTool(ToolMode::Inspect), "INSPECT"),
          (ControlAction::SetTool(ToolMode::Road), "ROAD"),
          (ControlAction::SetTool(ToolMode::Erase), "ERASE"),
          (ControlAction::SetTool(ToolMode::Building), "FACTORY"),
        ],
      );
      panel.spawn((
        Text::new("SELECTED // NONE"),
        TextFont {
          font_size: FontSize::Px(13.0),
          ..default()
        },
        TextColor(TEXT_MUTED),
        SelectionText,
      ));
      spawn_button_row(
        panel,
        &[
          (
            ControlAction::Configure(CompactRecipe::IronBars),
            "IRON BARS",
          ),
          (
            ControlAction::Configure(CompactRecipe::CopperBars),
            "COPPER BARS",
          ),
        ],
      );
      spawn_button_row(
        panel,
        &[
          (ControlAction::TogglePause, "PLAY"),
          (ControlAction::Step, "STEP"),
          (ControlAction::ToggleSpeed, "SPEED"),
          (ControlAction::Reset, "RESET"),
        ],
      );
      spawn_button_row(
        panel,
        &[
          (ControlAction::ZoomIn, "ZOOM +"),
          (ControlAction::ZoomOut, "ZOOM -"),
        ],
      );
      panel.spawn((
        Text::new("Draw a road from the warehouse apron toward a deposit."),
        TextFont {
          font_size: FontSize::Px(13.0),
          ..default()
        },
        TextColor(ACCENT_GOLD),
        Node {
          min_height: px(34),
          ..default()
        },
        FeedbackText,
      ));
      panel.spawn((
        Text::new("1 INSPECT // 2 ROAD // 3 ERASE // 4 FACTORY // WASD PAN // WHEEL ZOOM"),
        TextFont {
          font_size: FontSize::Px(10.0),
          ..default()
        },
        TextColor(TEXT_MUTED),
      ));
    });
}

fn spawn_button_row(parent: &mut ChildSpawnerCommands, buttons: &[(ControlAction, &'static str)]) {
  parent
    .spawn(Node {
      width: percent(100),
      height: px(32),
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
            min_width: px(0),
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
            TextColor(TEXT_PRIMARY),
          )],
        ));
      }
    });
}

fn handle_keyboard(
  keys: Res<ButtonInput<KeyCode>>,
  mut host: ResMut<SimHost>,
  mut view: ResMut<PlayerView>,
) {
  for (key, action) in [
    (KeyCode::Space, ControlAction::TogglePause),
    (KeyCode::KeyN, ControlAction::Step),
    (KeyCode::KeyR, ControlAction::Reset),
    (KeyCode::KeyF, ControlAction::ToggleSpeed),
    (KeyCode::Digit1, ControlAction::SetTool(ToolMode::Inspect)),
    (KeyCode::Digit2, ControlAction::SetTool(ToolMode::Road)),
    (KeyCode::Digit3, ControlAction::SetTool(ToolMode::Erase)),
    (KeyCode::Digit4, ControlAction::SetTool(ToolMode::Building)),
    (
      KeyCode::KeyI,
      ControlAction::Configure(CompactRecipe::IronBars),
    ),
    (
      KeyCode::KeyC,
      ControlAction::Configure(CompactRecipe::CopperBars),
    ),
    (KeyCode::KeyQ, ControlAction::ZoomIn),
    (KeyCode::KeyE, ControlAction::ZoomOut),
  ] {
    if keys.just_pressed(key) {
      apply_control(action, &mut host, &mut view);
    }
  }
  if keys.just_pressed(KeyCode::Escape) {
    host.tool = ToolMode::Inspect;
    host.selected_building = None;
    host.feedback = "Inspect mode.".into();
  }
}

fn handle_control_buttons(
  buttons: Query<(&Interaction, &ControlButton), (Changed<Interaction>, With<Button>)>,
  mut host: ResMut<SimHost>,
  mut view: ResMut<PlayerView>,
) {
  for (interaction, button) in &buttons {
    if *interaction == Interaction::Pressed {
      apply_control(button.0, &mut host, &mut view);
    }
  }
}

fn apply_control(action: ControlAction, host: &mut SimHost, view: &mut PlayerView) {
  match action {
    ControlAction::SetTool(tool) => {
      host.tool = tool;
      host.feedback = format!("{} mode.", tool.label());
    }
    ControlAction::Configure(recipe) => host.configure_selected(recipe),
    ControlAction::TogglePause => {
      host.paused = !host.paused;
      host.accumulated_seconds = 0.0;
      host.feedback = if host.paused {
        "Simulation paused. Planning edits remain available.".into()
      } else {
        "Simulation running. Trucks dispatch automatically.".into()
      };
    }
    ControlAction::Step => {
      host.paused = true;
      host.step_once();
    }
    ControlAction::Reset => {
      host.reset();
      *view = PlayerView::default();
    }
    ControlAction::ToggleSpeed => {
      host.ticks_per_second = if host.ticks_per_second == NORMAL_TICKS_PER_SECOND {
        FAST_TICKS_PER_SECOND
      } else {
        NORMAL_TICKS_PER_SECOND
      };
      host.feedback = format!("Simulation speed {}x.", host.ticks_per_second / 2.0);
    }
    ControlAction::ZoomIn => {
      view.zoom_level = view.zoom_level.saturating_sub(1).max(MIN_ZOOM_LEVEL);
    }
    ControlAction::ZoomOut => {
      view.zoom_level = view.zoom_level.saturating_add(1).min(MAX_ZOOM_LEVEL);
    }
  }
}

fn update_camera(
  keys: Res<ButtonInput<KeyCode>>,
  time: Res<Time>,
  mut wheel: MessageReader<MouseWheel>,
  window: Single<&Window>,
  mut view: ResMut<PlayerView>,
  mut camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
  let wheel_delta = wheel.read().map(|event| event.y).sum::<f32>();
  if wheel_delta > 0.0 {
    view.zoom_level = view.zoom_level.saturating_sub(1).max(MIN_ZOOM_LEVEL);
  } else if wheel_delta < 0.0 {
    view.zoom_level = view.zoom_level.saturating_add(1).min(MAX_ZOOM_LEVEL);
  }

  let direction = Vec2::new(
    f32::from(keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight))
      - f32::from(keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)),
    f32::from(keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp))
      - f32::from(keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)),
  );
  let scale = compact_zoom_scale(view.zoom_level, window.width(), window.height());
  if direction != Vec2::ZERO && view.zoom_level < MAX_ZOOM_LEVEL {
    view.center += direction.normalize() * 620.0 * scale * time.delta_secs();
  }

  let world_size = Vec2::new(
    COMPACT_WORLD_WIDTH as f32 * CELL_SIZE,
    COMPACT_WORLD_HEIGHT as f32 * CELL_SIZE,
  );
  let half_view = Vec2::new(
    window.width() * scale,
    (window.height() - TOP_BAR_HEIGHT) * scale,
  ) / 2.0;
  let limit = (world_size / 2.0 - half_view).max(Vec2::ZERO);
  view.center.x = view.center.x.clamp(-limit.x, limit.x);
  view.center.y = view.center.y.clamp(-limit.y, limit.y);
  if view.zoom_level == MAX_ZOOM_LEVEL {
    view.center = Vec2::ZERO;
  }

  camera.0.translation.x = view.center.x;
  camera.0.translation.y = view.center.y + TOP_BAR_HEIGHT * scale / 2.0;
  if let Projection::Orthographic(projection) = &mut *camera.1 {
    projection.scale = scale;
  }
}

fn handle_pointer_edits(
  mouse: Res<ButtonInput<MouseButton>>,
  touches: Res<Touches>,
  window: Single<&Window>,
  camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
  button_interactions: Query<&Interaction, With<Button>>,
  mut host: ResMut<SimHost>,
  mut hover: ResMut<HoverCell>,
  mut last_painted: Local<Option<GridPosition>>,
) {
  let touch = touches.iter().next();
  let screen_position = touch
    .map(|touch| touch.position())
    .or_else(|| window.cursor_position());
  let over_ui = screen_position
    .is_some_and(|position| pointer_over_ui(position, Vec2::new(window.width(), window.height())));
  let cell = (!over_ui)
    .then_some(screen_position)
    .flatten()
    .and_then(|position| {
      camera
        .0
        .viewport_to_world_2d(camera.1, position)
        .ok()
        .and_then(world_to_grid)
    });
  hover.0 = cell;

  let active = mouse.pressed(MouseButton::Left) || touch.is_some();
  if !active {
    *last_painted = None;
    return;
  }
  if over_ui {
    return;
  }
  if button_interactions
    .iter()
    .any(|interaction| *interaction != Interaction::None)
  {
    return;
  }
  let just_pressed =
    mouse.just_pressed(MouseButton::Left) || touches.iter_just_pressed().next().is_some();
  let paint_mode = matches!(host.tool, ToolMode::Road | ToolMode::Erase);
  if !paint_mode && !just_pressed {
    return;
  }
  let Some(cell) = cell else {
    return;
  };
  if *last_painted == Some(cell) {
    return;
  }
  host.edit_cell(cell);
  *last_painted = Some(cell);
}

const PAUSED_WAIT: Duration = Duration::from_secs(5);

/// A paused native shell idles instead of burning a core on an unchanged
/// frame. The browser is excluded: see docs/accessible-play.md.
fn pacing_for(paused: bool) -> WinitSettings {
  if !paused || cfg!(target_arch = "wasm32") {
    return WinitSettings::game();
  }
  WinitSettings {
    focused_mode: UpdateMode::reactive(PAUSED_WAIT),
    unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(60)),
  }
}

fn install_accessible_surface() {
  a11y::install();
}

/// The accessible surface never decides anything. It lands in the same host
/// paths the pointer and keyboard surfaces use.
fn apply_accessible_commands(
  mut host: ResMut<SimHost>,
  mut view: ResMut<PlayerView>,
  mut focus: ResMut<A11yFocus>,
) {
  for command in a11y::drain() {
    match command {
      a11y::Command::Control(action) => apply_control(action, &mut host, &mut view),
      a11y::Command::EditAt(tool, cell) => {
        host.tool = tool;
        host.edit_cell(cell);
        focus.0 = cell;
      }
      a11y::Command::SelectBuilding(id) => match host
        .snapshot
        .buildings
        .iter()
        .find(|building| building.id == id)
      {
        Some(building) => {
          focus.0 = building.position;
          host.selected_building = Some(id);
          host.feedback = format!("Factory {id} selected.");
        }
        None => host.feedback = format!("There is no factory {id}."),
      },
      a11y::Command::Focus(cell) => focus.0 = cell,
    }
  }
}

fn publish_accessible_surface(
  host: Res<SimHost>,
  focus: Res<A11yFocus>,
  mut log: ResMut<A11yLog>,
) {
  if log.revision != host.snapshot_revision {
    log.revision = host.snapshot_revision;
    let events = host.snapshot.events.clone();
    a11y::remember(&mut log.entries, &events);
  }
  a11y::publish(&a11y::Report {
    snapshot: &host.snapshot,
    paused: host.paused,
    speed: host.ticks_per_second,
    focus: focus.0,
    selected_building: host.selected_building,
    feedback: &host.feedback,
    events: &log.entries,
  });
}

fn sync_frame_pacing(
  host: Res<SimHost>,
  mut settings: ResMut<WinitSettings>,
  mut applied: Local<Option<bool>>,
) {
  if *applied == Some(host.paused) {
    return;
  }
  *applied = Some(host.paused);
  *settings = pacing_for(host.paused);
}

fn advance_simulation(time: Res<Time>, mut host: ResMut<SimHost>) {
  if host.paused {
    return;
  }
  host.accumulated_seconds += time.delta_secs();
  let interval = 1.0 / host.ticks_per_second;
  let mut ticks = 0;
  while host.accumulated_seconds >= interval && ticks < MAX_TICKS_PER_FRAME {
    host.accumulated_seconds -= interval;
    host.step_once();
    ticks += 1;
  }
}

// Autosave is time-based rather than per-tick: at eight ticks a second a
// per-tick write would serialize and store the world eight times a second.
fn autosave(time: Res<Time>, mut host: ResMut<SimHost>) {
  host.seconds_since_save += time.delta_secs();
  if host.seconds_since_save < AUTOSAVE_SECONDS {
    return;
  }
  host.seconds_since_save = 0.0;
  if host.saved_revision == host.snapshot_revision {
    return;
  }
  host.save_now();
  host.saved_revision = host.snapshot_revision;
}

fn rebuild_dynamic_projection(
  mut commands: Commands,
  host: Res<SimHost>,
  art: Res<FactoryArt>,
  mut last_revision: Local<u64>,
  entities: Query<Entity, With<DynamicProjection>>,
) {
  if *last_revision == host.snapshot_revision {
    return;
  }
  *last_revision = host.snapshot_revision;
  for entity in &entities {
    commands.entity(entity).despawn();
  }
  spawn_dynamic(&mut commands, &host, &art);
}

fn spawn_dynamic(commands: &mut Commands, host: &SimHost, art: &FactoryArt) {
  for road in &host.snapshot.roads {
    let position = grid_to_world(*road);
    let horizontal = host.snapshot.roads.contains(&GridPosition {
      x: road.x - 1,
      y: road.y,
    }) || host.snapshot.roads.contains(&GridPosition {
      x: road.x + 1,
      y: road.y,
    });
    commands.spawn((
      Sprite {
        image: art.road.clone(),
        custom_size: Some(Vec2::splat(88.0)),
        ..default()
      },
      Transform::from_xyz(position.x, position.y, 1.0).with_rotation(if horizontal {
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
      } else {
        Quat::IDENTITY
      }),
      DynamicProjection,
    ));
  }

  for deposit in &host.snapshot.deposits {
    let position = grid_to_world(deposit.position);
    let image = if deposit.item == IRON_ORE {
      art.iron_deposit.clone()
    } else {
      art.copper_deposit.clone()
    };
    commands.spawn((
      Sprite {
        image,
        custom_size: Some(Vec2::splat(92.0)),
        ..default()
      },
      Transform::from_xyz(position.x, position.y, 2.0),
      DynamicProjection,
    ));
    commands.spawn((
      Text2d::new(format!(
        "{}\nstock {} // left {}",
        item_name(deposit.item),
        deposit.stockpile,
        deposit.remaining
      )),
      TextFont {
        font_size: FontSize::Px(12.0),
        ..default()
      },
      TextColor(TEXT_PRIMARY),
      Transform::from_xyz(position.x, position.y - 52.0, 3.0),
      DynamicProjection,
    ));
  }

  let warehouse = grid_to_world(host.snapshot.warehouse_position);
  commands.spawn((
    Sprite {
      image: art.warehouse.clone(),
      custom_size: Some(Vec2::splat(98.0)),
      ..default()
    },
    Transform::from_xyz(warehouse.x, warehouse.y, 2.2),
    DynamicProjection,
  ));
  commands.spawn((
    Text2d::new("WAREHOUSE\nEXPORT HUB"),
    TextFont {
      font_size: FontSize::Px(13.0),
      ..default()
    },
    TextColor(ACCENT_GREEN),
    Transform::from_xyz(warehouse.x, warehouse.y - 54.0, 3.0),
    DynamicProjection,
  ));

  for building in &host.snapshot.buildings {
    let position = grid_to_world(building.position);
    let image = match building.recipe {
      Some(CompactRecipe::IronBars) => art.foundry.clone(),
      _ => art.factory.clone(),
    };
    commands.spawn((
      Sprite {
        image,
        custom_size: Some(Vec2::splat(94.0)),
        ..default()
      },
      Transform::from_xyz(position.x, position.y, 2.2),
      DynamicProjection,
    ));
    let recipe = building.recipe.map_or("CHOOSE RECIPE", CompactRecipe::name);
    commands.spawn((
      Text2d::new(format!(
        "F{} // {}\nin {} // out {}",
        building.id, recipe, building.input_stock, building.output_stock
      )),
      TextFont {
        font_size: FontSize::Px(12.0),
        ..default()
      },
      TextColor(if building.road_connected {
        TEXT_PRIMARY
      } else {
        BUTTON_PRESSED
      }),
      Transform::from_xyz(position.x, position.y - 52.0, 3.0),
      DynamicProjection,
    ));
    if host.selected_building == Some(building.id) {
      commands.spawn((
        Sprite::from_color(Color::srgba(0.48, 0.88, 0.62, 0.24), Vec2::splat(106.0)),
        Transform::from_xyz(position.x, position.y, 2.1),
        DynamicProjection,
      ));
    }
  }

  for truck in &host.snapshot.trucks {
    for route in &truck.route {
      let route_position = grid_to_world(*route);
      commands.spawn((
        Sprite::from_color(Color::srgba(0.95, 0.72, 0.24, 0.28), Vec2::splat(18.0)),
        Transform::from_xyz(route_position.x, route_position.y, 3.1),
        DynamicProjection,
      ));
    }
    let position = grid_to_world(truck.position);
    if let Some(item) = truck.cargo_item {
      commands.spawn((
        Text2d::new(format!("{} {}", item_name(item), truck.cargo_quantity)),
        TextFont {
          font_size: FontSize::Px(11.0),
          ..default()
        },
        TextColor(ACCENT_GOLD),
        Transform::from_xyz(position.x, position.y + 36.0, 4.5),
        DynamicProjection,
      ));
    }
  }
}

fn animate_trucks(
  time: Res<Time>,
  host: Res<SimHost>,
  mut trucks: Query<(&mut TruckVisual, &mut Transform)>,
) {
  let blend = 1.0 - (-10.0 * time.delta_secs()).exp();
  for (mut visual, mut transform) in &mut trucks {
    if let Some(truck) = host
      .snapshot
      .trucks
      .iter()
      .find(|truck| truck.id == visual.id)
    {
      visual.target = grid_to_world(truck.position);
    }
    let position = transform.translation.truncate().lerp(visual.target, blend);
    transform.translation.x = position.x;
    transform.translation.y = position.y;
  }
}

fn update_ui_text(
  host: Res<SimHost>,
  mut last_revision: Local<u64>,
  mut texts: Query<(
    &mut Text,
    Option<&StatusCount>,
    Option<&MarketStatusText>,
    Option<&PermitText>,
    Option<&SelectionText>,
    Option<&FeedbackText>,
  )>,
) {
  if *last_revision == host.snapshot_revision && !host.is_changed() {
    return;
  }
  *last_revision = host.snapshot_revision;

  for (mut text, count, market, permits, selection, feedback) in &mut texts {
    if let Some(count) = count {
      *text = Text::new(total_item(&host.snapshot, count.0).to_string());
    } else if market.is_some() {
      *text = Text::new(format!(
        "DEMAND {}{}SOLD {}{}${}",
        host.snapshot.market.remaining_demand,
        INLINE_SEPARATOR,
        host.snapshot.market.sold_total,
        INLINE_SEPARATOR,
        host.snapshot.market.revenue
      ));
    } else if permits.is_some() {
      *text = Text::new(format!(
        "BUILDINGS {} / {}{}{}",
        host.snapshot.allowance.used,
        host.snapshot.allowance.limit,
        INLINE_SEPARATOR,
        host.snapshot.allowance.next_unlock_at_sales.map_or_else(
          || "ALL SLOTS UNLOCKED".into(),
          |sales| format!("NEXT AT {sales} SALES")
        )
      ));
    } else if selection.is_some() {
      *text = Text::new(selected_building_text(&host));
    } else if feedback.is_some() {
      *text = Text::new(host.feedback.clone());
    }
  }
}

fn update_hover_cursor(
  hover: Res<HoverCell>,
  host: Res<SimHost>,
  mut cursor: Single<(&mut Transform, &mut Visibility, &mut Sprite), With<HoverCursor>>,
) {
  let Some(cell) = hover.0 else {
    *cursor.1 = Visibility::Hidden;
    return;
  };
  *cursor.1 = Visibility::Visible;
  let position = grid_to_world(cell);
  cursor.0.translation.x = position.x;
  cursor.0.translation.y = position.y;
  cursor.2.color = match host.tool {
    ToolMode::Inspect => Color::srgba(0.48, 0.88, 0.62, 0.24),
    ToolMode::Road => Color::srgba(0.32, 0.68, 0.92, 0.28),
    ToolMode::Erase => Color::srgba(0.92, 0.32, 0.28, 0.28),
    ToolMode::Building => Color::srgba(0.95, 0.72, 0.24, 0.28),
  };
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
    let selected = match button.0 {
      ControlAction::SetTool(tool) => host.tool == tool,
      ControlAction::Configure(recipe) => {
        host.selected_building.and_then(|id| {
          host
            .snapshot
            .buildings
            .iter()
            .find(|building| building.id == id)
            .and_then(|building| building.recipe)
        }) == Some(recipe)
      }
      ControlAction::TogglePause => !host.paused,
      ControlAction::ToggleSpeed => host.ticks_per_second == FAST_TICKS_PER_SECOND,
      _ => false,
    };
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

fn grid_to_world(cell: GridPosition) -> Vec2 {
  Vec2::new(
    (cell.x as f32 - (COMPACT_WORLD_WIDTH - 1) as f32 / 2.0) * CELL_SIZE,
    (cell.y as f32 - (COMPACT_WORLD_HEIGHT - 1) as f32 / 2.0) * CELL_SIZE,
  )
}

fn world_to_grid(world: Vec2) -> Option<GridPosition> {
  let x = ((world.x / CELL_SIZE) + COMPACT_WORLD_WIDTH as f32 / 2.0).floor() as i32;
  let y = ((world.y / CELL_SIZE) + COMPACT_WORLD_HEIGHT as f32 / 2.0).floor() as i32;
  (x >= 0 && y >= 0 && x < COMPACT_WORLD_WIDTH && y < COMPACT_WORLD_HEIGHT)
    .then_some(GridPosition { x, y })
}

fn compact_zoom_scale(level: u8, viewport_width: f32, viewport_height: f32) -> f32 {
  let available_height = (viewport_height - TOP_BAR_HEIGHT).max(1.0);
  let world_width = COMPACT_WORLD_WIDTH as f32 * CELL_SIZE;
  let world_height = COMPACT_WORLD_HEIGHT as f32 * CELL_SIZE;
  let overview = (world_width / viewport_width.max(1.0)).max(world_height / available_height);
  let detail = (MIN_VISIBLE_CELLS * CELL_SIZE / viewport_width.max(1.0))
    .max(MIN_VISIBLE_CELLS * CELL_SIZE / available_height)
    .min(overview);
  let level = level.clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL);
  let progress = f32::from(level - MIN_ZOOM_LEVEL) / f32::from(MAX_ZOOM_LEVEL - MIN_ZOOM_LEVEL);
  detail * (overview / detail).powf(progress)
}

fn pointer_over_ui(position: Vec2, window_size: Vec2) -> bool {
  position.y <= TOP_BAR_HEIGHT
    || (position.x >= window_size.x - PANEL_WIDTH - 18.0
      && position.y >= window_size.y - PANEL_HEIGHT - 18.0)
}

fn inspect_cell(snapshot: &CompactSnapshot, cell: GridPosition) -> String {
  if cell == snapshot.warehouse_position {
    return format!(
      "Warehouse // demand {} // sold {} // revenue ${}",
      snapshot.market.remaining_demand, snapshot.market.sold_total, snapshot.market.revenue
    );
  }
  if let Some(deposit) = snapshot
    .deposits
    .iter()
    .find(|deposit| deposit.position == cell)
  {
    return format!(
      "{} deposit // stock {} // remaining {}",
      item_name(deposit.item),
      deposit.stockpile,
      deposit.remaining
    );
  }
  if let Some(building) = snapshot
    .buildings
    .iter()
    .find(|building| building.position == cell)
  {
    return format!(
      "Factory {} // {} // input {} // output {}",
      building.id,
      building.recipe.map_or("choose recipe", CompactRecipe::name),
      building.input_stock,
      building.output_stock
    );
  }
  if snapshot.roads.contains(&cell) {
    return "Road // free // shared capacity".into();
  }
  "Empty cell.".into()
}

fn selected_building_text(host: &SimHost) -> String {
  let Some(id) = host.selected_building else {
    return "SELECTED // NONE".into();
  };
  let recipe = host
    .snapshot
    .buildings
    .iter()
    .find(|building| building.id == id)
    .and_then(|building| building.recipe)
    .map_or("CHOOSE RECIPE", CompactRecipe::name);
  format!("SELECTED // FACTORY {id} // {recipe}")
}

fn total_item(snapshot: &CompactSnapshot, item: ItemId) -> u32 {
  let deposits = snapshot
    .deposits
    .iter()
    .filter(|deposit| deposit.item == item)
    .map(|deposit| deposit.stockpile)
    .sum::<u32>();
  let buildings = snapshot
    .buildings
    .iter()
    .map(|building| {
      building.recipe.map_or(0, |recipe| {
        u32::from(recipe.input() == item) * building.input_stock
          + u32::from(recipe.output() == item) * building.output_stock
      })
    })
    .sum::<u32>();
  let trucks = snapshot
    .trucks
    .iter()
    .filter(|truck| truck.cargo_item == Some(item))
    .map(|truck| truck.cargo_quantity)
    .sum::<u32>();
  deposits
    .saturating_add(buildings)
    .saturating_add(trucks)
    .saturating_add(snapshot.warehouse_stock.get(&item).copied().unwrap_or(0))
}

fn item_name(item: ItemId) -> &'static str {
  match item {
    IRON_ORE => "iron ore",
    COPPER_ORE => "copper ore",
    IRON_BARS => "iron bars",
    COPPER_BARS => "copper bars",
    _ => "item",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_a_running_shell_redraws_every_frame() {
    assert_eq!(UpdateMode::Continuous, pacing_for(false).focused_mode);
    assert_ne!(UpdateMode::Continuous, pacing_for(true).focused_mode);
    assert_ne!(UpdateMode::Continuous, pacing_for(true).unfocused_mode);
  }

  #[test]
  fn compact_grid_conversion_round_trips_every_cell() {
    for y in 0..COMPACT_WORLD_HEIGHT {
      for x in 0..COMPACT_WORLD_WIDTH {
        let cell = GridPosition { x, y };
        assert_eq!(Some(cell), world_to_grid(grid_to_world(cell)));
      }
    }
  }

  #[test]
  fn overview_is_the_maximum_zoom_and_detail_is_about_ten_cells() {
    let overview = compact_zoom_scale(MAX_ZOOM_LEVEL, 1180.0, 720.0);
    let detail = compact_zoom_scale(MIN_ZOOM_LEVEL, 1180.0, 720.0);

    assert!(overview > detail);
    assert!((overview - 1600.0 / (720.0 - TOP_BAR_HEIGHT)).abs() < 0.01);
    assert!((detail - 1000.0 / (720.0 - TOP_BAR_HEIGHT)).abs() < 0.01);
  }

  #[test]
  fn player_shell_starts_paused_on_the_only_player_facing_world() {
    let host = SimHost::new();

    assert!(host.paused);
    assert_eq!(COMPACT_SCENARIO_NAME, host.snapshot.name);
    assert_eq!((16, 16), (host.snapshot.width, host.snapshot.height));
    assert!(host.snapshot.buildings.is_empty());
  }

  #[test]
  fn placement_and_recipe_controls_mutate_authoritative_state() {
    let mut host = SimHost::new();
    host.tool = ToolMode::Building;
    host.edit_cell(GridPosition { x: 7, y: 10 });

    let selected = host.selected_building.expect("placed factory is selected");
    host.configure_selected(CompactRecipe::IronBars);
    assert_eq!(
      Some(CompactRecipe::IronBars),
      host
        .snapshot
        .buildings
        .iter()
        .find(|building| building.id == selected)
        .and_then(|building| building.recipe)
    );
  }

  #[test]
  fn top_status_counts_all_authoritative_stockpile_locations() {
    let mut game = CompactGame::new();
    let mut snapshot = game.snapshot();
    for _ in 0..8 {
      snapshot = game.step();
    }

    assert_eq!(8, total_item(&snapshot, IRON_ORE));
    assert_eq!(8, total_item(&snapshot, COPPER_ORE));
    assert_eq!(0, total_item(&snapshot, IRON_BARS));
  }

  #[test]
  fn controls_use_literal_double_slash_separators() {
    let host = SimHost::new();
    assert!(selected_building_text(&host).contains(" // "));
    assert!(inspect_cell(&host.snapshot, host.snapshot.warehouse_position).contains(" // "));
  }

  #[test]
  fn pointer_edits_do_not_leak_through_the_two_ui_surfaces() {
    let window = Vec2::new(1180.0, 720.0);

    assert!(pointer_over_ui(Vec2::new(400.0, 60.0), window));
    assert!(pointer_over_ui(Vec2::new(1000.0, 600.0), window));
    assert!(!pointer_over_ui(Vec2::new(400.0, 400.0), window));
  }

  #[test]
  fn accepted_art_paths_remain_local_and_stable() {
    for path in [
      GROUND_ART,
      ROAD_ART,
      TRUCK_ART,
      IRON_DEPOSIT_ART,
      COPPER_DEPOSIT_ART,
      FOUNDRY_ART,
      FACTORY_ART,
      WAREHOUSE_ART,
      IRON_ORE_ART,
      IRON_BARS_ART,
    ] {
      assert!(path.starts_with("factory/"));
      assert!(path.ends_with(".png"));
    }
  }
}
