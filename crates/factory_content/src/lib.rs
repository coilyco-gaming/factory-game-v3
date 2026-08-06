use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ItemId(&'static str);

impl ItemId {
  pub const fn new(value: &'static str) -> Self {
    Self(value)
  }

  pub const fn as_str(self) -> &'static str {
    self.0
  }
}

impl fmt::Display for ItemId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.0)
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ScenarioId(&'static str);

impl ScenarioId {
  pub const fn new(value: &'static str) -> Self {
    Self(value)
  }

  pub const fn as_str(self) -> &'static str {
    self.0
  }
}

impl fmt::Display for ScenarioId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.0)
  }
}

pub const IRON_ORE: ItemId = ItemId::new("iron_ore");
pub const IRON_BARS: ItemId = ItemId::new("iron_bars");
pub const COPPER_ORE: ItemId = ItemId::new("copper_ore");
pub const COPPER_BARS: ItemId = ItemId::new("copper_bars");
pub const COAL: ItemId = ItemId::new("coal");
pub const STONE: ItemId = ItemId::new("stone");
pub const BUILDING_MATERIALS: ItemId = ItemId::new("building_materials");
pub const MOTORS: ItemId = ItemId::new("motors");
pub const CIRCUITS: ItemId = ItemId::new("circuits");
pub const FRAMES: ItemId = ItemId::new("frames");
pub const STORAGE_WAREHOUSE: ItemId = ItemId::new("storage_warehouse");
pub const COAL_PLANT: ItemId = ItemId::new("coal_plant");
pub const FACTORY_BUILDING: ItemId = ItemId::new("factory");
pub const MINING_DRILL: ItemId = ItemId::new("mining_drill");

pub const IRON_BARS_SCENARIO: ScenarioId = ScenarioId::new("iron-bars");
pub const IRON_BARS_FLEET_SCENARIO: ScenarioId = ScenarioId::new("iron-bars-fleet");
pub const BUILDING_MATERIALS_SCENARIO: ScenarioId = ScenarioId::new("building-materials");
pub const POWERED_IRONWORKS_SCENARIO: ScenarioId = ScenarioId::new("powered-ironworks");
pub const DEPLOYMENT_DEMO_SCENARIO: ScenarioId = ScenarioId::new("deployment-demo");
pub const PATHFINDING_DEMO_SCENARIO: ScenarioId = ScenarioId::new("pathfinding-demo");
pub const PRODUCTION_CHAIN_SCENARIO: ScenarioId = ScenarioId::new("production-chain");
pub const DISTRIBUTED_CHAIN_SCENARIO: ScenarioId = ScenarioId::new("distributed-chain");
pub const POWER_LINE_SCENARIO: ScenarioId = ScenarioId::new("power-line-demo");
pub const BUILDING_DEPLOYMENT_SCENARIO: ScenarioId = ScenarioId::new("building-deployment");
pub const HYBRID_GRID_SCENARIO: ScenarioId = ScenarioId::new("hybrid-grid");
pub const V2_WORLD_SCENARIO: ScenarioId = ScenarioId::new("v2-world");
pub const LEGACY_ASSEMBLY_YARD_SCENARIO: ScenarioId = ScenarioId::new("legacy-assembly-yard");
pub const TWIN_PLANT_BASIN_SCENARIO: ScenarioId = ScenarioId::new("twin-plant-basin");
pub const FOUR_CORNERS_WORKS_SCENARIO: ScenarioId = ScenarioId::new("four-corners-works");

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ItemDefinition {
  pub id: ItemId,
  pub name: String,
  pub weight: u32,
  pub volume: u32,
  pub stack_size: u32,
  pub craft_time: u32,
  pub craft_output: u32,
  pub ingredients: BTreeMap<ItemId, u32>,
  pub create_from_nothing: bool,
  pub can_spawn_game_object: bool,
}

impl ItemDefinition {
  pub fn new(
    id: ItemId,
    name: impl Into<String>,
    weight: u32,
    volume: u32,
    stack_size: u32,
    craft_time: u32,
    craft_output: u32,
    ingredients: BTreeMap<ItemId, u32>,
  ) -> Self {
    Self {
      id,
      name: name.into(),
      weight,
      volume,
      stack_size,
      craft_time,
      craft_output,
      ingredients,
      create_from_nothing: false,
      can_spawn_game_object: false,
    }
  }

  pub fn with_create_from_nothing(mut self) -> Self {
    self.create_from_nothing = true;
    self
  }

