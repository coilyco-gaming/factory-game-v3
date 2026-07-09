use factory_content::{
  ContentDatabase, ItemId, ScenarioDefinition, ScenarioId, IRON_BARS_SCENARIO,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimulationError {
  UnknownScenario(ScenarioId),
  RecipeMissingIngredients(ItemId),
}

impl fmt::Display for SimulationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownScenario(id) => write!(f, "unknown scenario: {id}"),
      Self::RecipeMissingIngredients(id) => {
        write!(f, "recipe for {id} must have exactly one ingredient")
      }
    }
  }
}

impl std::error::Error for SimulationError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InventoryError {
  UnknownItem(ItemId),
  CapacityExceeded {
    item: ItemId,
    requested: u32,
    accepted: u32,
  },
  InsufficientQuantity {
    item: ItemId,
    requested: u32,
    available: u32,
  },
  ReservationBlocked(ItemId),
}

impl fmt::Display for InventoryError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownItem(id) => write!(f, "unknown item: {id}"),
      Self::CapacityExceeded {
        item,
        requested,
        accepted,
      } => write!(
        f,
        "capacity exceeded for {item}: requested {requested}, accepted {accepted}"
      ),
      Self::InsufficientQuantity {
        item,
        requested,
        available,
      } => write!(
        f,
        "insufficient quantity for {item}: requested {requested}, available {available}"
      ),
      Self::ReservationBlocked(item) => write!(f, "reservation blocks {item}"),
    }
  }
}

impl std::error::Error for InventoryError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
  items: BTreeMap<ItemId, u32>,
  weight_capacity: u32,
  volume_capacity: u32,
  reserved_capacity: BTreeMap<ItemId, u32>,
}

impl Inventory {
  pub fn new(weight_capacity: u32, volume_capacity: u32) -> Self {
    Self {
      items: BTreeMap::new(),
      weight_capacity,
      volume_capacity,
      reserved_capacity: BTreeMap::new(),
    }
  }

  pub fn with_reserved_capacity(mut self, item: ItemId, quantity: u32) -> Self {
    self.reserved_capacity.insert(item, quantity);
    self
  }

  pub fn reserve(&mut self, item: ItemId, quantity: u32) {
    self.reserved_capacity.insert(item, quantity);
  }

  pub fn count(&self, item: ItemId) -> u32 {
    self.items.get(&item).copied().unwrap_or(0)
  }

  pub fn is_empty(&self) -> bool {
    self.items.values().all(|quantity| *quantity == 0)
  }

  fn item_allowed(&self, item: ItemId) -> bool {
    self.reserved_capacity.is_empty() || self.reserved_capacity.contains_key(&item)
  }

  fn used_weight(&self, content: &ContentDatabase) -> u32 {
    self.items
      .iter()
      .map(|(item, quantity)| content.item(*item).weight.saturating_mul(*quantity))
      .sum()
  }

  fn used_volume(&self, content: &ContentDatabase) -> u32 {
    self.items
      .iter()
      .map(|(item, quantity)| content.item(*item).volume.saturating_mul(*quantity))
      .sum()
  }

  pub fn remaining_weight(&self, content: &ContentDatabase) -> u32 {
    self.weight_capacity.saturating_sub(self.used_weight(content))
  }

  pub fn remaining_volume(&self, content: &ContentDatabase) -> u32 {
    self.volume_capacity.saturating_sub(self.used_volume(content))
  }

  pub fn max_insertable(&self, content: &ContentDatabase, item: ItemId, requested: u32) -> u32 {
    if requested == 0 || !self.item_allowed(item) {
      return 0;
    }
    let definition = content.item(item);
    let weight_limit = if definition.weight == 0 {
      requested
    } else {
      self.remaining_weight(content) / definition.weight
    };
    let volume_limit = if definition.volume == 0 {
      requested
    } else {
      self.remaining_volume(content) / definition.volume
    };
    requested.min(weight_limit.min(volume_limit))
  }

