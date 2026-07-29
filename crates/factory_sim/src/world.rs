use crate::dispatch::{DispatchAssignment, DispatchBoard, DispatchIntent, DispatchReceiverState};
use crate::mining::MiningExtractor;
use crate::power::{PowerPlant, PowerSnapshot};
use crate::production::{CraftSnapshot, FactoryProduction};
use crate::resources::Inventory;
use factory_content::{ItemId, ScenarioDefinition};
use serde::{Serialize, Serializer};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeId {
  Source(u8),
  Road,
  Factory,
  PowerPlant,
}

impl fmt::Display for NodeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Source(index) => write!(f, "source-{index}"),
      Self::Road => f.write_str("road"),
      Self::Factory => f.write_str("factory"),
      Self::PowerPlant => f.write_str("power-plant"),
    }
  }
}

impl Serialize for NodeId {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(self)
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

// Hub topology: every source and the factory hang off the single road
// node, so any trip is at most two hops and routing needs no search.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Topology {
  pub nodes: Vec<TopologyNode>,
}

impl Topology {
  pub fn for_sources(source_count: u8, include_power_plant: bool) -> Self {
    let mut nodes = Vec::with_capacity(usize::from(source_count) + 3);
    for index in 0..source_count {
      nodes.push(TopologyNode {
        id: NodeId::Source(index),
        position: GridPosition {
          x: 0,
          y: i32::from(index),
        },
      });
    }
    nodes.push(TopologyNode {
      id: NodeId::Road,
      position: GridPosition { x: 1, y: 0 },
    });
    nodes.push(TopologyNode {
      id: NodeId::Factory,
      position: GridPosition { x: 2, y: 0 },
    });
    if include_power_plant {
      nodes.push(TopologyNode {
        id: NodeId::PowerPlant,
        position: GridPosition { x: 2, y: 1 },
      });
    }
    Self { nodes }
  }

  pub fn position(&self, node: NodeId) -> GridPosition {
    self
      .nodes
      .iter()
      .find(|candidate| candidate.id == node)
      .map(|node| node.position)
      .expect("topology contains requested node")
  }

  pub fn step_toward(&self, from: NodeId, target: NodeId) -> NodeId {
    if from == target {
      from
    } else if from == NodeId::Road {
      target
    } else {
      NodeId::Road
    }
  }
}

#[derive(Clone, Debug)]
pub struct SourceNode {
  pub node: NodeId,
  pub stockpile: Inventory,
  pub item: ItemId,
  pub mining: MiningExtractor,
  pub dispatch: DispatchBoard,
  pub deployed: bool,
}

impl SourceNode {
  pub fn new(
    node: NodeId,
    stockpile: Inventory,
    item: ItemId,
    mining: MiningExtractor,
    deployed: bool,
  ) -> Self {
    Self {
      node,
      stockpile,
      item,
      mining,
      dispatch: DispatchBoard::new(),
      deployed,
    }
  }

  pub fn refresh_dispatch(&mut self, factory_location: NodeId) {
    self.dispatch.intents = (self.deployed && self.stockpile.count(self.item) > 0)
      .then(|| DispatchIntent::collect(self.item, self.node, factory_location))
      .into_iter()
      .collect();
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

  pub fn refresh_dispatch(&mut self) {
    let input_buffer = self.input_buffer;
    let inventory = &self.production.inventory;
    self.dispatch.intents = self
      .production
      .recipe
      .inputs
      .keys()
      .filter(|item| input_buffer.saturating_sub(inventory.count(**item)) > 0)
      .map(|item| DispatchIntent::deliver(*item, NodeId::Road, NodeId::Factory))
      .collect();
  }
}

#[derive(Clone, Debug)]
pub struct Hauler {
  pub id: u8,
  pub cargo: Inventory,
  pub position: NodeId,
  pub target: NodeId,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

impl Hauler {
  pub fn new(id: u8, cargo: Inventory, position: NodeId, carry_limit: u32) -> Self {
    Self {
      id,
      cargo,
      position,
      target: position,
      carry_limit,
      dispatch: DispatchReceiverState::Unassigned,
    }
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
  pub node: NodeId,
  pub item: ItemId,
  pub stockpile: crate::resources::InventorySnapshot,
  pub mining: MiningExtractor,
  pub dispatch: DispatchBoard,
  pub deployed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactorySnapshot {
  pub inventory: crate::resources::InventorySnapshot,
  pub craft: CraftSnapshot,
  pub dispatch: DispatchBoard,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HaulerSnapshot {
  pub id: u8,
  pub position: NodeId,
  pub position_grid: GridPosition,
  pub target: NodeId,
  pub target_grid: GridPosition,
  pub cargo: crate::resources::InventorySnapshot,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologySnapshot {
  pub nodes: Vec<TopologyNode>,
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
  pub sources: Vec<SourceSnapshot>,
  pub haulers: Vec<HaulerSnapshot>,
  pub factory: FactorySnapshot,
  pub power: Option<PowerSnapshot>,
  pub events: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldMutation {
  DeploySource(u8),
}

#[derive(Clone, Debug)]
pub struct WorldState {
  pub tick: u64,
  pub scenario: ScenarioDefinition,
  pub sources: Vec<SourceNode>,
  pub haulers: Vec<Hauler>,
  pub factory: FactoryNode,
  pub power: Option<PowerPlant>,
  pub topology: Topology,
  pub queued_mutations: Vec<WorldMutation>,
}