  pub fn with_spawnable_game_object(mut self) -> Self {
    self.can_spawn_game_object = true;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceSpec {
  pub item: ItemId,
  pub deposit: u32,
  pub mining_speed: u32,
  pub requires_deployment: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildSiteSpec {
  pub item: ItemId,
  pub position: GridPoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RadarSpec {
  pub deployment_item: ItemId,
  pub target_item: ItemId,
  pub position: GridPoint,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GridPoint {
  pub x: i32,
  pub y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LayoutSpec {
  pub width: i32,
  pub height: i32,
  pub source_positions: Vec<GridPoint>,
  pub road_position: GridPoint,
  pub factory_positions: Vec<GridPoint>,
  pub generator_positions: Vec<GridPoint>,
  pub hauler_positions: Vec<GridPoint>,
  pub obstacles: Vec<GridPoint>,
}

impl LayoutSpec {
  pub fn linear(source_count: u16, include_generator: bool) -> Self {
    Self {
      width: 3,
      height: i32::from(source_count).max(2),
      source_positions: (0..source_count)
        .map(|index| GridPoint {
          x: 0,
          y: i32::from(index),
        })
        .collect(),
      road_position: GridPoint { x: 1, y: 0 },
      factory_positions: vec![GridPoint { x: 2, y: 0 }],
      generator_positions: include_generator
        .then_some(GridPoint { x: 2, y: 1 })
        .into_iter()
        .collect(),
      hauler_positions: Vec::new(),
      obstacles: Vec::new(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactorySpec {
  pub product_item: ItemId,
  pub input_buffer: u32,
  pub output_buffer: u32,
  pub starting_items: BTreeMap<ItemId, u32>,
}

impl FactorySpec {
  pub fn new(product_item: ItemId, input_buffer: u32, output_buffer: u32) -> Self {
    Self {
      product_item,
      input_buffer,
      output_buffer,
      starting_items: BTreeMap::new(),
    }
  }

  pub fn with_starting_items(mut self, starting_items: BTreeMap<ItemId, u32>) -> Self {
    self.starting_items = starting_items;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PowerSpec {
  pub generators: Vec<GeneratorSpec>,
  pub mining_cost: u32,
  pub dispatch_cost: u32,
  pub production_cost: u32,
  pub object_batteries: ObjectBatterySpec,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObjectBatterySpec {
  pub source_capacity: u32,
  pub factory_capacity: u32,
  pub hauler_capacity: u32,
  pub start_charged: bool,
}

impl Default for ObjectBatterySpec {
  fn default() -> Self {
    Self {
      source_capacity: 100,
      factory_capacity: 1_000,
      hauler_capacity: 250,
      start_charged: false,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratorSpec {
  pub item: Option<ItemId>,
  pub fuel_item: Option<ItemId>,
  pub initial_fuel: u32,
  pub fuel_buffer: u32,
  pub burn_rate: u32,
  pub gain_rate: u32,
  pub grid_capacity: u32,
}

impl GeneratorSpec {
  pub fn coal_plant(initial_fuel: u32) -> Self {
    Self {
      item: Some(COAL_PLANT),
      fuel_item: Some(COAL),
      initial_fuel,
      fuel_buffer: 4_000,
      burn_rate: 4,
      gain_rate: 160,
      grid_capacity: 10_000,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScenarioDefinition {
  pub id: ScenarioId,
  pub name: String,
  pub sources: Vec<SourceSpec>,
  pub factories: Vec<FactorySpec>,
  pub build_sites: Vec<BuildSiteSpec>,
  pub radars: Vec<RadarSpec>,
  pub hauler_count: u32,
  pub hauler_capacity: u32,
  pub hauler_weight_capacity: u32,
  pub hauler_volume_capacity: u32,
  pub power: Option<PowerSpec>,
  pub layout: LayoutSpec,
}

const V2_MAP_SIZE: i32 = 50;
const V2_ORE_PER_ITEM: usize = 141;
const V2_RANDOM_SEED: u32 = 4_382_721;

const fn v2_position(x: i32, y: i32) -> GridPoint {
  GridPoint {
    x: V2_MAP_SIZE / 2 + x,
    y: V2_MAP_SIZE / 2 + y,
  }
}

struct V2WorldRng(u32);

impl V2WorldRng {
  fn new(seed: u32) -> Self {
    Self(seed)
  }

  fn next(&mut self) -> u32 {
    self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    self.0
  }
}

fn v2_ore_position(rng: &mut V2WorldRng, occupied: &mut BTreeSet<GridPoint>) -> GridPoint {
  let map_size = u32::try_from(V2_MAP_SIZE).expect("v2 map size fits u32");
  let valid = |position: GridPoint| {
    let dx = position.x - V2_MAP_SIZE / 2;
    let dy = position.y - V2_MAP_SIZE / 2;
    dx * dx + dy * dy > 100 && !occupied.contains(&position)
  };
  for _ in 0..10 {
    let position = GridPoint {
      x: i32::try_from(rng.next() % map_size).expect("generated x fits i32"),
      y: i32::try_from(rng.next() % map_size).expect("generated y fits i32"),
    };
    if valid(position) {
      occupied.insert(position);
      return position;
    }
  }
  for y in 0..V2_MAP_SIZE {
    for x in 0..V2_MAP_SIZE {
      let position = GridPoint { x, y };
      if valid(position) {
        occupied.insert(position);
        return position;
      }
    }
  }
  panic!("v2 world has space for every generated ore deposit")
}

fn v2_world_scenario() -> ScenarioDefinition {
  let mut rng = V2WorldRng::new(V2_RANDOM_SEED);
  let mut occupied = BTreeSet::new();
  let mut sources = Vec::with_capacity(V2_ORE_PER_ITEM * 3 + 1);
  let mut source_positions = Vec::with_capacity(V2_ORE_PER_ITEM * 3 + 1);
  for item in [IRON_ORE, COPPER_ORE, COAL] {
    for _ in 0..V2_ORE_PER_ITEM {
      source_positions.push(v2_ore_position(&mut rng, &mut occupied));
      sources.push(SourceSpec {
        item,
        deposit: 2_000 + (rng.next() % 200) * 20,
        mining_speed: 20,
        requires_deployment: true,
      });
    }
  }
  source_positions.push(v2_position(6, 0));
  sources.push(SourceSpec {
    item: STONE,
    deposit: 0,
    mining_speed: 20,
    requires_deployment: false,
  });

  let factories = vec![
    FactorySpec::new(IRON_BARS, 100, 100),
    FactorySpec::new(COPPER_BARS, 100, 100),
    FactorySpec::new(IRON_BARS, 100, 100),
    FactorySpec::new(COPPER_BARS, 100, 100),
    FactorySpec::new(IRON_BARS, 100, 100),
    FactorySpec::new(IRON_BARS, 100, 100),
    FactorySpec::new(COPPER_BARS, 100, 100),
    FactorySpec::new(MOTORS, 100, 20),
    FactorySpec::new(FRAMES, 100, 20),
    FactorySpec::new(MINING_DRILL, 100, 10)
      .with_starting_items(BTreeMap::from([(MINING_DRILL, 10)])),
    FactorySpec::new(COAL_PLANT, 100, 1),
    FactorySpec::new(CIRCUITS, 100, 20),
    FactorySpec::new(FRAMES, 100, 20),
    FactorySpec::new(MOTORS, 100, 20),
    FactorySpec::new(BUILDING_MATERIALS, 100, 20),
  ];
  ScenarioDefinition {
    id: V2_WORLD_SCENARIO,
    name: "V3 50x50 Factory World".into(),
    sources,
    factories,
    build_sites: Vec::new(),
    radars: vec![
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: IRON_ORE,
        position: v2_position(0, 0),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COPPER_ORE,
        position: v2_position(0, 1),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COAL,
        position: v2_position(0, 2),
      },
      RadarSpec {
        deployment_item: COAL_PLANT,
        target_item: COAL,
        position: v2_position(0, 3),
      },
    ],
    hauler_count: 15,
    hauler_capacity: 500,
    hauler_weight_capacity: 500,
    hauler_volume_capacity: 500,
    power: Some(PowerSpec {
      generators: vec![GeneratorSpec::coal_plant(4_000)],
      mining_cost: 2,
      dispatch_cost: 1,
      production_cost: 2,
      object_batteries: ObjectBatterySpec {
        source_capacity: 1_000,
        factory_capacity: 1_000,
        hauler_capacity: 250,
        start_charged: true,
      },
    }),
    layout: LayoutSpec {
      width: V2_MAP_SIZE,
      height: V2_MAP_SIZE,
      source_positions,
      road_position: v2_position(0, 0),
      factory_positions: vec![
        v2_position(2, 0),
        v2_position(3, 0),
        v2_position(4, 0),
        v2_position(7, 2),
        v2_position(7, 1),
        v2_position(7, 3),
        v2_position(5, 0),
        v2_position(2, 1),
        v2_position(4, 1),
        v2_position(3, 2),
        v2_position(5, 2),
        v2_position(5, 1),
        v2_position(6, 2),
        v2_position(6, 3),
        v2_position(6, 1),
      ],
      generator_positions: vec![v2_position(2, 2)],
      hauler_positions: vec![
        v2_position(-1, 0),
        v2_position(-1, 1),
        v2_position(-1, 2),
        v2_position(1, 0),
        v2_position(1, 1),
        v2_position(1, 2),
        v2_position(1, 3),
        v2_position(2, 3),
        v2_position(3, 3),
        v2_position(4, 3),
        v2_position(7, 0),
        v2_position(7, 1),
        v2_position(7, 2),
        v2_position(7, 3),
        v2_position(7, 4),
      ],
      obstacles: Vec::new(),
    },
  }
}

fn authored_sources(
  fields: [(ItemId, &[GridPoint]); 3],
  stone_position: GridPoint,
  requires_deployment: bool,
) -> (Vec<SourceSpec>, Vec<GridPoint>) {
  let mut sources = Vec::new();
  let mut positions = Vec::new();
  for (item, field) in fields {
    for position in field {
      sources.push(SourceSpec {
        item,
        deposit: 4_000,
        mining_speed: 20,
        requires_deployment,
      });
      positions.push(*position);
    }
  }
  sources.push(SourceSpec {
    item: STONE,
    deposit: 0,
    mining_speed: 20,
    requires_deployment: false,
  });
  positions.push(stone_position);
  (sources, positions)
}

fn world_power(generators: Vec<GeneratorSpec>) -> PowerSpec {
  PowerSpec {
    generators,
    mining_cost: 2,
    dispatch_cost: 1,
    production_cost: 2,
    object_batteries: ObjectBatterySpec {
      source_capacity: 1_000,
      factory_capacity: 1_000,
      hauler_capacity: 250,
      start_charged: true,
    },
  }
}

fn legacy_assembly_yard_scenario() -> ScenarioDefinition {
  const IRON_FIELD: [GridPoint; 8] = [
    GridPoint { x: 4, y: 7 },
    GridPoint { x: 7, y: 7 },
    GridPoint { x: 10, y: 7 },
    GridPoint { x: 13, y: 7 },
    GridPoint { x: 4, y: 11 },
    GridPoint { x: 7, y: 11 },
    GridPoint { x: 10, y: 11 },
    GridPoint { x: 13, y: 11 },
  ];
  const COPPER_FIELD: [GridPoint; 8] = [
    GridPoint { x: 36, y: 7 },
    GridPoint { x: 39, y: 7 },
    GridPoint { x: 42, y: 7 },
    GridPoint { x: 45, y: 7 },
    GridPoint { x: 36, y: 11 },
    GridPoint { x: 39, y: 11 },
    GridPoint { x: 42, y: 11 },
    GridPoint { x: 45, y: 11 },
  ];
  const COAL_FIELD: [GridPoint; 8] = [
    GridPoint { x: 13, y: 40 },
    GridPoint { x: 16, y: 40 },
    GridPoint { x: 19, y: 40 },
    GridPoint { x: 22, y: 40 },
    GridPoint { x: 25, y: 40 },
    GridPoint { x: 28, y: 40 },
    GridPoint { x: 31, y: 40 },
    GridPoint { x: 34, y: 40 },
  ];
  let (sources, source_positions) = authored_sources(
    [
      (IRON_ORE, &IRON_FIELD),
      (COPPER_ORE, &COPPER_FIELD),
      (COAL, &COAL_FIELD),
    ],
    v2_position(1, 0),
    false,
  );
  ScenarioDefinition {
    id: LEGACY_ASSEMBLY_YARD_SCENARIO,
    name: "Legacy Assembly Yard".into(),
    sources,
    factories: vec![
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(COPPER_BARS, 100, 100),
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(BUILDING_MATERIALS, 100, 20),
      FactorySpec::new(MOTORS, 100, 20),
      FactorySpec::new(CIRCUITS, 100, 20),
      FactorySpec::new(FRAMES, 100, 20),
      FactorySpec::new(COAL_PLANT, 100, 1),
      FactorySpec::new(FACTORY_BUILDING, 100, 1),
      FactorySpec::new(MINING_DRILL, 100, 10),
    ],
    build_sites: Vec::new(),
    radars: Vec::new(),
    hauler_count: 8,
    hauler_capacity: 500,
    hauler_weight_capacity: 500,
    hauler_volume_capacity: 500,
    power: Some(world_power(vec![GeneratorSpec::coal_plant(4_000)])),
    layout: LayoutSpec {
      width: V2_MAP_SIZE,
      height: V2_MAP_SIZE,
      source_positions,
      road_position: v2_position(0, 0),
      factory_positions: vec![
        v2_position(2, 0),
        v2_position(3, 0),
        v2_position(4, 0),
        v2_position(1, 1),
        v2_position(2, 1),
        v2_position(3, 1),
        v2_position(4, 1),
        v2_position(1, 2),
        v2_position(2, 2),
        v2_position(3, 2),
      ],
      generator_positions: vec![v2_position(0, 0)],
      hauler_positions: vec![
        v2_position(-1, 0),
        v2_position(-1, 1),
        v2_position(-1, 2),
        v2_position(-1, 3),
        v2_position(5, 0),
        v2_position(5, 1),
        v2_position(5, 2),
        v2_position(5, 3),
      ],
      obstacles: Vec::new(),
    },
  }
}

fn twin_plant_basin_scenario() -> ScenarioDefinition {
  const IRON_FIELD: [GridPoint; 8] = [
    GridPoint { x: 5, y: 5 },
    GridPoint { x: 8, y: 5 },
    GridPoint { x: 5, y: 8 },
    GridPoint { x: 8, y: 8 },
    GridPoint { x: 11, y: 8 },
    GridPoint { x: 8, y: 11 },
    GridPoint { x: 11, y: 11 },
    GridPoint { x: 14, y: 11 },
  ];
  const COPPER_FIELD: [GridPoint; 8] = [
    GridPoint { x: 35, y: 5 },
    GridPoint { x: 38, y: 5 },
    GridPoint { x: 41, y: 5 },
    GridPoint { x: 35, y: 8 },
    GridPoint { x: 38, y: 8 },
    GridPoint { x: 41, y: 8 },
    GridPoint { x: 38, y: 11 },
    GridPoint { x: 41, y: 11 },
  ];
  const COAL_FIELD: [GridPoint; 8] = [
    GridPoint { x: 7, y: 37 },
    GridPoint { x: 10, y: 40 },
    GridPoint { x: 13, y: 43 },
    GridPoint { x: 16, y: 40 },
    GridPoint { x: 33, y: 40 },
    GridPoint { x: 36, y: 43 },
    GridPoint { x: 39, y: 40 },
    GridPoint { x: 42, y: 37 },
  ];
  let (sources, source_positions) = authored_sources(
    [
      (IRON_ORE, &IRON_FIELD),
      (COPPER_ORE, &COPPER_FIELD),
      (COAL, &COAL_FIELD),
    ],
    v2_position(6, 0),
    true,
  );
  ScenarioDefinition {
    id: TWIN_PLANT_BASIN_SCENARIO,
    name: "Twin Plant Basin".into(),
    sources,
    factories: vec![
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(COPPER_BARS, 100, 100),
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(MOTORS, 100, 20),
      FactorySpec::new(FRAMES, 100, 20),
      FactorySpec::new(MINING_DRILL, 100, 10)
        .with_starting_items(BTreeMap::from([(MINING_DRILL, 10)])),
    ],
    build_sites: Vec::new(),
    radars: vec![
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: IRON_ORE,
        position: v2_position(0, 0),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COPPER_ORE,
        position: v2_position(0, 1),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COAL,
        position: v2_position(0, 2),
      },
    ],
    hauler_count: 12,
    hauler_capacity: 500,
    hauler_weight_capacity: 500,
    hauler_volume_capacity: 500,
    power: Some(world_power(vec![
      GeneratorSpec::coal_plant(4_000),
      GeneratorSpec::coal_plant(4_000),
    ])),
    layout: LayoutSpec {
      width: V2_MAP_SIZE,
      height: V2_MAP_SIZE,
      source_positions,
      road_position: v2_position(0, 0),
      factory_positions: vec![
        v2_position(2, 0),
        v2_position(3, 0),
        v2_position(4, 0),
        v2_position(2, 1),
        v2_position(4, 1),
        v2_position(3, 2),
      ],
      generator_positions: vec![v2_position(2, 2), v2_position(4, 2)],
      hauler_positions: vec![
        v2_position(1, 0),
        v2_position(1, 1),
        v2_position(1, 2),
        v2_position(1, 3),
        v2_position(2, 3),
        v2_position(3, 3),
        v2_position(4, 3),
        v2_position(5, 0),
        v2_position(5, 1),
        v2_position(5, 2),
        v2_position(5, 3),
        v2_position(5, 4),
      ],
      obstacles: Vec::new(),
    },
  }
}

fn four_corners_works_scenario() -> ScenarioDefinition {
  const IRON_FIELD: [GridPoint; 8] = [
    GridPoint { x: 5, y: 5 },
    GridPoint { x: 9, y: 5 },
    GridPoint { x: 5, y: 9 },
    GridPoint { x: 9, y: 9 },
    GridPoint { x: 5, y: 40 },
    GridPoint { x: 9, y: 40 },
    GridPoint { x: 5, y: 44 },
    GridPoint { x: 9, y: 44 },
  ];
  const COPPER_FIELD: [GridPoint; 8] = [
    GridPoint { x: 40, y: 5 },
    GridPoint { x: 44, y: 5 },
    GridPoint { x: 40, y: 9 },
    GridPoint { x: 44, y: 9 },
    GridPoint { x: 40, y: 40 },
    GridPoint { x: 44, y: 40 },
    GridPoint { x: 40, y: 44 },
    GridPoint { x: 44, y: 44 },
  ];
  const COAL_FIELD: [GridPoint; 8] = [
    GridPoint { x: 20, y: 5 },
    GridPoint { x: 24, y: 5 },
    GridPoint { x: 28, y: 5 },
    GridPoint { x: 32, y: 5 },
    GridPoint { x: 20, y: 44 },
    GridPoint { x: 24, y: 44 },
    GridPoint { x: 28, y: 44 },
    GridPoint { x: 32, y: 44 },
  ];
  let (sources, source_positions) = authored_sources(
    [
      (IRON_ORE, &IRON_FIELD),
      (COPPER_ORE, &COPPER_FIELD),
      (COAL, &COAL_FIELD),
    ],
    v2_position(0, 8),
    true,
  );
  ScenarioDefinition {
    id: FOUR_CORNERS_WORKS_SCENARIO,
    name: "Four Corners Works".into(),
    sources,
    factories: vec![
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(IRON_BARS, 100, 100),
      FactorySpec::new(COPPER_BARS, 100, 100),
      FactorySpec::new(COPPER_BARS, 100, 100),
      FactorySpec::new(COPPER_BARS, 100, 100),
      FactorySpec::new(MOTORS, 100, 20),
      FactorySpec::new(CIRCUITS, 100, 20),
      FactorySpec::new(FRAMES, 100, 20),
      FactorySpec::new(BUILDING_MATERIALS, 100, 20),
      FactorySpec::new(MINING_DRILL, 100, 12)
        .with_starting_items(BTreeMap::from([(MINING_DRILL, 12)])),
      FactorySpec::new(COAL_PLANT, 100, 1),
    ],
    build_sites: Vec::new(),
    radars: vec![
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: IRON_ORE,
        position: v2_position(0, -1),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COPPER_ORE,
        position: v2_position(0, 0),
      },
      RadarSpec {
        deployment_item: MINING_DRILL,
        target_item: COAL,
        position: v2_position(0, 1),
      },
      RadarSpec {
        deployment_item: COAL_PLANT,
        target_item: COAL,
        position: v2_position(0, 2),
      },
    ],
    hauler_count: 16,
    hauler_capacity: 500,
    hauler_weight_capacity: 500,
    hauler_volume_capacity: 500,
    power: Some(world_power(vec![
      GeneratorSpec::coal_plant(4_000),
      GeneratorSpec::coal_plant(4_000),
    ])),
    layout: LayoutSpec {
      width: V2_MAP_SIZE,
      height: V2_MAP_SIZE,
      source_positions,
      road_position: v2_position(0, 0),
      factory_positions: vec![
        v2_position(-5, -3),
        v2_position(-5, 0),
        v2_position(-5, 3),
        v2_position(5, -3),
        v2_position(5, 0),
        v2_position(5, 3),
        v2_position(-2, -5),
        v2_position(2, -5),
        v2_position(-2, 5),
        v2_position(2, 5),
        v2_position(0, 7),
        v2_position(0, -7),
      ],
      generator_positions: vec![v2_position(-1, 0), v2_position(1, 0)],
      hauler_positions: vec![
        v2_position(-3, -2),
        v2_position(-3, -1),
        v2_position(-3, 0),
        v2_position(-3, 1),
        v2_position(-3, 2),
        v2_position(3, -2),
        v2_position(3, -1),
        v2_position(3, 0),
        v2_position(3, 1),
        v2_position(3, 2),
        v2_position(-2, -3),
        v2_position(-1, -3),
        v2_position(0, -3),
        v2_position(1, -3),
        v2_position(2, -3),
        v2_position(0, 3),
      ],
      obstacles: Vec::new(),
    },
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ContentDatabase {
  pub items: BTreeMap<ItemId, ItemDefinition>,
  pub scenarios: BTreeMap<ScenarioId, ScenarioDefinition>,
}

impl ContentDatabase {
  pub fn starter() -> Self {
    let mut items = BTreeMap::new();

    items.insert(
      IRON_ORE,
      ItemDefinition::new(IRON_ORE, "Iron Ore", 1, 1, 100, 1, 1, BTreeMap::new()),
    );

    items.insert(
      IRON_BARS,
      ItemDefinition::new(
        IRON_BARS,
        "Iron Bars",
        1,
        1,
        100,
        1,
        10,
        BTreeMap::from([(IRON_ORE, 3)]),
      ),
    );

    items.insert(
      COPPER_ORE,
      ItemDefinition::new(COPPER_ORE, "Copper Ore", 1, 1, 100, 1, 1, BTreeMap::new()),
    );

    items.insert(
      COPPER_BARS,
      ItemDefinition::new(
        COPPER_BARS,
        "Copper Bars",
        1,
        1,
        100,
        1,
        10,
        BTreeMap::from([(COPPER_ORE, 3)]),
      ),
    );

    items.insert(
      COAL,
      ItemDefinition::new(COAL, "Coal", 1, 1, 100, 1, 1, BTreeMap::new()),
    );

    items.insert(
      STONE,
      ItemDefinition::new(STONE, "Stone", 1, 1, 100, 1, 1, BTreeMap::new())
        .with_create_from_nothing(),
    );

    items.insert(
      BUILDING_MATERIALS,
      ItemDefinition::new(
        BUILDING_MATERIALS,
        "Building Materials",
        20,
        5,
        20,
        5,
        1,
        BTreeMap::from([(IRON_BARS, 4), (STONE, 4)]),
      ),
    );

    items.insert(
      MOTORS,
      ItemDefinition::new(
        MOTORS,
        "Motors",
        1,
        1,
        20,
        5,
        1,
        BTreeMap::from([(IRON_BARS, 2), (COPPER_BARS, 2)]),
      ),
    );

    items.insert(
      CIRCUITS,
      ItemDefinition::new(
        CIRCUITS,
        "Circuits",
        1,
        1,
        20,
        5,
        1,
        BTreeMap::from([(COPPER_BARS, 1)]),
      ),
    );

    items.insert(
      FRAMES,
      ItemDefinition::new(
        FRAMES,
        "Frames",
        10,
        10,
        20,
        5,
        1,
        BTreeMap::from([(IRON_BARS, 4)]),
      ),
    );

    items.insert(
      STORAGE_WAREHOUSE,
      ItemDefinition::new(
        STORAGE_WAREHOUSE,
        "Storage Warehouse",
        400,
        200,
        1,
        40,
        1,
        BTreeMap::from([(FRAMES, 20), (BUILDING_MATERIALS, 20)]),
      )
      .with_spawnable_game_object(),
    );

    items.insert(
      COAL_PLANT,
      ItemDefinition::new(
        COAL_PLANT,
        "Coal Plant",
        400,
        200,
        1,
        40,
        1,
        BTreeMap::from([
          (FRAMES, 20),
          (BUILDING_MATERIALS, 10),
          (MOTORS, 8),
          (CIRCUITS, 16),
        ]),
      )
      .with_spawnable_game_object(),
    );

    items.insert(
      FACTORY_BUILDING,
      ItemDefinition::new(
        FACTORY_BUILDING,
        "Factory",
        400,
        200,
        1,
        40,
        1,
        BTreeMap::from([
          (FRAMES, 20),
          (BUILDING_MATERIALS, 10),
          (CIRCUITS, 16),
          (MOTORS, 16),
        ]),
      )
      .with_spawnable_game_object(),
    );

    items.insert(
      MINING_DRILL,
      ItemDefinition::new(
        MINING_DRILL,
        "Mining Drill",
        50,
        1,
        1,
        5,
        1,
        BTreeMap::from([(FRAMES, 4), (MOTORS, 1)]),
      )
      .with_spawnable_game_object(),
    );

    let mut scenarios = BTreeMap::new();
    scenarios.insert(
      IRON_BARS_SCENARIO,
      ScenarioDefinition {
        id: IRON_BARS_SCENARIO,
        name: "Iron Bars".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 9,
          mining_speed: 3,
          requires_deployment: false,
        }],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 1,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: None,
        layout: LayoutSpec::linear(1, false),
      },
    );
    scenarios.insert(
      IRON_BARS_FLEET_SCENARIO,
      ScenarioDefinition {
        id: IRON_BARS_FLEET_SCENARIO,
        name: "Iron Bars Fleet".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 24,
          mining_speed: 6,
          requires_deployment: false,
        }],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 3,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: None,
        layout: LayoutSpec::linear(1, false),
      },
    );
    scenarios.insert(
      BUILDING_MATERIALS_SCENARIO,
      ScenarioDefinition {
        id: BUILDING_MATERIALS_SCENARIO,
        name: "Building Materials".into(),
        sources: vec![
          SourceSpec {
            item: IRON_BARS,
            deposit: 12,
            mining_speed: 2,
            requires_deployment: false,
          },
          SourceSpec {
            item: STONE,
            deposit: 0,
            mining_speed: 2,
            requires_deployment: false,
          },
        ],
        factories: vec![FactorySpec::new(BUILDING_MATERIALS, 4, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 2,
        hauler_capacity: 2,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: None,
        layout: LayoutSpec::linear(2, false),
      },
    );
    scenarios.insert(
      POWERED_IRONWORKS_SCENARIO,
      ScenarioDefinition {
        id: POWERED_IRONWORKS_SCENARIO,
        name: "Powered Ironworks".into(),
        sources: vec![
          SourceSpec {
            item: IRON_ORE,
            deposit: 30,
            mining_speed: 3,
            requires_deployment: false,
          },
          SourceSpec {
            item: COAL,
            deposit: 18,
            mining_speed: 2,
            requires_deployment: false,
          },
        ],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: Some(PowerSpec {
          generators: vec![GeneratorSpec {
            item: None,
            fuel_item: Some(COAL),
            initial_fuel: 8,
            fuel_buffer: 20,
            burn_rate: 4,
            gain_rate: 160,
            grid_capacity: 10_000,
          }],
          mining_cost: 1,
          dispatch_cost: 1,
          production_cost: 2,
          object_batteries: ObjectBatterySpec::default(),
        }),
        layout: LayoutSpec::linear(2, true),
      },
    );
    scenarios.insert(
      DEPLOYMENT_DEMO_SCENARIO,
      ScenarioDefinition {
        id: DEPLOYMENT_DEMO_SCENARIO,
        name: "Drill Deployment".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 18,
          mining_speed: 3,
          requires_deployment: true,
        }],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 100)
          .with_starting_items(BTreeMap::from([(MINING_DRILL, 1)]))],
        build_sites: Vec::new(),
        radars: vec![RadarSpec {
          deployment_item: MINING_DRILL,
          target_item: IRON_ORE,
          position: GridPoint { x: 1, y: 1 },
        }],
        hauler_count: 1,
        hauler_capacity: 3,
        hauler_weight_capacity: 100,
        hauler_volume_capacity: 100,
        power: None,
        layout: LayoutSpec::linear(1, false),
      },
    );
    scenarios.insert(
      PATHFINDING_DEMO_SCENARIO,
      ScenarioDefinition {
        id: PATHFINDING_DEMO_SCENARIO,
        name: "Obstacle Convoy".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 30,
          mining_speed: 6,
          requires_deployment: false,
        }],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: None,
        layout: LayoutSpec {
          width: 4,
          height: 3,
          source_positions: vec![GridPoint { x: 0, y: 1 }],
          road_position: GridPoint { x: 1, y: 0 },
          factory_positions: vec![GridPoint { x: 3, y: 1 }],
          generator_positions: Vec::new(),
          hauler_positions: Vec::new(),
          obstacles: vec![GridPoint { x: 1, y: 1 }],
        },
      },
    );
    scenarios.insert(
      PRODUCTION_CHAIN_SCENARIO,
      ScenarioDefinition {
        id: PRODUCTION_CHAIN_SCENARIO,
        name: "Drill Production Chain".into(),
        sources: vec![
          SourceSpec {
            item: IRON_ORE,
            deposit: 180,
            mining_speed: 12,
            requires_deployment: false,
          },
          SourceSpec {
            item: COPPER_ORE,
            deposit: 90,
            mining_speed: 8,
            requires_deployment: false,
          },
        ],
        factories: vec![
          FactorySpec::new(IRON_BARS, 30, 100),
          FactorySpec::new(COPPER_BARS, 30, 100),
          FactorySpec::new(FRAMES, 40, 20),
          FactorySpec::new(MOTORS, 40, 20),
          FactorySpec::new(MINING_DRILL, 20, 5),
        ],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 0,
        hauler_capacity: 5,
        hauler_weight_capacity: 100,
        hauler_volume_capacity: 100,
        power: None,
        layout: LayoutSpec {
          width: 4,
          height: 4,
          source_positions: vec![GridPoint { x: 0, y: 1 }, GridPoint { x: 0, y: 3 }],
          road_position: GridPoint { x: 3, y: 3 },
          factory_positions: vec![
            GridPoint { x: 1, y: 1 },
            GridPoint { x: 1, y: 3 },
            GridPoint { x: 2, y: 1 },
            GridPoint { x: 2, y: 2 },
            GridPoint { x: 3, y: 1 },
          ],
          generator_positions: Vec::new(),
          hauler_positions: Vec::new(),
          obstacles: Vec::new(),
        },
      },
    );
    scenarios.insert(
      DISTRIBUTED_CHAIN_SCENARIO,
      ScenarioDefinition {
        id: DISTRIBUTED_CHAIN_SCENARIO,
        name: "Distributed Frame Line".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 48,
          mining_speed: 6,
          requires_deployment: false,
        }],
        factories: vec![
          FactorySpec::new(IRON_BARS, 18, 100),
          FactorySpec::new(FRAMES, 20, 10),
        ],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 2,
        hauler_capacity: 5,
        hauler_weight_capacity: 100,
        hauler_volume_capacity: 100,
        power: None,
        layout: LayoutSpec {
          width: 5,
          height: 3,
          source_positions: vec![GridPoint { x: 0, y: 1 }],
          road_position: GridPoint { x: 1, y: 1 },
          factory_positions: vec![GridPoint { x: 2, y: 0 }, GridPoint { x: 4, y: 1 }],
          generator_positions: Vec::new(),
          hauler_positions: Vec::new(),
          obstacles: Vec::new(),
        },
      },
    );
    scenarios.insert(
      POWER_LINE_SCENARIO,
      ScenarioDefinition {
        id: POWER_LINE_SCENARIO,
        name: "Automatic Grid Link".into(),
        sources: vec![SourceSpec {
          item: IRON_ORE,
          deposit: 36,
          mining_speed: 3,
          requires_deployment: false,
        }],
        factories: vec![FactorySpec::new(IRON_BARS, 12, 40)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 1,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: Some(PowerSpec {
          generators: vec![GeneratorSpec {
            item: None,
            fuel_item: Some(COAL),
            initial_fuel: 40,
            fuel_buffer: 40,
            burn_rate: 4,
            gain_rate: 160,
            grid_capacity: 10_000,
          }],
          mining_cost: 1,
          dispatch_cost: 1,
          production_cost: 2,
          object_batteries: ObjectBatterySpec::default(),
        }),
        layout: LayoutSpec {
          width: 5,
          height: 3,
          source_positions: vec![GridPoint { x: 4, y: 2 }],
          road_position: GridPoint { x: 4, y: 0 },
          factory_positions: vec![GridPoint { x: 4, y: 1 }],
          generator_positions: vec![GridPoint { x: 0, y: 1 }],
          hauler_positions: Vec::new(),
          obstacles: Vec::new(),
        },
      },
    );
    scenarios.insert(
      BUILDING_DEPLOYMENT_SCENARIO,
      ScenarioDefinition {
        id: BUILDING_DEPLOYMENT_SCENARIO,
        name: "Warehouse Construction".into(),
        sources: Vec::new(),
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)
          .with_starting_items(BTreeMap::from([(STORAGE_WAREHOUSE, 1)]))],
        build_sites: vec![BuildSiteSpec {
          item: STORAGE_WAREHOUSE,
          position: GridPoint { x: 4, y: 1 },
        }],
        radars: Vec::new(),
        hauler_count: 1,
        hauler_capacity: 1,
        hauler_weight_capacity: 500,
        hauler_volume_capacity: 250,
        power: None,
        layout: LayoutSpec {
          width: 5,
          height: 3,
          source_positions: Vec::new(),
          road_position: GridPoint { x: 1, y: 1 },
          factory_positions: vec![GridPoint { x: 0, y: 1 }],
          generator_positions: Vec::new(),
          hauler_positions: Vec::new(),
          obstacles: Vec::new(),
        },
      },
    );
    scenarios.insert(
      HYBRID_GRID_SCENARIO,
      ScenarioDefinition {
        id: HYBRID_GRID_SCENARIO,
        name: "Hybrid Generator Grid".into(),
        sources: vec![
          SourceSpec {
            item: IRON_ORE,
            deposit: 30,
            mining_speed: 3,
            requires_deployment: false,
          },
          SourceSpec {
            item: COAL,
            deposit: 18,
            mining_speed: 2,
            requires_deployment: false,
          },
        ],
        factories: vec![FactorySpec::new(IRON_BARS, 6, 20)],
        build_sites: Vec::new(),
        radars: Vec::new(),
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        power: Some(PowerSpec {
          generators: vec![
            GeneratorSpec {
              item: None,
              fuel_item: Some(COAL),
              initial_fuel: 4,
              fuel_buffer: 20,
              burn_rate: 2,
              gain_rate: 80,
              grid_capacity: 1_000,
            },
            GeneratorSpec {
              item: None,
              fuel_item: None,
              initial_fuel: 0,
              fuel_buffer: 0,
              burn_rate: 0,
              gain_rate: 40,
              grid_capacity: 1_000,
            },
          ],
          mining_cost: 1,
          dispatch_cost: 1,
          production_cost: 2,
          object_batteries: ObjectBatterySpec::default(),
        }),
        layout: LayoutSpec {
          width: 4,
          height: 3,
          source_positions: vec![GridPoint { x: 0, y: 0 }, GridPoint { x: 0, y: 2 }],
          road_position: GridPoint { x: 1, y: 0 },
          factory_positions: vec![GridPoint { x: 3, y: 0 }],
          generator_positions: vec![GridPoint { x: 1, y: 1 }, GridPoint { x: 2, y: 1 }],
          hauler_positions: Vec::new(),
          obstacles: Vec::new(),
        },
      },
    );
    scenarios.insert(V2_WORLD_SCENARIO, v2_world_scenario());
    scenarios.insert(
      LEGACY_ASSEMBLY_YARD_SCENARIO,
      legacy_assembly_yard_scenario(),
    );
    scenarios.insert(TWIN_PLANT_BASIN_SCENARIO, twin_plant_basin_scenario());
    scenarios.insert(FOUR_CORNERS_WORKS_SCENARIO, four_corners_works_scenario());

    Self { items, scenarios }
  }

  pub fn item(&self, id: ItemId) -> &ItemDefinition {
    self
      .items
      .get(&id)
      .unwrap_or_else(|| panic!("missing item definition for {id}"))
  }

  pub fn scenario(&self, id: ScenarioId) -> &ScenarioDefinition {
    self
      .scenarios
      .get(&id)
      .unwrap_or_else(|| panic!("missing scenario definition for {id}"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn starter_catalog_matches_the_active_unity_item_set() {
    let content = ContentDatabase::starter();

    assert_eq!(
      vec![
        BUILDING_MATERIALS,
        CIRCUITS,
        COAL,
        COAL_PLANT,
        COPPER_BARS,
        COPPER_ORE,
        FACTORY_BUILDING,
        FRAMES,
        IRON_BARS,
        IRON_ORE,
        MINING_DRILL,
        MOTORS,
        STONE,
        STORAGE_WAREHOUSE,
      ],
      content.items.keys().copied().collect::<Vec<_>>()
    );
  }

  #[test]
  fn starter_catalog_preserves_unity_recipes_and_flags() {
    let content = ContentDatabase::starter();

    assert_eq!(
      BTreeMap::from([(IRON_ORE, 3)]),
      content.item(IRON_BARS).ingredients
    );
    assert_eq!(10, content.item(IRON_BARS).craft_output);
    assert_eq!(
      BTreeMap::from([(COPPER_ORE, 3)]),
      content.item(COPPER_BARS).ingredients
    );
    assert_eq!(10, content.item(COPPER_BARS).craft_output);
    assert_eq!(
      BTreeMap::from([(IRON_BARS, 4), (STONE, 4)]),
      content.item(BUILDING_MATERIALS).ingredients
    );
    assert_eq!(
      BTreeMap::from([(IRON_BARS, 2), (COPPER_BARS, 2)]),
      content.item(MOTORS).ingredients
    );
    assert_eq!(
      BTreeMap::from([(COPPER_BARS, 1)]),
      content.item(CIRCUITS).ingredients
    );
    assert_eq!(
      BTreeMap::from([(IRON_BARS, 4)]),
      content.item(FRAMES).ingredients
    );
    assert_eq!(
      BTreeMap::from([(FRAMES, 4), (MOTORS, 1)]),
      content.item(MINING_DRILL).ingredients
    );
    assert!(content.item(STONE).create_from_nothing);
    for spawnable in [
      STORAGE_WAREHOUSE,
      COAL_PLANT,
      FACTORY_BUILDING,
      MINING_DRILL,
    ] {
      assert!(content.item(spawnable).can_spawn_game_object);
    }
  }

  #[test]
  fn starter_catalog_preserves_unity_physical_and_timing_values() {
    let content = ContentDatabase::starter();

    let building_materials = content.item(BUILDING_MATERIALS);
    assert_eq!(
      (20, 5, 20, 5),
      (
        building_materials.weight,
        building_materials.volume,
        building_materials.stack_size,
        building_materials.craft_time
      )
    );
    for resource in [IRON_ORE, COPPER_ORE, COAL, STONE] {
      assert_eq!(1, content.item(resource).craft_time);
    }
    for bars in [IRON_BARS, COPPER_BARS] {
      assert_eq!(1, content.item(bars).craft_time);
    }
    let frames = content.item(FRAMES);
    assert_eq!(
      (10, 10, 20, 5),
      (
        frames.weight,
        frames.volume,
        frames.stack_size,
        frames.craft_time
      )
    );
    for building in [STORAGE_WAREHOUSE, COAL_PLANT, FACTORY_BUILDING] {
      let definition = content.item(building);
      assert_eq!(
        (400, 200, 1, 40),
        (
          definition.weight,
          definition.volume,
          definition.stack_size,
          definition.craft_time
        )
      );
    }
    let mining_drill = content.item(MINING_DRILL);
    assert_eq!(
      (50, 1, 1, 5),
      (
        mining_drill.weight,
        mining_drill.volume,
        mining_drill.stack_size,
        mining_drill.craft_time
      )
    );
  }

  #[test]
  fn hybrid_grid_pairs_fueled_and_fuel_free_generators() {
    let content = ContentDatabase::starter();
    let scenario = content
      .scenarios
      .get(&HYBRID_GRID_SCENARIO)
      .expect("hybrid scenario exists");
    let power = scenario.power.as_ref().expect("hybrid scenario is powered");

    assert_eq!(2, power.generators.len());
    assert_eq!(2, scenario.layout.generator_positions.len());
    assert_eq!(Some(COAL), power.generators[0].fuel_item);
    assert_eq!(None, power.generators[1].fuel_item);
    assert_eq!(0, power.generators[1].burn_rate);
    assert!(power.generators[1].gain_rate > 0);
  }

  #[test]
  fn v2_world_recreates_the_large_generated_map_and_central_district() {
    let content = ContentDatabase::starter();
    let scenario = content.scenario(V2_WORLD_SCENARIO);

    assert_eq!((50, 50), (scenario.layout.width, scenario.layout.height));
    assert_eq!(424, scenario.sources.len());
    assert_eq!(
      423,
      scenario
        .sources
        .iter()
        .filter(|source| source.item != STONE)
        .count()
    );
    for item in [IRON_ORE, COPPER_ORE, COAL] {
      assert_eq!(
        141,
        scenario
          .sources
          .iter()
          .filter(|source| source.item == item)
          .count()
      );
    }
    assert_eq!(
      scenario.layout.source_positions.len(),
      scenario
        .layout
        .source_positions
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len()
    );
    assert_eq!(15, scenario.factories.len());
    assert_eq!(15, scenario.hauler_count);
    assert_eq!(15, scenario.layout.hauler_positions.len());
    assert_eq!(4, scenario.radars.len());
    assert_eq!(IRON_ORE, scenario.radars[0].target_item);
    assert_eq!(COPPER_ORE, scenario.radars[1].target_item);
    assert_eq!(COAL, scenario.radars[2].target_item);
    assert_eq!(COAL, scenario.radars[3].target_item);
    assert!(scenario.radars[..3]
      .iter()
      .all(|radar| radar.deployment_item == MINING_DRILL));
    assert_eq!(COAL_PLANT, scenario.radars[3].deployment_item);
    assert_eq!(
      Some(COAL_PLANT),
      scenario.power.as_ref().unwrap().generators[0].item
    );
    assert_eq!(10, scenario.factories[9].starting_items[&MINING_DRILL]);
    assert_eq!(
      GridPoint { x: 27, y: 27 },
      scenario.layout.generator_positions[0]
    );
    assert_eq!(
      GridPoint { x: 31, y: 25 },
      scenario.layout.source_positions[423]
    );
    assert_eq!(
      scenario,
      ContentDatabase::starter().scenario(V2_WORLD_SCENARIO)
    );
  }

  #[test]
  fn selectable_worlds_are_distinct_authored_50x50_layouts() {
    let content = ContentDatabase::starter();
    let world_ids = [
      V2_WORLD_SCENARIO,
      LEGACY_ASSEMBLY_YARD_SCENARIO,
      TWIN_PLANT_BASIN_SCENARIO,
      FOUR_CORNERS_WORKS_SCENARIO,
    ];
    let names = world_ids
      .iter()
      .map(|id| content.scenario(*id).name.as_str())
      .collect::<BTreeSet<_>>();

    assert_eq!(world_ids.len(), names.len());
    for id in world_ids {
      let scenario = content.scenario(id);
      assert_eq!((50, 50), (scenario.layout.width, scenario.layout.height));
      assert_eq!(
        scenario.sources.len(),
        scenario.layout.source_positions.len()
      );
      assert_eq!(
        scenario.factories.len(),
        scenario.layout.factory_positions.len()
      );
      assert_eq!(
        usize::try_from(scenario.hauler_count).unwrap(),
        scenario.layout.hauler_positions.len()
      );
    }
  }
}