  pub fn insert_up_to(&mut self, content: &ContentDatabase, item: ItemId, requested: u32) -> u32 {
    let accepted = self.max_insertable(content, item, requested);
    if accepted == 0 {
      return 0;
    }
    *self.items.entry(item).or_insert(0) += accepted;
    accepted
  }

  pub fn insert_exact(
    &mut self,
    content: &ContentDatabase,
    item: ItemId,
    quantity: u32,
  ) -> Result<(), InventoryError> {
    let accepted = self.insert_up_to(content, item, quantity);
    if accepted == quantity {
      Ok(())
    } else if !self.item_allowed(item) {
      Err(InventoryError::ReservationBlocked(item))
    } else {
      Err(InventoryError::CapacityExceeded {
        item,
        requested: quantity,
        accepted,
      })
    }
  }

  pub fn remove_up_to(&mut self, item: ItemId, requested: u32) -> u32 {
    let available = self.count(item);
    let removed = requested.min(available);
    if removed == 0 {
      return 0;
    }
    let remaining = available - removed;
    if remaining == 0 {
      self.items.remove(&item);
    } else {
      self.items.insert(item, remaining);
    }
    removed
  }

  pub fn remove_exact(&mut self, item: ItemId, quantity: u32) -> Result<(), InventoryError> {
    let available = self.count(item);
    if available < quantity {
      return Err(InventoryError::InsufficientQuantity {
        item,
        requested: quantity,
        available,
      });
    }
    self.remove_up_to(item, quantity);
    Ok(())
  }

  pub fn transfer_up_to(
    &mut self,
    content: &ContentDatabase,
    target: &mut Inventory,
    item: ItemId,
    requested: u32,
  ) -> u32 {
    let available = self.count(item);
    let max_target = target.max_insertable(content, item, requested.min(available));
    if max_target == 0 {
      return 0;
    }
    self.remove_up_to(item, max_target);
    target.insert_up_to(content, item, max_target)
  }

