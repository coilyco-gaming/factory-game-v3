use crate::alerts::AlertHistory;
use crate::dispatch::{DispatchAssignment, DispatchBoard, DispatchIntent, DispatchReceiverState};
use crate::mining::MiningExtractor;
use crate::power::{Battery, BatteryOwner, PowerGrid, PowerSnapshot};
use crate::production::{CraftSnapshot, FactoryProduction};
use crate::radar::{DeploymentRadar, RadarSnapshot};
use crate::resources::Inventory;
use factory_content::{ContentDatabase, ItemId, LayoutSpec, ScenarioDefinition};
use serde::{Serialize, Serializer};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use std::fmt;
use std::sync::RwLock;

pub type NodeIndex = u16;
pub type HaulerId = u16;
const ROUTE_CACHE_CAPACITY: usize = 4_096;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeId {
  Source(NodeIndex),
  Road,
  Factory(NodeIndex),
  Generator(NodeIndex),
  Radar(NodeIndex),
  BuildSite(NodeIndex),
  Structure(NodeIndex),
  Transit(GridPosition),
}

impl fmt::Display for NodeId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Source(index) => write!(f, "source-{index}"),
      Self::Road => f.write_str("road"),
      Self::Factory(index) => write!(f, "factory-{index}"),
      Self::Generator(index) => write!(f, "generator-{index}"),
      Self::Radar(index) => write!(f, "radar-{index}"),
      Self::BuildSite(index) => write!(f, "build-site-{index}"),
      Self::Structure(index) => write!(f, "structure-{index}"),
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

#[derive(Debug, Serialize)]
pub struct Topology {
  pub width: i32,
  pub height: i32,
  pub nodes: Vec<TopologyNode>,
  pub blocked: BTreeSet<GridPosition>,
  pub obstacles: BTreeSet<GridPosition>,
  #[serde(skip)]
  positions: BTreeMap<NodeId, GridPosition>,
  #[serde(skip)]
  route_cache: RwLock<BTreeMap<(NodeId, NodeId), Option<Vec<NodeId>>>>,
}

impl Clone for Topology {
  fn clone(&self) -> Self {
    Self {
      width: self.width,
      height: self.height,
      nodes: self.nodes.clone(),
      blocked: self.blocked.clone(),
      obstacles: self.obstacles.clone(),
      positions: self.positions.clone(),
      route_cache: RwLock::new(BTreeMap::new()),
    }
  }
}

impl PartialEq for Topology {
  fn eq(&self, other: &Self) -> bool {
    self.width == other.width
      && self.height == other.height
      && self.nodes == other.nodes
      && self.blocked == other.blocked
      && self.obstacles == other.obstacles
      && self.positions == other.positions
  }
}

impl Eq for Topology {}

impl Topology {
  pub fn for_sources(source_count: NodeIndex, include_generator: bool) -> Self {
    Self::from_layout(&LayoutSpec::linear(source_count, include_generator))
  }

