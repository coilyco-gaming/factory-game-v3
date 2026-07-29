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
  pub factory_positions: Vec<GridPoint>,
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
      factory_positions: vec![GridPoint { x: 2, y: 0 }],
      power_plant_position: include_power_plant.then_some(GridPoint { x: 2, y: 1 }),
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
  pub factories: Vec<FactorySpec>,
  pub hauler_count: u32,
  pub hauler_capacity: u32,
  pub hauler_weight_capacity: u32,
  pub hauler_volume_capacity: u32,
  pub power: Option<PowerSpec>,
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
        hauler_count: 2,
        hauler_capacity: 3,
        hauler_weight_capacity: 32,
        hauler_volume_capacity: 32,
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
        factories: vec![
          FactorySpec::new(IRON_BARS, 6, 100)
            .with_starting_items(BTreeMap::from([(MINING_DRILL, 1)])),
        ],
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
          power_plant_position: None,
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
          power_plant_position: None,
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
          power_plant_position: None,
          obstacles: Vec::new(),
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
}