  pub fn snapshot(&self) -> InventorySnapshot {
    InventorySnapshot {
      items: self
        .items
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      reserved_capacity: self
        .reserved_capacity
        .iter()
        .map(|(item, quantity)| (item.to_string(), *quantity))
        .collect(),
      weight_capacity: self.weight_capacity,
      volume_capacity: self.volume_capacity,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InventorySnapshot {
  pub items: BTreeMap<String, u32>,
  pub reserved_capacity: BTreeMap<String, u32>,
  pub weight_capacity: u32,
  pub volume_capacity: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
  Source,
  Factory,
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Source => f.write_str("source"),
      Self::Factory => f.write_str("factory"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct RecipeRuntime {
  pub input_item: ItemId,
  pub input_quantity: u32,
  pub output_item: ItemId,
  pub output_quantity: u32,
  pub craft_time: u32,
}

#[derive(Clone, Debug)]
pub struct SourceNode {
  pub stockpile: Inventory,
  pub item: ItemId,
}

#[derive(Clone, Debug)]
pub struct Hauler {
  pub cargo: Inventory,
  pub position: Location,
  pub carry_limit: u32,
}

#[derive(Clone, Debug)]
pub struct Factory {
  pub inventory: Inventory,
  pub recipe: RecipeRuntime,
  pub craft_progress: u32,
  pub crafting: bool,
}

#[derive(Clone, Debug)]
pub struct GameState {
  pub tick: u64,
  pub scenario: ScenarioDefinition,
  pub source: SourceNode,
  pub hauler: Hauler,
  pub factory: Factory,
  pub route: [Location; 2],
  content: ContentDatabase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TickSnapshot {
  pub tick: u64,
  pub scenario: ScenarioSnapshot,
  pub topology: TopologySnapshot,
  pub source: NodeSnapshot,
  pub hauler: HaulerSnapshot,
  pub factory: FactorySnapshot,
  pub events: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScenarioSnapshot {
  pub id: ScenarioId,
  pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologySnapshot {
  pub route: [Location; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NodeSnapshot {
  pub item: ItemId,
  pub stockpile: InventorySnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HaulerSnapshot {
  pub position: Location,
  pub cargo: InventorySnapshot,
  pub carry_limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CraftSnapshot {
  pub input_item: ItemId,
  pub output_item: ItemId,
  pub input_quantity: u32,
  pub output_quantity: u32,
  pub craft_time: u32,
  pub craft_progress: u32,
  pub crafting: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactorySnapshot {
  pub inventory: InventorySnapshot,
  pub craft: CraftSnapshot,
}

impl GameState {
  pub fn new(content: ContentDatabase, scenario_id: ScenarioId) -> Result<Self, SimulationError> {
    let scenario = content
      .scenarios
      .get(&scenario_id)
      .cloned()
      .ok_or(SimulationError::UnknownScenario(scenario_id))?;
    let product = content.item(scenario.product_item).clone();
    if product.ingredients.len() != 1 {
      return Err(SimulationError::RecipeMissingIngredients(product.id));
    }
    let (&input_item, &input_quantity) = product.ingredients.iter().next().unwrap();
    let recipe = RecipeRuntime {
      input_item,
      input_quantity,
      output_item: product.id,
      output_quantity: product.craft_output,
      craft_time: product.craft_time.max(1),
    };

    let mut factory_inventory = Inventory::new(32, 32);
    factory_inventory.reserve(recipe.input_item, scenario.craft_input_buffer);
    factory_inventory.reserve(recipe.output_item, scenario.craft_output_buffer);

    Ok(Self {
      tick: 0,
      source: SourceNode {
        stockpile: {
          let mut stockpile = Inventory::new(1024, 1024);
          stockpile
            .insert_exact(&content, scenario.source_item, scenario.source_stockpile)
            .expect("starter source stockpile fits");
          stockpile
        },
        item: scenario.source_item,
      },
      hauler: Hauler {
        cargo: Inventory::new(32, 32),
        position: Location::Source,
        carry_limit: scenario.hauler_capacity,
      },
      factory: Factory {
        inventory: factory_inventory,
        recipe,
        craft_progress: 0,
        crafting: false,
      },
      route: [Location::Source, Location::Factory],
      scenario,
      content,
    })
  }

  pub fn starter_iron_bars() -> Self {
    Self::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).expect("starter scenario is valid")
  }

  fn collect(&mut self, events: &mut Vec<String>) {
    if self.hauler.position != Location::Source || !self.hauler.cargo.is_empty() {
      return;
    }
    let requested = self.hauler.carry_limit;
    let moved = self
      .source
      .stockpile
      .transfer_up_to(&self.content, &mut self.hauler.cargo, self.source.item, requested);
    if moved > 0 {
      events.push(format!("collect {moved} {} from source", self.source.item));
    }
  }

  fn deliver(&mut self, events: &mut Vec<String>) {
    if self.hauler.position != Location::Factory || self.hauler.cargo.is_empty() {
      return;
    }
    let delivered = self
      .hauler
      .cargo
      .transfer_up_to(
        &self.content,
        &mut self.factory.inventory,
        self.factory.recipe.input_item,
        self.factory.recipe.input_quantity,
      );
    if delivered > 0 {
      events.push(format!(
        "deliver {delivered} {} to factory",
        self.factory.recipe.input_item
      ));
    }
  }

  fn advance_production(&mut self, events: &mut Vec<String>) {
    let mut completed_this_tick = false;
    if self.factory.crafting {
      self.factory.craft_progress += 1;
      events.push(format!(
        "craft {} progress {}/{}",
        self.factory.recipe.output_item, self.factory.craft_progress, self.factory.recipe.craft_time
      ));
      if self.factory.craft_progress >= self.factory.recipe.craft_time {
        let produced = self.factory.inventory.insert_up_to(
          &self.content,
          self.factory.recipe.output_item,
          self.factory.recipe.output_quantity,
        );
        self.factory.crafting = false;
        self.factory.craft_progress = 0;
        completed_this_tick = true;
        events.push(format!(
          "craft {} completed produced {}",
          self.factory.recipe.output_item, produced
        ));
      }
    }

    if !completed_this_tick
      && !self.factory.crafting
      && self.factory.inventory.count(self.factory.recipe.input_item)
        >= self.factory.recipe.input_quantity
    {
      if self
        .factory
        .inventory
        .remove_exact(self.factory.recipe.input_item, self.factory.recipe.input_quantity)
        .is_ok()
      {
        self.factory.crafting = true;
        self.factory.craft_progress = 0;
        events.push(format!("craft {} started", self.factory.recipe.output_item));
      }
    }
  }

  fn move_hauler(&mut self, events: &mut Vec<String>) {
    self.hauler.position = match self.hauler.position {
      Location::Source => Location::Factory,
      Location::Factory => Location::Source,
    };
    events.push(format!("move hauler to {}", self.hauler.position));
  }

  pub fn step(&mut self) -> TickSnapshot {
    self.tick += 1;
    let mut events = Vec::new();
    self.collect(&mut events);
    self.deliver(&mut events);
    self.advance_production(&mut events);
    self.move_hauler(&mut events);
    self.snapshot(events)
  }

  pub fn snapshot(&self, events: Vec<String>) -> TickSnapshot {
    TickSnapshot {
      tick: self.tick,
      scenario: ScenarioSnapshot {
        id: self.scenario.id,
        name: self.scenario.name.clone(),
      },
      topology: TopologySnapshot { route: self.route },
      source: NodeSnapshot {
        item: self.source.item,
        stockpile: self.source.stockpile.snapshot(),
      },
      hauler: HaulerSnapshot {
        position: self.hauler.position,
        cargo: self.hauler.cargo.snapshot(),
        carry_limit: self.hauler.carry_limit,
      },
      factory: FactorySnapshot {
        inventory: self.factory.inventory.snapshot(),
        craft: CraftSnapshot {
          input_item: self.factory.recipe.input_item,
          output_item: self.factory.recipe.output_item,
          input_quantity: self.factory.recipe.input_quantity,
          output_quantity: self.factory.recipe.output_quantity,
          craft_time: self.factory.recipe.craft_time,
          craft_progress: self.factory.craft_progress,
          crafting: self.factory.crafting,
        },
      },
      events,
    }
  }
}

pub fn sample_content() -> ContentDatabase {
  ContentDatabase::starter()
}

pub fn sample_game_state() -> GameState {
  GameState::starter_iron_bars()
}

#[cfg(test)]
mod tests {
  use super::*;
  use factory_content::{
    ContentDatabase, COPPER_BARS, COPPER_ORE, IRON_BARS, IRON_BARS_SCENARIO, IRON_ORE,
  };

  #[test]
  fn inventory_respects_capacity_and_reservations() {
    let content = ContentDatabase::starter();
    let mut inventory = Inventory::new(2, 2);
    assert_eq!(2, inventory.insert_up_to(&content, IRON_ORE, 3));
    assert_eq!(2, inventory.count(IRON_ORE));

    inventory.reserve(IRON_ORE, 4);
    assert_eq!(0, inventory.insert_up_to(&content, COPPER_ORE, 1));

    let mut target = Inventory::new(2, 2).with_reserved_capacity(IRON_ORE, 1);
    let moved = inventory.transfer_up_to(&content, &mut target, IRON_ORE, 3);
    assert_eq!(2, moved);
    assert_eq!(0, inventory.count(IRON_ORE));
    assert_eq!(2, target.count(IRON_ORE));
    assert_eq!(0, target.insert_up_to(&content, COPPER_BARS, 1));
  }

  #[test]
  fn iron_bars_loop_is_deterministic() {
    let mut first = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();
    let mut second = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();

    let first_run: Vec<_> = (0..6).map(|_| first.step()).collect();
    let second_run: Vec<_> = (0..6).map(|_| second.step()).collect();

    assert_eq!(first_run, second_run);
    assert!(first_run.iter().any(|snapshot| {
      snapshot
        .factory
        .inventory
        .items
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    }));
  }
}