  pub fn from_layout(layout: &LayoutSpec) -> Self {
    let mut nodes =
      Vec::with_capacity(layout.source_positions.len() + layout.factory_positions.len() + 2);
    for (index, position) in layout.source_positions.iter().enumerate() {
      nodes.push(TopologyNode {
        id: NodeId::Source(node_index(index)),
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
    for (index, position) in layout.factory_positions.iter().enumerate() {
      nodes.push(TopologyNode {
        id: NodeId::Factory(node_index(index)),
        position: GridPosition {
          x: position.x,
          y: position.y,
        },
      });
    }
    for (index, position) in layout.generator_positions.iter().enumerate() {
      nodes.push(TopologyNode {
        id: NodeId::Generator(node_index(index)),
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
    // Position lookup sits on several 100x100 hot paths. Keep the ordered node
    // vector for stable snapshots while avoiding a full scan for every query.
    let positions = nodes.iter().map(|node| (node.id, node.position)).collect();
    Self {
      width: layout.width,
      height: layout.height,
      nodes,
      blocked,
      obstacles,
      positions,
      route_cache: RwLock::new(BTreeMap::new()),
    }
  }

  pub fn from_scenario(scenario: &ScenarioDefinition) -> Self {
    let mut topology = Self::from_layout(&scenario.layout);
    let scenario_nodes = scenario
      .radars
      .iter()
      .enumerate()
      .map(|(index, radar)| TopologyNode {
        id: NodeId::Radar(node_index(index)),
        position: GridPosition {
          x: radar.position.x,
          y: radar.position.y,
        },
      })
      .chain(
        scenario
          .build_sites
          .iter()
          .enumerate()
          .map(|(index, site)| TopologyNode {
            id: NodeId::BuildSite(node_index(index)),
            position: GridPosition {
              x: site.position.x,
              y: site.position.y,
            },
          }),
      )
      .collect::<Vec<_>>();
    topology
      .positions
      .extend(scenario_nodes.iter().map(|node| (node.id, node.position)));
    topology.nodes.extend(scenario_nodes);
    topology
  }

  pub fn with_obstacles(mut self, obstacles: impl IntoIterator<Item = GridPosition>) -> Self {
    let obstacles = obstacles.into_iter().collect::<Vec<_>>();
    self.blocked.extend(obstacles.iter().copied());
    self.obstacles.extend(obstacles);
    self
      .route_cache
      .get_mut()
      .expect("route cache is not poisoned")
      .clear();
    self
  }

  pub fn position(&self, node: NodeId) -> GridPosition {
    if let NodeId::Transit(position) = node {
      return position;
    }
    self
      .positions
      .get(&node)
      .copied()
      .expect("topology contains requested node")
  }

  pub(crate) fn replace_node_id(
    &mut self,
    current: NodeId,
    replacement: NodeId,
  ) -> Option<GridPosition> {
    let node = self.nodes.iter_mut().find(|node| node.id == current)?;
    let position = node.position;
    node.id = replacement;
    self.positions.remove(&current);
    self.positions.insert(replacement, position);
    self
      .route_cache
      .get_mut()
      .expect("route cache is not poisoned")
      .clear();
    Some(position)
  }

  pub(crate) fn insert_node(&mut self, id: NodeId, position: GridPosition) -> bool {
    if self.positions.contains_key(&id) {
      return false;
    }
    self.positions.insert(id, position);
    self.nodes.push(TopologyNode { id, position });
    self
      .route_cache
      .get_mut()
      .expect("route cache is not poisoned")
      .clear();
    true
  }

  pub(crate) fn block(&mut self, position: GridPosition) {
    if self.blocked.insert(position) {
      self
        .route_cache
        .get_mut()
        .expect("route cache is not poisoned")
        .clear();
    }
  }

  pub(crate) fn unblock(&mut self, position: GridPosition) {
    if self.blocked.remove(&position) {
      self
        .route_cache
        .get_mut()
        .expect("route cache is not poisoned")
        .clear();
    }
  }

  pub fn has_transfer_access(&self, node: NodeId) -> bool {
    let target = self.position(node);
    (-1..=1).any(|y| {
      (-1..=1).any(|x| {
        if x == 0 && y == 0 {
          return false;
        }
        let position = GridPosition {
          x: target.x + x,
          y: target.y + y,
        };
        self.in_bounds(position) && !self.blocked.contains(&position)
      })
    })
  }

  fn in_bounds(&self, position: GridPosition) -> bool {
    position.x >= 0 && position.y >= 0 && position.x < self.width && position.y < self.height
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
    if let Some(path) = self
      .route_cache
      .read()
      .expect("route cache is not poisoned")
      .get(&(from, target))
    {
      return path.clone();
    }
    let path = self.compute_path(from, target);
    let key = (from, target);
    let mut cache = self
      .route_cache
      .write()
      .expect("route cache is not poisoned");
    if cache.len() >= ROUTE_CACHE_CAPACITY && !cache.contains_key(&key) {
      let evicted_key = cache
        .keys()
        .next()
        .copied()
        .expect("a full route cache contains an entry");
      cache.remove(&evicted_key);
    }
    cache.insert(key, path.clone());
    path
  }

  fn compute_path(&self, from: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
    let start = self.position(from);
    let end = self.position(target);
    let start_heuristic = Self::heuristic(start, end);
    let mut open = BinaryHeap::from([Reverse((
      start_heuristic,
      start_heuristic,
      start.y,
      start.x,
      start,
    ))]);
    let mut came_from = BTreeMap::new();
    let mut cost = BTreeMap::from([(start, 0_u32)]);

    while let Some(Reverse((estimated_cost, heuristic, _, _, current))) = open.pop() {
      if cost[&current] + heuristic != estimated_cost {
        continue;
      }
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
        let heuristic = Self::heuristic(neighbor, end);
        open.push(Reverse((
          candidate_cost + heuristic,
          heuristic,
          neighbor.y,
          neighbor.x,
          neighbor,
        )));
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
  pub occupied_by: Option<NodeId>,
  pub exhausted: bool,
  pub alerts: AlertHistory,
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
      occupied_by: None,
      exhausted: false,
      alerts: AlertHistory::default(),
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
  pub node: NodeId,
  pub production: FactoryProduction,
  pub dispatch: DispatchBoard,
  pub input_buffer: u32,
  pub alerts: AlertHistory,
}

impl FactoryNode {
  pub fn new(node: NodeId, production: FactoryProduction, input_buffer: u32) -> Self {
    Self {
      node,
      production,
      dispatch: DispatchBoard::new(),
      input_buffer,
      alerts: AlertHistory::default(),
    }
  }

  pub fn refresh_dispatch(&mut self, content: &ContentDatabase) {
    let input_buffer = self.input_buffer;
    let inventory = &self.production.inventory;
    self.dispatch.intents = self
      .production
      .recipe
      .inputs
      .keys()
      .filter(|item| input_buffer.saturating_sub(inventory.count(**item)) > 0)
      .map(|item| DispatchIntent::deliver(*item, NodeId::Road, self.node))
      .collect();
    let output_item = self.production.recipe.output_item;
    if self.production.inventory.count(output_item) > 0
      && !content.item(output_item).can_spawn_game_object
    {
      self.dispatch.intents.push(DispatchIntent::collect(
        output_item,
        self.node,
        NodeId::Road,
      ));
    }
  }
}

#[derive(Clone, Debug)]
pub struct Hauler {
  pub id: HaulerId,
  pub cargo: Inventory,
  pub position: NodeId,
  pub target: NodeId,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
  pub alerts: AlertHistory,
  pub route: VecDeque<NodeId>,
}

impl Hauler {
  pub fn new(id: HaulerId, cargo: Inventory, position: NodeId, carry_limit: u32) -> Self {
    Self {
      id,
      cargo,
      position,
      target: position,
      carry_limit,
      dispatch: DispatchReceiverState::Unassigned,
      alerts: AlertHistory::default(),
      route: VecDeque::new(),
    }
  }

  pub fn set_target(&mut self, target: NodeId) {
    if self.target != target {
      self.route.clear();
    }
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub occupied_by: Option<NodeId>,
  pub exhausted: bool,
  pub alerts: AlertHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactorySnapshot {
  pub node: NodeId,
  pub inventory: crate::resources::InventorySnapshot,
  pub craft: CraftSnapshot,
  pub dispatch: DispatchBoard,
  pub alerts: AlertHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StructureSnapshot {
  pub node: NodeId,
  pub item: ItemId,
  pub alerts: AlertHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HaulerSnapshot {
  pub id: HaulerId,
  pub position: NodeId,
  pub position_grid: GridPosition,
  pub target: NodeId,
  pub target_grid: GridPosition,
  pub cargo: crate::resources::InventorySnapshot,
  pub carry_limit: u32,
  pub dispatch: DispatchReceiverState,
  pub alerts: AlertHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TopologySnapshot {
  pub width: i32,
  pub height: i32,
  pub nodes: Vec<TopologyNode>,
  pub blocked: BTreeSet<GridPosition>,
  pub obstacles: BTreeSet<GridPosition>,
  pub power_lines: BTreeSet<GridPosition>,
  pub generator_power_lines: Vec<GeneratorPowerLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratorPowerLine {
  pub generator: NodeId,
  pub target: NodeId,
  pub cells: Vec<GridPosition>,
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
  pub factories: Vec<FactorySnapshot>,
  pub radars: Vec<RadarSnapshot>,
  pub structures: Vec<StructureSnapshot>,
  pub power: Option<PowerSnapshot>,
  pub events: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LivenessSummary {
  pub tick: u64,
  pub deployed_sources: usize,
  pub occupied_sources: usize,
  pub exhausted_sources: usize,
  pub generators: usize,
  pub claimed_radars: usize,
  pub dispatch_intents: usize,
  pub assigned_haulers: usize,
  pub routed_haulers: usize,
  pub max_route_len: usize,
  pub queued_mutations: usize,
  pub power_links: usize,
  pub power_line_cells: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldMutation {
  DeploySource(NodeIndex),
  SpawnGenerator {
    source_index: NodeIndex,
    item: ItemId,
    hauler_id: HaulerId,
  },
  DeleteDepletedDeposit(NodeIndex),
  TeardownSource(NodeIndex),
  SpawnStructure {
    site_index: NodeIndex,
    item: ItemId,
    hauler_id: HaulerId,
  },
  MoveHauler {
    hauler_id: HaulerId,
    from: NodeId,
    to: NodeId,
    target: NodeId,
  },
}

fn node_index(index: usize) -> NodeIndex {
  NodeIndex::try_from(index).expect("scenario object index fits NodeIndex")
}

#[derive(Clone, Debug)]
pub struct WorldState {
  pub tick: u64,
  pub scenario: ScenarioDefinition,
  pub sources: Vec<SourceNode>,
  pub haulers: Vec<Hauler>,
  pub factories: Vec<FactoryNode>,
  pub radars: Vec<DeploymentRadar>,
  pub structures: Vec<StructureSnapshot>,
  pub power: Option<PowerGrid>,
  pub batteries: BTreeMap<BatteryOwner, Battery>,
  pub power_lines: BTreeSet<GridPosition>,
  pub generator_power_lines: Vec<GeneratorPowerLine>,
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
      Some(vec![NodeId::Source(0), NodeId::Road, NodeId::Factory(0)]),
      topology.path(NodeId::Source(0), NodeId::Factory(0))
    );
  }

  #[test]
  fn astar_routes_around_an_occupied_cell_deterministically() {
    let topology = Topology::from_layout(&LayoutSpec {
      width: 4,
      height: 3,
      source_positions: vec![GridPoint { x: 0, y: 1 }],
      road_position: GridPoint { x: 1, y: 0 },
      factory_positions: vec![GridPoint { x: 3, y: 1 }],
      generator_positions: Vec::new(),
      hauler_positions: Vec::new(),
      obstacles: vec![GridPoint { x: 1, y: 1 }],
    });
    assert_eq!(
      Some(vec![
        NodeId::Source(0),
        NodeId::Road,
        NodeId::Transit(GridPosition { x: 2, y: 1 }),
        NodeId::Factory(0),
      ]),
      topology.path(NodeId::Source(0), NodeId::Factory(0))
    );
  }

  #[test]
  fn astar_reports_when_occupancy_seals_the_target() {
    let topology = Topology::for_sources(1, false).with_obstacles([
      GridPosition { x: 1, y: 0 },
      GridPosition { x: 0, y: 1 },
      GridPosition { x: 1, y: 1 },
    ]);
    assert_eq!(None, topology.path(NodeId::Source(0), NodeId::Factory(0)));
    assert!(!topology.has_transfer_access(NodeId::Source(0)));
  }

  #[test]
  fn topology_mutations_invalidate_cached_routes() {
    let mut topology = Topology::for_sources(1, false);
    let route = topology.path(NodeId::Source(0), NodeId::Factory(0));

    assert!(route.is_some());
    assert_eq!(1, topology.route_cache.read().unwrap().len());

    let road = topology.position(NodeId::Road);
    topology.block(road);
    assert!(topology.route_cache.read().unwrap().is_empty());

    topology.path(NodeId::Source(0), NodeId::Factory(0));
    assert_eq!(1, topology.route_cache.read().unwrap().len());

    topology.unblock(road);
    assert!(topology.route_cache.read().unwrap().is_empty());
  }

  #[test]
  fn route_cache_evicts_deterministically_at_its_owning_capacity() {
    let topology = Topology::for_sources(1, false);
    {
      let mut cache = topology.route_cache.write().unwrap();
      for x in 0..ROUTE_CACHE_CAPACITY {
        cache.insert(
          (
            NodeId::Transit(GridPosition {
              x: i32::try_from(x).unwrap(),
              y: 0,
            }),
            NodeId::Road,
          ),
          None,
        );
      }
    }

    let requested = (NodeId::Source(0), NodeId::Factory(0));
    assert!(topology.path(requested.0, requested.1).is_some());

    let cache = topology.route_cache.read().unwrap();
    assert_eq!(ROUTE_CACHE_CAPACITY, cache.len());
    assert!(cache.contains_key(&requested));
  }
}
