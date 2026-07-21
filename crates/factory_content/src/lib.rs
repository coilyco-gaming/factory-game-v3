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
pub const STONE: ItemId = ItemId::new("stone");

pub const IRON_BARS_SCENARIO: ScenarioId = ScenarioId::new("iron-bars");

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
    }
  }

  pub fn with_create_from_nothing(mut self) -> Self {
    self.create_from_nothing = true;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScenarioDefinition {
  pub id: ScenarioId,
  pub name: String,
  pub source_item: ItemId,
  pub product_item: ItemId,
  pub source_deposit: u32,
  pub mining_speed: u32,
  pub hauler_capacity: u32,
  pub craft_input_buffer: u32,
  pub craft_output_buffer: u32,
}

impl ScenarioDefinition {
  pub fn new(
    id: ScenarioId,
    name: impl Into<String>,
    source_item: ItemId,
    product_item: ItemId,
    source_deposit: u32,
    mining_speed: u32,
    hauler_capacity: u32,
    craft_input_buffer: u32,
    craft_output_buffer: u32,
  ) -> Self {
    Self {
      id,
      name: name.into(),
      source_item,
      product_item,
      source_deposit,
      mining_speed,
      hauler_capacity,
      craft_input_buffer,
      craft_output_buffer,
    }
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
      STONE,
      ItemDefinition::new(STONE, "Stone", 1, 1, 100, 0, 1, BTreeMap::new())
        .with_create_from_nothing(),
    );

    let mut scenarios = BTreeMap::new();
    scenarios.insert(
      IRON_BARS_SCENARIO,
      ScenarioDefinition::new(
        IRON_BARS_SCENARIO,
        "Iron Bars",
        IRON_ORE,
        IRON_BARS,
        9,
        3,
        3,
        6,
        20,
      ),
    );

    Self { items, scenarios }
  }

  pub fn item(&self, id: ItemId) -> &ItemDefinition {
    self.items
      .get(&id)
      .unwrap_or_else(|| panic!("missing item definition for {id}"))
  }

  pub fn scenario(&self, id: ScenarioId) -> &ScenarioDefinition {
    self.scenarios
      .get(&id)
      .unwrap_or_else(|| panic!("missing scenario definition for {id}"))
  }
}
