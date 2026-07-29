use serde::Serialize;
use std::collections::BTreeMap;
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
pub const MINING_DRILL: ItemId = ItemId::new("mining_drill");

pub const IRON_BARS_SCENARIO: ScenarioId = ScenarioId::new("iron-bars");
pub const IRON_BARS_FLEET_SCENARIO: ScenarioId = ScenarioId::new("iron-bars-fleet");
pub const BUILDING_MATERIALS_SCENARIO: ScenarioId = ScenarioId::new("building-materials");
pub const POWERED_IRONWORKS_SCENARIO: ScenarioId = ScenarioId::new("powered-ironworks");
pub const DEPLOYMENT_DEMO_SCENARIO: ScenarioId = ScenarioId::new("deployment-demo");
pub const PATHFINDING_DEMO_SCENARIO: ScenarioId = ScenarioId::new("pathfinding-demo");

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
  pub factory_position: GridPoint,
  pub power_plant_position: Option<GridPoint>,
  pub obstacles: Vec<GridPoint>,
}

impl LayoutSpec {
  pub fn linear(source_count: u8, include_power_plant: bool) -> Self {
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
      factory_position: GridPoint { x: 2, y: 0 },
      power_plant_position: include_power_plant.then_some(GridPoint { x: 2, y: 1 }),
      obstacles: Vec::new(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PowerSpec {
  pub fuel_item: ItemId,
  pub initial_fuel: u32,
  pub fuel_buffer: u32,
  pub burn_rate: u32,
  pub gain_rate: u32,
  pub grid_capacity: u32,
  pub mining_cost: u32,
  pub dispatch_cost: u32,
  pub production_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScenarioDefinition {
  pub id: ScenarioId,
  pub name: String,
  pub sources: Vec<SourceSpec>,
  pub product_item: ItemId,
  pub hauler_count: u32,
  pub hauler_capacity: u32,
  pub hauler_weight_capacity: u32,
  pub hauler_volume_capacity: u32,
  pub craft_input_buffer: u32,
  pub craft_output_buffer: u32,
  pub power: Option<PowerSpec>,
  pub factory_starting_items: BTreeMap<ItemId, u32>,
  pub layout: LayoutSpec,
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
      ItemDefinition::new(IRON_ORE, "Iron Ore", 1, 1, 100, 0, 1, BTreeMap::new()),
    );

    items.insert(
      IRON_BARS,
      ItemDefinition::new(
        IRON_BARS,
        "Iron Bars",
        1,
        1,
        100,
        2,
        10,
        BTreeMap::from([(IRON_ORE, 3)]),
      ),
    );

    items.insert(
      COPPER_ORE,
      ItemDefinition::new(COPPER_ORE, "Copper Ore", 1, 1, 100, 0, 1, BTreeMap::new()),
    );

    items.insert(
      COPPER_BARS,
      ItemDefinition::new(
        COPPER_BARS,
        "Copper Bars",
        1,
        1,
        100,
        2,
        10,
        BTreeMap::from([(COPPER_ORE, 3)]),
      ),
    );

    items.insert(
      COAL,
      ItemDefinition::new(COAL, "Coal", 1, 1, 100, 0, 1, BTreeMap::new()),
    );

    items.insert(
      STONE,
      ItemDefinition::new(STONE, "Stone", 1, 1, 100, 0, 1, BTreeMap::new())
        .with_create_from_nothing(),
    );

    items.insert(
      BUILDING_MATERIALS,
      ItemDefinition::new(
        BUILDING_MATERIALS,
        "Building Materials",
        1,
        1,
        100,
        2,
        4,
        BTreeMap::from([(IRON_ORE, 2), (STONE, 2)]),
      ),
    );

    items.insert(
      MINING_DRILL,
      ItemDefinition::new(
        MINING_DRILL,
        "Mining Drill",
        50,
        20,
        1,
        5,
        1,
        BTreeMap::new(),
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
        product_item: IRON_BARS,
        hauler_count: 1,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        craft_input_buffer: 6,
        craft_output_buffer: 20,
        power: None,
        factory_starting_items: BTreeMap::new(),
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
        product_item: IRON_BARS,
        hauler_count: 3,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        craft_input_buffer: 6,
        craft_output_buffer: 20,
        power: None,
        factory_starting_items: BTreeMap::new(),
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
            item: IRON_ORE,
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
        product_item: BUILDING_MATERIALS,
        hauler_count: 2,
        hauler_capacity: 2,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        craft_input_buffer: 4,
        craft_output_buffer: 20,
        power: None,
        factory_starting_items: BTreeMap::new(),
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
        product_item: IRON_BARS,
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        craft_input_buffer: 6,
        craft_output_buffer: 20,
        power: Some(PowerSpec {
          fuel_item: COAL,
          initial_fuel: 2,
          fuel_buffer: 5,
          burn_rate: 1,
          gain_rate: 12,
          grid_capacity: 48,
          mining_cost: 1,
          dispatch_cost: 1,
          production_cost: 2,
        }),
        factory_starting_items: BTreeMap::new(),
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
        product_item: IRON_BARS,
        hauler_count: 1,
        hauler_capacity: 3,
        hauler_weight_capacity: 100,
        hauler_volume_capacity: 100,
        craft_input_buffer: 6,
        craft_output_buffer: 20,
        power: None,
        factory_starting_items: BTreeMap::from([(MINING_DRILL, 1)]),
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
        product_item: IRON_BARS,
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
        craft_input_buffer: 6,
        craft_output_buffer: 20,
        power: None,
        factory_starting_items: BTreeMap::new(),
        layout: LayoutSpec {
          width: 4,
          height: 3,
          source_positions: vec![GridPoint { x: 0, y: 1 }],
          road_position: GridPoint { x: 1, y: 0 },
          factory_position: GridPoint { x: 3, y: 1 },
          power_plant_position: None,
          obstacles: vec![GridPoint { x: 1, y: 1 }],
        },
      },
    );

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
