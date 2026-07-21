use crate::dispatch::{DispatchAssignment, DispatchBoard, DispatchIntent, DispatchReceiverState};
use crate::mining::MiningExtractor;
use crate::production::{CraftSnapshot, FactoryProduction};
use crate::resources::Inventory;
use factory_content::{ItemId, ScenarioDefinition};
use serde::Serialize;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeId {
  Source,
  Road,
  Factory,
}

impl fmt::Display for NodeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::Source => "source",
      Self::Road => "road",
      Self::Factory => "factory",
    })
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GridPosition {
  pub x: i32,
  pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologyNode {
  pub id: NodeId,
  pub position: GridPosition,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Topology {
  pub nodes: [TopologyNode; 3],
  pub route: [NodeId; 3],
}

impl Topology {
  pub fn starter() -> Self {
    Self {
      nodes: [
        TopologyNode {
          id: NodeId::Source,
          position: GridPosition { x: 0, y: 0 },
        },
        TopologyNode {
          id: NodeId::Road,
          position: GridPosition { x: 1, y: 0 },
        },
        TopologyNode {
          id: NodeId::Factory,
          position: GridPosition { x: 2, y: 0 },
        },
      ],
      route: [NodeId::Source, NodeId::Road, NodeId::Factory],
    }
  }

  pub fn position(&self, node: NodeId) -> GridPosition {
    self
      .nodes
      .iter()
      .find(|candidate| candidate.id == node)
      .map(|node| node.position)
      .expect("topology contains requested node")
  }

  pub fn route_index(&self, node: NodeId) -> Option<usize> {
    self.route.iter().position(|candidate| *candidate == node)
  }

  pub fn step_toward(&self, from: NodeId, target: NodeId) -> NodeId {
    let from_index = self.route_index(from);
    let target_index = self.route_index(target);
    match (from_index, target_index) {
      (Some(from_index), Some(target_index)) if from_index < target_index => {
        self.route[from_index + 1]
      }
      (Some(from_index), Some(target_index)) if from_index > target_index => {
        self.route[from_index - 1]
      }
      _ => from,
    }
  }
}

#[derive(Clone, Debug)]
pub struct SourceNode {
  pub stockpile: Inventory,
  pub item: ItemId,
  pub mining: MiningExtractor,
  pub dispatch: DispatchBoard,
}

impl SourceNode {
  pub fn new(stockpile: Inventory, item: ItemId, mining: MiningExtractor) -> Self {
    Self {
      stockpile,
      item,
      mining,
      dispatch: DispatchBoard::new(),
    }
  }

  pub fn refresh_dispatch(&mut self, factory_location: NodeId) {
    self.dispatch.intent = (self.stockpile.count(self.item) > 0).then(|| {
      DispatchIntent::collect(self.item, NodeId::Source, factory_location)
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

  pub fn refresh_dispatch(&mut self, source_location: NodeId) {
    let needed = self
      .input_buffer
      .saturating_sub(self.production.inventory.count(self.production.recipe.input_item));
    self.dispatch.intent = (needed > 0).then(|| {
      DispatchIntent::deliver(self.production.recipe.input_item, source_location, NodeId::Factory)
    });
  }
}

#[derive(Clone, Debug)]
pub struct Hauler {
  pub cargo: Inventory,
  pub position: NodeId,
  pub target: NodeId,
  pub route_index: usize,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

impl Hauler {
  pub fn new(cargo: Inventory, position: NodeId, carry_limit: u32, target: NodeId) -> Self {
    Self {
      cargo,
      position,
      target,
      route_index: 0,
      carry_limit,
      dispatch: DispatchReceiverState::Unassigned,
    }
  }

  pub fn set_route_index(&mut self, topology: &Topology) {
    self.route_index = topology.route_index(self.position).unwrap_or(self.route_index);
  }

  pub fn set_target(&mut self, target: NodeId) {
    self.target = target;
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
  pub mining: MiningExtractor,
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
  pub position: NodeId,
  pub position_grid: GridPosition,
  pub target: NodeId,
  pub target_grid: GridPosition,
  pub route_index: usize,
  pub cargo: crate::resources::InventorySnapshot,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologySnapshot {
  pub nodes: [TopologyNode; 3],
  pub route: [NodeId; 3],
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
  pub topology: Topology,
}
