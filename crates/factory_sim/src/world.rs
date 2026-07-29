use crate::dispatch::{DispatchAssignment, DispatchBoard, DispatchIntent, DispatchReceiverState};
use crate::mining::MiningExtractor;
use crate::power::{PowerPlant, PowerSnapshot};
use crate::production::{CraftSnapshot, FactoryProduction};
use crate::resources::Inventory;
use factory_content::{ItemId, LayoutSpec, ScenarioDefinition};
use serde::{Serialize, Serializer};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeId {
  Source(u8),
  Road,
  Factory,
  PowerPlant,
  Transit(GridPosition),
}

impl fmt::Display for NodeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Source(index) => write!(f, "source-{index}"),
      Self::Road => f.write_str("road"),
      Self::Factory => f.write_str("factory"),
      Self::PowerPlant => f.write_str("power-plant"),
      Self::Transit(position) => write!(f, "transit-{}-{}", position.x, position.y),
    }
  }
}

impl Serialize for NodeId {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(self)
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GridPosition {
  pub x: i32,
  pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologyNode {
  pub id: NodeId,
  pub position: GridPosition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Topology {
  pub width: i32,
  pub height: i32,
  pub nodes: Vec<TopologyNode>,
  pub blocked: BTreeSet<GridPosition>,
  pub obstacles: BTreeSet<GridPosition>,
}

impl Topology {
  pub fn for_sources(source_count: u8, include_power_plant: bool) -> Self {
    Self::from_layout(&LayoutSpec::linear(source_count, include_power_plant))
  }

  pub fn from_layout(layout: &LayoutSpec) -> Self {
    let mut nodes = Vec::with_capacity(layout.source_positions.len() + 3);
    for (index, position) in layout.source_positions.iter().enumerate() {
      nodes.push(TopologyNode {
        id: NodeId::Source(index as u8),
        position: GridPosition {
          x: position.x,
          y: position.y,
        },
      });
    }
    nodes.push(TopologyNode {
      id: NodeId::Road,
      position: GridPosition {
        x: layout.road_position.x,
        y: layout.road_position.y,
      },
    });
    nodes.push(TopologyNode {
      id: NodeId::Factory,
      position: GridPosition {
        x: layout.factory_position.x,
        y: layout.factory_position.y,
      },
    });
    if let Some(position) = layout.power_plant_position {
      nodes.push(TopologyNode {
        id: NodeId::PowerPlant,
        position: GridPosition {
          x: position.x,
          y: position.y,
        },
      });
    }
    let obstacles = layout
      .obstacles
      .iter()
      .map(|position| GridPosition {
        x: position.x,
        y: position.y,
      })
      .collect::<BTreeSet<_>>();
    let mut blocked = nodes
      .iter()
      .filter(|node| node.id != NodeId::Road)
      .map(|node| node.position)
      .collect::<BTreeSet<_>>();
    blocked.extend(obstacles.iter().copied());
    Self {
      width: layout.width,
      height: layout.height,
      nodes,
      blocked,
      obstacles,
    }
  }

  pub fn with_obstacles(
    mut self,
    obstacles: impl IntoIterator<Item = GridPosition>,
  ) -> Self {
    let obstacles = obstacles.into_iter().collect::<Vec<_>>();
    self.blocked.extend(obstacles.iter().copied());
    self.obstacles.extend(obstacles);
    self
  }

  pub fn position(&self, node: NodeId) -> GridPosition {
    if let NodeId::Transit(position) = node {
      return position;
    }
    self
      .nodes
      .iter()
      .find(|candidate| candidate.id == node)
      .map(|node| node.position)
      .expect("topology contains requested node")
  }

  fn in_bounds(&self, position: GridPosition) -> bool {
    position.x >= 0
      && position.y >= 0
      && position.x < self.width
      && position.y < self.height
  }

  fn walkable(&self, position: GridPosition, start: GridPosition, end: GridPosition) -> bool {
    self.in_bounds(position)
      && (position == start || position == end || !self.blocked.contains(&position))
  }

  fn neighbors(
    &self,
    position: GridPosition,
    start: GridPosition,
    end: GridPosition,
  ) -> Vec<(GridPosition, u32)> {
    const CARDINAL: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    const DIAGONAL: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut neighbors = Vec::with_capacity(8);
    for (x, y) in CARDINAL {
      let candidate = GridPosition {
        x: position.x + x,
        y: position.y + y,
      };
      if self.walkable(candidate, start, end) {
        neighbors.push((candidate, 10));
      }
    }
    for (x, y) in DIAGONAL {
      let candidate = GridPosition {
        x: position.x + x,
        y: position.y + y,
      };
      let horizontal = GridPosition {
        x: position.x + x,
        y: position.y,
      };
      let vertical = GridPosition {
        x: position.x,
        y: position.y + y,
      };
      if self.walkable(candidate, start, end)
        && (self.walkable(horizontal, start, end) || self.walkable(vertical, start, end))
      {
        neighbors.push((candidate, 14));
      }
    }
    neighbors
  }

  fn heuristic(from: GridPosition, to: GridPosition) -> u32 {
    let x = from.x.abs_diff(to.x);
    let y = from.y.abs_diff(to.y);
    10 * x.max(y) + 4 * x.min(y)
  }

  pub fn path(&self, from: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
    let start = self.position(from);
    let end = self.position(target);
    let mut open = vec![start];
    let mut came_from = BTreeMap::new();
    let mut cost = BTreeMap::from([(start, 0_u32)]);

    while !open.is_empty() {
      let current_index = open
        .iter()
        .enumerate()
        .min_by_key(|(_, position)| {
          (
            cost.get(position).copied().unwrap_or(u32::MAX)
              + Self::heuristic(**position, end),
            Self::heuristic(**position, end),
            position.y,
            position.x,
          )
        })
        .map(|(index, _)| index)
        .expect("open path set is not empty");
      let current = open.swap_remove(current_index);
      if current == end {
        let mut path = vec![end];
        let mut cursor = end;
        while let Some(previous) = came_from.get(&cursor).copied() {
          path.push(previous);
          cursor = previous;
        }
        path.reverse();
        return Some(
          path
            .into_iter()
            .map(|position| {
              if position == start {
                from
              } else if position == end {
                target
              } else {
                self
                  .nodes
                  .iter()
                  .find(|node| node.position == position && node.id == NodeId::Road)
                  .map_or(NodeId::Transit(position), |node| node.id)
              }
            })
            .collect(),
        );
      }

      let current_cost = cost[&current];
      for (neighbor, step_cost) in self.neighbors(current, start, end) {
        let candidate_cost = current_cost + step_cost;
        if cost
          .get(&neighbor)
          .is_some_and(|known_cost| *known_cost <= candidate_cost)
        {
          continue;
        }
        came_from.insert(neighbor, current);
        cost.insert(neighbor, candidate_cost);
        if !open.contains(&neighbor) {
          open.push(neighbor);
        }
      }
    }
    None
  }

  pub fn step_toward(&self, from: NodeId, target: NodeId) -> Option<NodeId> {
    self
      .path(from, target)
      .and_then(|path| path.get(1).copied().or(Some(from)))
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
  pub width: i32,
  pub height: i32,
  pub nodes: Vec<TopologyNode>,
  pub blocked: BTreeSet<GridPosition>,
  pub obstacles: BTreeSet<GridPosition>,
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
  MoveHauler {
    hauler_id: u8,
    from: NodeId,
    to: NodeId,
    target: NodeId,
  },
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

#[cfg(test)]
mod tests {
  use super::*;
  use factory_content::{GridPoint, LayoutSpec};

  #[test]
  fn astar_uses_the_open_road_for_the_linear_layout() {
    let topology = Topology::for_sources(1, false);
    assert_eq!(
      Some(vec![NodeId::Source(0), NodeId::Road, NodeId::Factory]),
      topology.path(NodeId::Source(0), NodeId::Factory)
    );
  }

  #[test]
  fn astar_routes_around_an_occupied_cell_deterministically() {
    let topology = Topology::from_layout(&LayoutSpec {
      width: 4,
      height: 3,
      source_positions: vec![GridPoint { x: 0, y: 1 }],
      road_position: GridPoint { x: 1, y: 0 },
      factory_position: GridPoint { x: 3, y: 1 },
      power_plant_position: None,
      obstacles: vec![GridPoint { x: 1, y: 1 }],
    });
    assert_eq!(
      Some(vec![
        NodeId::Source(0),
        NodeId::Road,
        NodeId::Transit(GridPosition { x: 2, y: 1 }),
        NodeId::Factory,
      ]),
      topology.path(NodeId::Source(0), NodeId::Factory)
    );
  }

  #[test]
  fn astar_reports_when_occupancy_seals_the_target() {
    let topology = Topology::for_sources(1, false).with_obstacles([
      GridPosition { x: 1, y: 0 },
      GridPosition { x: 0, y: 1 },
      GridPosition { x: 1, y: 1 },
    ]);
    assert_eq!(None, topology.path(NodeId::Source(0), NodeId::Factory));
  }
}
