use crate::dispatch::{DispatchAssignment, DispatchBoard, DispatchIntent, DispatchReceiverState};
use crate::production::{CraftSnapshot, FactoryProduction};
use crate::resources::Inventory;
use factory_content::{ItemId, ScenarioDefinition};
use serde::Serialize;
use std::fmt;

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

impl Location {
  pub fn other(self) -> Self {
    match self {
      Self::Source => Self::Factory,
      Self::Factory => Self::Source,
    }
  }
}

#[derive(Clone, Debug)]
pub struct SourceNode {
  pub stockpile: Inventory,
  pub item: ItemId,
  pub dispatch: DispatchBoard,
}

impl SourceNode {
  pub fn new(stockpile: Inventory, item: ItemId) -> Self {
    Self {
      stockpile,
      item,
      dispatch: DispatchBoard::new(),
    }
  }

  pub fn refresh_dispatch(&mut self, factory_location: Location) {
    self.dispatch.intent = (self.stockpile.count(self.item) > 0).then(|| {
      DispatchIntent::collect(self.item, Location::Source, factory_location)
    });
  }
}

#[derive(Clone, Debug)]
pub struct FactoryNode {
  pub production: FactoryProduction,
  pub dispatch: DispatchBoard,
  pub input_buffer: u32,
}

impl FactoryNode {
  pub fn new(production: FactoryProduction, input_buffer: u32) -> Self {
    Self {
      production,
      dispatch: DispatchBoard::new(),
      input_buffer,
    }
  }

  pub fn refresh_dispatch(&mut self, source_location: Location) {
    let needed = self
      .input_buffer
      .saturating_sub(self.production.inventory.count(self.production.recipe.input_item));
    self.dispatch.intent = (needed > 0).then(|| {
      DispatchIntent::deliver(self.production.recipe.input_item, source_location, Location::Factory)
    });
  }
}

#[derive(Clone, Debug)]
pub struct Hauler {
  pub cargo: Inventory,
  pub position: Location,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

impl Hauler {
  pub fn new(cargo: Inventory, position: Location, carry_limit: u32) -> Self {
    Self {
      cargo,
      position,
      carry_limit,
      dispatch: DispatchReceiverState::Unassigned,
    }
  }

  pub fn assign(&mut self, assignment: DispatchAssignment) {
    self.dispatch = DispatchReceiverState::Assigned(assignment);
  }

  pub fn clear_assignment(&mut self) {
    self.dispatch = DispatchReceiverState::Unassigned;
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceSnapshot {
  pub item: ItemId,
  pub stockpile: crate::resources::InventorySnapshot,
  pub dispatch: DispatchBoard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactorySnapshot {
  pub inventory: crate::resources::InventorySnapshot,
  pub craft: CraftSnapshot,
  pub dispatch: DispatchBoard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HaulerSnapshot {
  pub position: Location,
  pub cargo: crate::resources::InventorySnapshot,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologySnapshot {
  pub route: [Location; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScenarioSnapshot {
  pub id: factory_content::ScenarioId,
  pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TickSnapshot {
  pub tick: u64,
  pub scenario: ScenarioSnapshot,
  pub topology: TopologySnapshot,
  pub source: SourceSnapshot,
  pub hauler: HaulerSnapshot,
  pub factory: FactorySnapshot,
  pub events: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct WorldState {
  pub tick: u64,
  pub scenario: ScenarioDefinition,
  pub source: SourceNode,
  pub hauler: Hauler,
  pub factory: FactoryNode,
  pub route: [Location; 2],
}
