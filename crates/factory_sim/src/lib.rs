mod alerts;
mod dispatch;
mod metrics;
mod mining;
mod power;
mod production;
mod resources;
mod world;

use factory_content::{ContentDatabase, ItemId, ScenarioId, IRON_BARS_SCENARIO, MINING_DRILL};
use std::fmt;

pub use alerts::{AlertEntry, AlertHistory, MAX_OBJECT_ALERTS};
pub use dispatch::{
  DispatchAssignment, DispatchBoard, DispatchIntent, DispatchPhase, DispatchPolicy,
  DispatchPriority, DispatchReceiverState, DispatchVerb,
};
pub use metrics::{RunMetrics, RunMetricsSnapshot};
pub use mining::{Deposit, MiningExtractor};
pub use power::{Battery, BatteryOwner, PowerPlant, PowerSnapshot};
pub use production::{
  CraftSnapshot, FactoryProduction, ProductionBlockReason, RecipeRuntime,
};
pub use resources::{Inventory, InventoryError, InventorySnapshot};
pub use world::{
  FactoryNode, FactorySnapshot, GridPosition, Hauler, HaulerSnapshot, NodeId, ScenarioSnapshot,
  SourceNode, SourceSnapshot, StructureSnapshot, TickSnapshot, Topology, TopologyNode,
  TopologySnapshot, WorldMutation, WorldState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimulationError {
  UnknownScenario(ScenarioId),
  ScenarioMissingSources(ScenarioId),
  ScenarioMissingFactories(ScenarioId),
  ScenarioLayoutMismatch(ScenarioId),
  RecipeMissingIngredients(ItemId),
}

impl fmt::Display for SimulationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownScenario(id) => write!(f, "unknown scenario: {id}"),
      Self::ScenarioMissingSources(id) => {
        write!(f, "scenario {id} must define at least one source")
      }
      Self::ScenarioMissingFactories(id) => {
        write!(f, "scenario {id} must define at least one factory")
      }
      Self::ScenarioLayoutMismatch(id) => {
        write!(f, "scenario {id} layout does not match its world objects")
      }
      Self::RecipeMissingIngredients(id) => {
        write!(f, "recipe for {id} must have at least one ingredient")
      }
    }
  }
}

impl std::error::Error for SimulationError {}

pub struct GameState {
  pub world: WorldState,
  content: ContentDatabase,
  dispatch_policy: DispatchPolicy,
  metrics: RunMetrics,
}

#[derive(Copy, Clone)]
enum AdjacentProvider {
  Source(usize),
  Factory(usize),
}

impl GameState {
  pub fn new(content: ContentDatabase, scenario_id: ScenarioId) -> Result<Self, SimulationError> {
    let scenario = content
      .scenarios
      .get(&scenario_id)
      .cloned()
      .ok_or(SimulationError::UnknownScenario(scenario_id))?;
    if scenario.factories.is_empty() {
      return Err(SimulationError::ScenarioMissingFactories(scenario_id));
    }
    if scenario.sources.len() != scenario.layout.source_positions.len()
      || scenario.factories.len() != scenario.layout.factory_positions.len()
      || scenario.power.is_some() != scenario.layout.power_plant_position.is_some()
    {
      return Err(SimulationError::ScenarioLayoutMismatch(scenario_id));
    }
    let factories = scenario
      .factories
      .iter()
      .enumerate()
      .map(|(index, spec)| {
        let product = content.item(spec.product_item).clone();
        if product.ingredients.is_empty() && !product.create_from_nothing {
          return Err(SimulationError::RecipeMissingIngredients(product.id));
        }
        let recipe = RecipeRuntime {
          inputs: product.ingredients.clone(),
          output_item: product.id,
          output_quantity: product.craft_output,
          craft_time: product.craft_time.max(1),
        };
        let mut inventory = Inventory::new(1024, 1024);
        for input_item in recipe.inputs.keys() {
          inventory.reserve(*input_item, spec.input_buffer);
        }
        inventory.reserve(recipe.output_item, spec.output_buffer);
        for (item, quantity) in &spec.starting_items {
          inventory.reserve(*item, *quantity);
          inventory
            .insert_exact(&content, *item, *quantity)
            .expect("scenario starting inventory fits its factory");
        }
        Ok(FactoryNode::new(
          NodeId::Factory(index as u8),
          FactoryProduction::new(inventory, recipe, spec.output_buffer),
          spec.input_buffer,
        ))
      })
      .collect::<Result<Vec<_>, SimulationError>>()?;

    let sources: Vec<SourceNode> = scenario
      .sources
      .iter()
      .enumerate()
      .map(|(index, spec)| {
        SourceNode::new(
          NodeId::Source(index as u8),
          Inventory::new(1024, 1024),
          spec.item,
          MiningExtractor::for_item(&content, spec.item, spec.mining_speed, spec.deposit),
          !spec.requires_deployment,
        )
      })
      .collect();
    let haulers: Vec<Hauler> = (0..scenario.hauler_count)
      .map(|index| {
        Hauler::new(
          index as u8,
          Inventory::new(
            scenario.hauler_weight_capacity,
            scenario.hauler_volume_capacity,
          ),
          NodeId::Road,
          scenario.hauler_capacity,
        )
      })
      .collect();

    let power = scenario
      .power
      .clone()
      .map(|spec| PowerPlant::new(&content, spec, Inventory::new(64, 64)));
    let topology = Topology::from_scenario(&scenario);
    let mut batteries = std::collections::BTreeMap::new();
    if power.is_some() {
      for source in &sources {
        let owner = BatteryOwner::Node(source.node);
        batteries.insert(owner, Battery::new(owner, 0, 100));
      }
      for factory in &factories {
        let owner = BatteryOwner::Node(factory.node);
        batteries.insert(owner, Battery::new(owner, 0, 1_000));
      }
      for hauler in &haulers {
        let owner = BatteryOwner::Hauler(hauler.id);
        batteries.insert(owner, Battery::new(owner, 0, 250));
      }
      let owner = BatteryOwner::Node(NodeId::PowerPlant);
      let capacity = power
        .as_ref()
        .expect("powered scenario has a power plant")
        .spec
        .grid_capacity;
      batteries.insert(owner, Battery::new(owner, 0, capacity));
    }

    Ok(Self {
      world: WorldState {
        tick: 0,
        sources,
        haulers,
        factories,
        structures: Vec::new(),
        power,
        batteries,
        power_lines: std::collections::BTreeSet::new(),
        power_lines_built: false,
        topology,
        queued_mutations: Vec::new(),
        scenario,
      },
      content,
      dispatch_policy: DispatchPolicy::default(),
      metrics: RunMetrics::default(),
    })
  }

  pub fn starter_iron_bars() -> Self {
    Self::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).expect("starter scenario is valid")
  }

  pub fn dispatch_priority(&self, destination: NodeId, item: ItemId) -> DispatchPriority {
    self.dispatch_policy.priority(destination, item)
  }

  pub fn set_dispatch_priority(
    &mut self,
    destination: NodeId,
    item: ItemId,
    priority: DispatchPriority,
  ) -> Option<DispatchPriority> {
    self
      .dispatch_policy
      .set_priority(destination, item, priority)
  }

  pub fn clear_dispatch_priority(
    &mut self,
    destination: NodeId,
    item: ItemId,
  ) -> Option<DispatchPriority> {
    self.dispatch_policy.clear_priority(destination, item)
  }

  fn advance_mining(&mut self, events: &mut Vec<String>) {
    let mining_cost = self
      .world
      .power
      .as_ref()
      .map_or(0, |power| power.spec.mining_cost);
    for source_index in 0..self.world.sources.len() {
      if !self.world.sources[source_index].deployed {
        continue;
      }
      let node = self.world.sources[source_index].node;
      let consumer = format!("mining-{node}");
      if !self.consume_power(node, mining_cost, &consumer, events) {
        continue;
      }
      let source = &mut self.world.sources[source_index];
      let mined = source.mining.advance(&self.content, &mut source.stockpile);
      self.metrics.record_mined(source.mining.item, mined);
      if mined > 0 {
        match source.mining.deposit {
          Deposit::Finite(remaining) => events.push(format!(
            "mine {} +{} deposit {} at {}",
            source.mining.item, mined, remaining, source.node
          )),
          Deposit::Manifest => events.push(format!(
            "mine {} +{} manifest at {}",
            source.mining.item, mined, source.node
          )),
        }
      } else if source.mining.deposit == Deposit::Finite(0) {
        source.alerts.record(self.world.tick, "nothing to mine");
      }
    }
  }

  fn refresh_dispatch_intents(&mut self) {
    let power_fuel = self.world.power.as_ref().map(|power| power.spec.fuel_item);
    for source in &mut self.world.sources {
      let destination = if Some(source.item) == power_fuel {
        NodeId::PowerPlant
      } else {
        self
          .world
          .factories
          .iter()
          .find(|factory| factory.production.recipe.inputs.contains_key(&source.item))
          .map_or(NodeId::Factory(0), |factory| factory.node)
      };
      source.refresh_dispatch(destination);
    }
    for factory in &mut self.world.factories {
      factory.refresh_dispatch(&self.content);
      for intent in &mut factory.dispatch.intents {
        if intent.verb == DispatchVerb::Deliver {
          intent.priority = self.dispatch_policy.priority(intent.to, intent.item);
        }
      }
    }
    let mut claimed_targets = std::collections::BTreeSet::new();
    for factory_index in 0..self.world.factories.len() {
      let factory_node = self.world.factories[factory_index].node;
      if self.world.factories[factory_index]
        .production
        .inventory
        .count(MINING_DRILL)
        == 0
      {
        continue;
      }
      let target = self
        .world
        .sources
        .iter()
        .filter(|source| {
          !source.deployed && !source.exhausted && !claimed_targets.contains(&source.node)
        })
        .min_by_key(|source| self.dispatch_distance(factory_node, source.node))
        .map(|source| source.node);
      if let Some(target) = target {
        claimed_targets.insert(target);
        self.world.factories[factory_index]
          .dispatch
          .intents
          .push(DispatchIntent::retrieve(MINING_DRILL, factory_node, target));
      }
    }
    for (site_index, site) in self.world.scenario.build_sites.iter().enumerate() {
      if self
        .world
        .structures
        .iter()
        .any(|structure| structure.node == NodeId::Structure(site_index as u8))
      {
        continue;
      }
      if let Some(factory_index) = self
        .world
        .factories
        .iter()
        .enumerate()
        .filter(|(_, factory)| factory.production.inventory.count(site.item) > 0)
        .min_by_key(|(_, factory)| {
          self.dispatch_distance(factory.node, NodeId::BuildSite(site_index as u8))
        })
        .map(|(index, _)| index)
      {
        let factory = &mut self.world.factories[factory_index];
        factory.dispatch.intents.push(DispatchIntent::retrieve(
          site.item,
          factory.node,
          NodeId::BuildSite(site_index as u8),
        ));
      }
    }
    if let Some(power) = &mut self.world.power {
      power.refresh_dispatch();
      for intent in &mut power.dispatch.intents {
        intent.priority = self.dispatch_policy.priority(intent.to, intent.item);
      }
    }
  }

  // Priority, destination, and item order demand before haulers are chosen.
  // In-flight cargo still reduces need so demand is never double-served.
  fn assign_dispatch(&mut self, events: &mut Vec<String>) {
    let demands: Vec<(DispatchPriority, ItemId, NodeId, u32, u32)> = self
      .world
      .factories
      .iter()
      .flat_map(|factory| {
        factory
          .dispatch
          .intents
          .iter()
          .filter(|intent| intent.verb == DispatchVerb::Deliver)
          .map(|intent| {
            (
              intent.priority,
              intent.item,
              factory.node,
              factory.input_buffer,
              factory.production.inventory.count(intent.item),
            )
          })
      })
      .collect();
    let mut demands = demands;
    if let Some(power) = &self.world.power {
      demands.extend(power.dispatch.intents.iter().map(|intent| {
        (
          intent.priority,
          intent.item,
          NodeId::PowerPlant,
          power.spec.fuel_buffer,
          power.fuel.count(intent.item),
        )
      }));
    }
    demands.sort_by_key(|(priority, item, destination, _, _)| {
      (std::cmp::Reverse(*priority), *destination, *item)
    });
    for (priority, item, destination, buffer, stocked) in demands {
      self.assign_dispatch_for_demand(priority, item, destination, buffer, stocked, events);
    }
    self.assign_deployments(events);
  }

  fn assign_deployments(&mut self, events: &mut Vec<String>) {
    let intents = self
      .world
      .factories
      .iter()
      .flat_map(|factory| factory.dispatch.intents.iter())
      .filter(|intent| intent.verb == DispatchVerb::Retrieve)
      .cloned()
      .collect::<Vec<_>>();
    for intent in intents {
      if self.world.haulers.iter().any(|hauler| {
        matches!(
          &hauler.dispatch,
          DispatchReceiverState::Assigned(assignment)
            if assignment.item == intent.item && assignment.destination == intent.to
        )
      }) {
        continue;
      }
      let Some(hauler_index) = self.world.haulers.iter().position(|hauler| {
        matches!(hauler.dispatch, DispatchReceiverState::Unassigned) && hauler.cargo.is_empty()
      }) else {
        break;
      };
      let dispatch_cost = self
        .world
        .power
        .as_ref()
        .map_or(0, |power| power.spec.dispatch_cost);
      if !self.consume_power(intent.from, dispatch_cost, "dispatch-deploy", events) {
        break;
      }
      let hauler = &mut self.world.haulers[hauler_index];
      hauler.assign(DispatchAssignment::retrieve(
        intent.item,
        intent.from,
        intent.to,
      ));
      self.metrics.dispatches_assigned += 1;
      events.push(format!(
        "dispatch assigned retrieve {} {} -> {} to hauler-{}",
        intent.item, intent.from, intent.to, hauler.id
      ));
    }
  }

  fn assign_dispatch_for_demand(
    &mut self,
    priority: DispatchPriority,
    item: ItemId,
    destination: NodeId,
    buffer: u32,
    stocked: u32,
    events: &mut Vec<String>,
  ) {
    let mut need = buffer.saturating_sub(stocked);
    for hauler in &self.world.haulers {
      if let DispatchReceiverState::Assigned(assignment) = &hauler.dispatch {
        if assignment.item == item && assignment.destination == destination {
          let in_flight = match assignment.phase {
            DispatchPhase::Collect => hauler.carry_limit,
            DispatchPhase::Deliver => hauler.cargo.count(item),
            DispatchPhase::Retrieve | DispatchPhase::Deploy => 0,
          };
          need = need.saturating_sub(in_flight);
        }
      }
    }
    for hauler_index in 0..self.world.haulers.len() {
      if need == 0 {
        break;
      }
      let hauler = &self.world.haulers[hauler_index];
      if !matches!(hauler.dispatch, DispatchReceiverState::Unassigned) || !hauler.cargo.is_empty() {
        continue;
      }
      let source_node = self
        .world
        .sources
        .iter()
        .filter(|source| {
          source
            .dispatch
            .intents
            .iter()
            .any(|intent| intent.verb == DispatchVerb::Collect && intent.item == item)
        })
        .map(|source| source.node)
        .chain(self.world.factories.iter().filter_map(|factory| {
          factory
            .dispatch
            .intents
            .iter()
            .any(|intent| intent.verb == DispatchVerb::Collect && intent.item == item)
            .then_some(factory.node)
        }))
        .min_by_key(|source| self.dispatch_distance(*source, destination));
      let source_node = match source_node {
        Some(node) => node,
        None => break,
      };
      let dispatch_cost = self
        .world
        .power
        .as_ref()
        .map_or(0, |power| power.spec.dispatch_cost);
      if !self.consume_power(destination, dispatch_cost, "dispatch", events) {
        break;
      }
      let hauler = &mut self.world.haulers[hauler_index];
      hauler.assign(DispatchAssignment::collect_with_priority(
        item,
        source_node,
        destination,
        priority,
      ));
      self.metrics.dispatches_assigned += 1;
      need = need.saturating_sub(hauler.carry_limit);
      events.push(format!(
        "dispatch assigned collect {} {} -> {} to hauler-{}",
        item,
        source_node,
        destination,
        hauler.id
      ));
    }
  }

  fn dispatch_distance(&self, from: NodeId, to: NodeId) -> (u32, NodeId) {
    let from = self.world.topology.position(from);
    let to_position = self.world.topology.position(to);
    (
      from.x.abs_diff(to_position.x).pow(2) + from.y.abs_diff(to_position.y).pow(2),
      to,
    )
  }

  fn advance_receiver_phases(&mut self, events: &mut Vec<String>) {
    let tick = self.world.tick;
    for hauler in &mut self.world.haulers {
      let DispatchReceiverState::Assigned(assignment) = &hauler.dispatch else {
        continue;
      };
      let next_phase = match assignment.phase {
        DispatchPhase::Collect if hauler.cargo.count(assignment.item) > 0 => {
          Some(DispatchPhase::Deliver)
        }
        DispatchPhase::Retrieve if hauler.cargo.count(assignment.item) > 0 => {
          Some(DispatchPhase::Deploy)
        }
        DispatchPhase::Deliver | DispatchPhase::Deploy
          if hauler.cargo.count(assignment.item) == 0 =>
        {
          None
        }
        _ => continue,
      };
      if let Some(phase) = next_phase {
        let alert = match (assignment.phase, phase) {
          (DispatchPhase::Collect, DispatchPhase::Deliver) => "collect => deliver",
          (DispatchPhase::Retrieve, DispatchPhase::Deploy) => "retrieve => deploy",
          _ => "receiver phase changed",
        };
        let item = assignment.item;
        hauler.dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
          phase,
          ..assignment.clone()
        });
        hauler.alerts.record(tick, alert);
        events.push(format!(
          "receiver hauler-{} {} => {:?}",
          hauler.id, item, phase
        ));
      } else {
        let phase = assignment.phase;
        let alert = match phase {
          DispatchPhase::Deliver => "deliver => collect",
          DispatchPhase::Deploy => "deploy => retrieve",
          _ => "receiver => awaiting",
        };
        hauler.clear_assignment();
        hauler.alerts.record(tick, alert);
        events.push(format!("receiver hauler-{} {:?} => awaiting", hauler.id, phase));
      }
    }
  }

  fn collect(&mut self, events: &mut Vec<String>) {
    for hauler_index in 0..self.world.haulers.len() {
      let (assignment, position) = match &self.world.haulers[hauler_index].dispatch {
        DispatchReceiverState::Assigned(assignment) => (
          assignment.clone(),
          self.world.haulers[hauler_index].position,
        ),
        DispatchReceiverState::Unassigned => continue,
      };
      if assignment.phase != DispatchPhase::Collect
        || !self.nodes_in_transfer_range(position, assignment.source)
      {
        continue;
      }
      let carry_limit = self.world.haulers[hauler_index].carry_limit;
      let moved = match assignment.source {
        NodeId::Source(_) => {
          let Some(source) = self
            .world
            .sources
            .iter_mut()
            .find(|source| source.node == assignment.source)
          else {
            continue;
          };
          source.stockpile.transfer_up_to(
            &self.content,
            &mut self.world.haulers[hauler_index].cargo,
            assignment.item,
            carry_limit,
          )
        }
        NodeId::Factory(factory_index) => {
          let Some(factory) = self.world.factories.get_mut(factory_index as usize) else {
            continue;
          };
          factory.production.inventory.transfer_up_to(
            &self.content,
            &mut self.world.haulers[hauler_index].cargo,
            assignment.item,
            carry_limit,
          )
        }
        NodeId::Road
        | NodeId::PowerPlant
        | NodeId::BuildSite(_)
        | NodeId::Structure(_)
        | NodeId::Transit(_) => 0,
      };
      self.metrics.units_collected += moved;
      if moved > 0 {
        let hauler = &mut self.world.haulers[hauler_index];
        events.push(format!(
          "dispatch collect {} from {} to {} by hauler-{}",
          moved, assignment.source, assignment.destination, hauler.id
        ));
      }
    }
  }

  fn deliver(&mut self, events: &mut Vec<String>) {
    for hauler_index in 0..self.world.haulers.len() {
      let (assignment, position) = match &self.world.haulers[hauler_index].dispatch {
        DispatchReceiverState::Assigned(assignment) => (
          assignment.clone(),
          self.world.haulers[hauler_index].position,
        ),
        DispatchReceiverState::Unassigned => continue,
      };
      if assignment.phase != DispatchPhase::Deliver
        || !self.nodes_in_transfer_range(position, assignment.destination)
      {
        continue;
      }
      let carried = self.world.haulers[hauler_index]
        .cargo
        .count(assignment.item);
      let delivered = match assignment.destination {
        NodeId::Factory(factory_index) => {
          let Some(factory) = self.world.factories.get_mut(factory_index as usize) else {
            continue;
          };
          self.world.haulers[hauler_index].cargo.transfer_up_to(
            &self.content,
            &mut factory.production.inventory,
            assignment.item,
            carried,
          )
        }
        NodeId::PowerPlant => {
          let Some(power) = &mut self.world.power else {
            continue;
          };
          self.world.haulers[hauler_index].cargo.transfer_up_to(
            &self.content,
            &mut power.fuel,
            assignment.item,
            carried,
          )
        }
        NodeId::Source(_)
        | NodeId::Road
        | NodeId::BuildSite(_)
        | NodeId::Structure(_)
        | NodeId::Transit(_) => 0,
      };
      self.metrics.units_delivered += delivered;
      if delivered > 0 {
        events.push(format!(
          "dispatch deliver {} to {} by hauler-{}",
          delivered, assignment.destination, self.world.haulers[hauler_index].id
        ));
      }
    }
  }

  fn retrieve_and_deploy(&mut self, events: &mut Vec<String>) {
    for hauler_index in 0..self.world.haulers.len() {
      let (assignment, position) = match &self.world.haulers[hauler_index].dispatch {
        DispatchReceiverState::Assigned(assignment) => (
          assignment.clone(),
          self.world.haulers[hauler_index].position,
        ),
        DispatchReceiverState::Unassigned => continue,
      };
      match assignment.phase {
        DispatchPhase::Retrieve if self.nodes_in_transfer_range(position, assignment.source) => {
          let NodeId::Factory(factory_index) = assignment.source else {
            continue;
          };
          let Some(factory) = self.world.factories.get_mut(factory_index as usize) else {
            continue;
          };
          let moved = factory.production.inventory.transfer_up_to(
            &self.content,
            &mut self.world.haulers[hauler_index].cargo,
            assignment.item,
            1,
          );
          if moved > 0 {
            events.push(format!(
              "dispatch retrieve {} from factory by hauler-{}",
              moved, self.world.haulers[hauler_index].id
            ));
          }
        }
        DispatchPhase::Deploy
          if self.nodes_in_transfer_range(position, assignment.destination) =>
        {
          if self.world.haulers[hauler_index]
            .cargo
            .remove_exact(assignment.item, 1)
            .is_ok()
          {
            match assignment.destination {
              NodeId::Source(source_index) => self
                .world
                .queued_mutations
                .push(WorldMutation::DeploySource(source_index)),
              NodeId::BuildSite(site_index) => {
                self.world.queued_mutations.push(WorldMutation::SpawnStructure {
                  site_index,
                  item: assignment.item,
                  hauler_id: self.world.haulers[hauler_index].id,
                });
              }
              _ => continue,
            }
            events.push(format!(
              "dispatch deploy {} queued at {} by hauler-{}",
              assignment.item,
              assignment.destination,
              self.world.haulers[hauler_index].id
            ));
          }
        }
        _ => {}
      }
    }
  }

  fn apply_world_mutations(&mut self, events: &mut Vec<String>) {
    for mutation in std::mem::take(&mut self.world.queued_mutations) {
      match mutation {
        WorldMutation::DeploySource(source_index) => {
          let Some(source) = self
            .world
            .sources
            .iter_mut()
            .find(|source| source.node == NodeId::Source(source_index))
          else {
            continue;
          };
          source.deployed = true;
          self.metrics.deployments += 1;
          events.push(format!("world deploy mining drill at {}", source.node));
        }
        WorldMutation::DeleteDepletedDeposit(source_index) => {
          let Some(source) = self
            .world
            .sources
            .iter_mut()
            .find(|source| source.node == NodeId::Source(source_index))
          else {
            continue;
          };
          source.exhausted = true;
          self.metrics.world_deletions += 1;
          events.push(format!("world delete depleted ore at {}", source.node));
        }
        WorldMutation::TeardownSource(source_index) => {
          let Some(source) = self
            .world
            .sources
            .iter_mut()
            .find(|source| source.node == NodeId::Source(source_index))
          else {
            continue;
          };
          source.deployed = false;
          self
            .world
            .topology
            .blocked
            .remove(&self.world.topology.position(source.node));
          self.metrics.world_deletions += 1;
          events.push(format!("world delete depleted mining drill at {}", source.node));
        }
        WorldMutation::SpawnStructure {
          site_index,
          item,
          hauler_id,
        } => {
          let build_site = NodeId::BuildSite(site_index);
          let structure = NodeId::Structure(site_index);
          let Some(node) = self
            .world
            .topology
            .nodes
            .iter_mut()
            .find(|node| node.id == build_site)
          else {
            continue;
          };
          node.id = structure;
          self.world.topology.blocked.insert(node.position);
          self.world.structures.push(StructureSnapshot {
            node: structure,
            item,
            alerts: AlertHistory::default(),
          });
          if let Some(hauler) = self
            .world
            .haulers
            .iter_mut()
            .find(|hauler| hauler.id == hauler_id)
          {
            hauler.target = structure;
            hauler.clear_assignment();
          }
          self.metrics.deployments += 1;
          events.push(format!("world spawn {item} at {structure}"));
        }
        WorldMutation::MoveHauler {
          hauler_id,
          from,
          to,
          target,
        } => {
          let Some(hauler) = self
            .world
            .haulers
            .iter_mut()
            .find(|hauler| hauler.id == hauler_id && hauler.position == from)
          else {
            continue;
          };
          hauler.position = to;
          events.push(format!(
            "move hauler-{} {} -> {} toward {}",
            hauler.id, from, to, target
          ));
        }
      }
    }
  }

  fn queue_source_lifecycle(&mut self) {
    for (source_index, source) in self.world.sources.iter().enumerate() {
      if !source.exhausted && source.mining.is_depleted() {
        self
          .world
          .queued_mutations
          .push(WorldMutation::DeleteDepletedDeposit(source_index as u8));
      } else if source.exhausted && source.deployed && source.stockpile.is_empty() {
        self
          .world
          .queued_mutations
          .push(WorldMutation::TeardownSource(source_index as u8));
      }
    }
  }

  fn advance_inserters(&mut self, events: &mut Vec<String>) {
    const INSERTION_RATE: u32 = 5;
    let mut transfers = Vec::new();
    for (target_index, factory) in self.world.factories.iter().enumerate() {
      let target_position = self.world.topology.position(factory.node);
      for item in factory.production.recipe.inputs.keys().copied() {
        for (source_index, source) in self.world.sources.iter().enumerate() {
          let source_position = self.world.topology.position(source.node);
          if adjacent(source_position, target_position) && source.stockpile.count(item) > 0 {
            transfers.push((target_index, item, AdjacentProvider::Source(source_index)));
          }
        }
        for (source_index, source) in self.world.factories.iter().enumerate() {
          if source_index == target_index {
            continue;
          }
          let source_position = self.world.topology.position(source.node);
          if adjacent(source_position, target_position)
            && source.production.inventory.count(item) > 0
          {
            transfers.push((
              target_index,
              item,
              AdjacentProvider::Factory(source_index),
            ));
          }
        }
      }
    }

    for (target_index, item, provider) in transfers {
      let Some(target) = self.world.factories.get(target_index) else {
        continue;
      };
      let needed = target
        .input_buffer
        .saturating_sub(target.production.inventory.count(item));
      if needed == 0 {
        continue;
      }
      let consumer = format!("inserter-{}", target.node);
      let node = target.node;
      if !self.consume_power(node, 1, &consumer, events) {
        continue;
      }
      let quantity = INSERTION_RATE.min(needed);
      let (moved, from, to) = match provider {
        AdjacentProvider::Source(source_index) => {
          let source = &mut self.world.sources[source_index];
          let target = &mut self.world.factories[target_index];
          (
            source.stockpile.transfer_up_to(
              &self.content,
              &mut target.production.inventory,
              item,
              quantity,
            ),
            source.node,
            target.node,
          )
        }
        AdjacentProvider::Factory(source_index) => {
          let (source, target) = two_factories_mut(
            &mut self.world.factories,
            source_index,
            target_index,
          );
          (
            source.production.inventory.transfer_up_to(
              &self.content,
              &mut target.production.inventory,
              item,
              quantity,
            ),
            source.node,
            target.node,
          )
        }
      };
      if moved > 0 {
        events.push(format!("inserter move {item} +{moved} {from} -> {to}"));
      }
    }
  }

  fn advance_production(&mut self, events: &mut Vec<String>) {
    let production_cost = self
      .world
      .power
      .as_ref()
      .map_or(0, |power| power.spec.production_cost);
    for factory_index in 0..self.world.factories.len() {
      let wants_power = self.world.factories[factory_index]
        .production
        .wants_power(&self.content);
      let node = self.world.factories[factory_index].node;
      if wants_power
        && !self.consume_power(node, production_cost, &node.to_string(), events)
      {
        continue;
      }
      let factory = &mut self.world.factories[factory_index];
      let produced = factory.production.advance(&self.content, events);
      if let Some(blocked) = factory.production.blocked {
        factory.alerts.record(
          self.world.tick,
          match blocked {
            ProductionBlockReason::OutputFull => "product output full",
            ProductionBlockReason::NoOutputSpace => "no space for product",
          },
        );
      }
      let output_item = factory.production.recipe.output_item;
      self.metrics.record_crafted(output_item, produced);
    }
  }

  fn advance_power(&mut self, events: &mut Vec<String>) {
    let Some(power) = &mut self.world.power else {
      return;
    };
    let owner = BatteryOwner::Node(NodeId::PowerPlant);
    let battery = self
      .world
      .batteries
      .get_mut(&owner)
      .expect("powered scenario has a plant battery");
    let (burned, generated) = power.generate(battery, events);
    self.metrics.fuel_burned += burned;
    self.metrics.energy_generated += generated;
    self.construct_power_lines(events);
    self.balance_batteries(events);
  }

  fn construct_power_lines(&mut self, events: &mut Vec<String>) {
    if self.world.power_lines_built {
      return;
    }
    let plant = self.world.topology.position(NodeId::PowerPlant);
    let target = self
      .world
      .batteries
      .keys()
      .filter_map(|owner| match owner {
        BatteryOwner::Node(node) if *node != NodeId::PowerPlant => {
          Some((*owner, self.world.topology.position(*node)))
        }
        BatteryOwner::Node(_) | BatteryOwner::Hauler(_) | BatteryOwner::PowerLine(_) => None,
      })
      .min_by_key(|(owner, position)| {
        (
          plant.x.abs_diff(position.x).pow(2) + plant.y.abs_diff(position.y).pow(2),
          *owner,
        )
      });
    let Some((_, target)) = target else {
      events.push("power line no battery target found".into());
      if let Some(power) = &mut self.world.power {
        power
          .alerts
          .record(self.world.tick, "no power source found");
      }
      return;
    };

    let mut current = plant;
    let mut built = Vec::new();
    while !adjacent(current, target) {
      current = GridPosition {
        x: current.x + (target.x - current.x).signum(),
        y: current.y + (target.y - current.y).signum(),
      };
      if current == target
        || current.x < 0
        || current.y < 0
        || current.x >= self.world.topology.width
        || current.y >= self.world.topology.height
      {
        break;
      }
      if self.world.power_lines.insert(current) {
        let owner = BatteryOwner::PowerLine(current);
        self
          .world
          .batteries
          .insert(owner, Battery::new(owner, 0, 1_000));
        built.push(current);
      }
    }
    self.world.power_lines_built = true;
    events.push(format!(
      "power line built {} cells {:?} toward {},{}",
      built.len(),
      built,
      target.x,
      target.y
    ));
  }

  fn balance_batteries(&mut self, events: &mut Vec<String>) {
    let positions = self
      .world
      .batteries
      .keys()
      .copied()
      .map(|owner| {
        let position = match owner {
          BatteryOwner::Node(node) => self.world.topology.position(node),
          BatteryOwner::Hauler(id) => {
            let hauler = self
              .world
              .haulers
              .iter()
              .find(|hauler| hauler.id == id)
              .expect("battery owner has a hauler");
            self.world.topology.position(hauler.position)
          }
          BatteryOwner::PowerLine(position) => position,
        };
        (owner, position)
      })
      .collect::<std::collections::BTreeMap<_, _>>();
    let mut unseen = positions.keys().copied().collect::<std::collections::BTreeSet<_>>();

    while let Some(start) = unseen.pop_first() {
      let mut component = vec![start];
      let mut cursor = 0;
      while cursor < component.len() {
        let owner = component[cursor];
        cursor += 1;
        let neighbors = unseen
          .iter()
          .copied()
          .filter(|candidate| {
            positions[&owner] == positions[candidate]
              || adjacent(positions[&owner], positions[candidate])
          })
          .collect::<Vec<_>>();
        for neighbor in neighbors {
          unseen.remove(&neighbor);
          component.push(neighbor);
        }
      }
      self.balance_battery_component(&component, events);
    }
  }

  fn balance_battery_component(
    &mut self,
    owners: &[BatteryOwner],
    events: &mut Vec<String>,
  ) {
    let total_energy = owners
      .iter()
      .map(|owner| u64::from(self.world.batteries[owner].energy))
      .sum::<u64>();
    let total_capacity = owners
      .iter()
      .map(|owner| u64::from(self.world.batteries[owner].capacity))
      .sum::<u64>();
    if total_capacity == 0 {
      return;
    }
    let mut assigned = 0_u64;
    let mut shares = owners
      .iter()
      .map(|owner| {
        let capacity = u64::from(self.world.batteries[owner].capacity);
        let numerator = total_energy * capacity;
        let energy = numerator / total_capacity;
        assigned += energy;
        (*owner, energy as u32, numerator % total_capacity)
      })
      .collect::<Vec<_>>();
    shares.sort_by_key(|(owner, _, remainder)| (std::cmp::Reverse(*remainder), *owner));
    for (_, energy, _) in shares
      .iter_mut()
      .take(total_energy.saturating_sub(assigned) as usize)
    {
      *energy += 1;
    }
    shares.sort_by_key(|(owner, _, _)| *owner);
    let changed = shares.iter().any(|(owner, energy, _)| {
      self.world.batteries[owner].energy != *energy
    });
    for (owner, energy, _) in shares {
      self.world
        .batteries
        .get_mut(&owner)
        .expect("component owner has a battery")
        .energy = energy;
    }
    if changed && owners.len() > 1 {
      events.push(format!(
        "power balance {} energy across {} adjacent batteries",
        total_energy,
        owners.len()
      ));
    }
  }

  fn consume_power(
    &mut self,
    node: NodeId,
    amount: u32,
    consumer: &str,
    events: &mut Vec<String>,
  ) -> bool {
    if self.world.power.is_none() || amount == 0 {
      return true;
    }
    let owner = BatteryOwner::Node(node);
    let Some(battery) = self.world.batteries.get_mut(&owner) else {
      events.push(format!("power starved {consumer} missing battery at {node}"));
      self.metrics.power_starvations += 1;
      self.record_node_alert(node, format!("not enough energy for {consumer}"));
      return false;
    };
    if battery.consume(amount) {
      self.metrics.energy_consumed += amount;
      events.push(format!(
        "power consume {consumer} {amount} battery {}/{} at {node}",
        battery.energy, battery.capacity
      ));
      true
    } else {
      let energy = battery.energy;
      let capacity = battery.capacity;
      self.metrics.power_starvations += 1;
      events.push(format!(
        "power starved {consumer} need {amount} battery {}/{} at {node}",
        energy, capacity
      ));
      self.record_node_alert(node, format!("not enough energy for {consumer}"));
      false
    }
  }

  fn record_node_alert(&mut self, node: NodeId, message: impl Into<String>) {
    let message = message.into();
    match node {
      NodeId::Source(index) => {
        if let Some(source) = self.world.sources.get_mut(usize::from(index)) {
          source.alerts.record(self.world.tick, message);
        }
      }
      NodeId::Factory(index) => {
        if let Some(factory) = self.world.factories.get_mut(usize::from(index)) {
          factory.alerts.record(self.world.tick, message);
        }
      }
      NodeId::PowerPlant => {
        if let Some(power) = &mut self.world.power {
          power.alerts.record(self.world.tick, message);
        }
      }
      NodeId::Structure(index) => {
        if let Some(structure) = self
          .world
          .structures
          .iter_mut()
          .find(|structure| structure.node == NodeId::Structure(index))
        {
          structure.alerts.record(self.world.tick, message);
        }
      }
      NodeId::Road | NodeId::BuildSite(_) | NodeId::Transit(_) => {}
    }
  }

  fn nodes_in_transfer_range(&self, left: NodeId, right: NodeId) -> bool {
    let left = self.world.topology.position(left);
    let right = self.world.topology.position(right);
    left == right || adjacent(left, right)
  }

  fn queue_hauler_movements(&mut self, events: &mut Vec<String>) {
    let mut reserved_transit = std::collections::BTreeSet::new();
    let topology = &self.world.topology;
    for hauler in &mut self.world.haulers {
      let target = match &hauler.dispatch {
        DispatchReceiverState::Assigned(assignment) => match assignment.phase {
          DispatchPhase::Collect => assignment.source,
          DispatchPhase::Deliver => assignment.destination,
          DispatchPhase::Retrieve => assignment.source,
          DispatchPhase::Deploy => assignment.destination,
        },
        DispatchReceiverState::Unassigned => {
          if hauler.cargo.is_empty() {
            hauler.position
          } else {
            NodeId::Factory(0)
          }
        }
      };
      hauler.set_target(target);
      let current = hauler.position;
      let current_position = topology.position(current);
      let target_position = topology.position(target);
      if current_position == target_position || adjacent(current_position, target_position) {
        continue;
      }
      let Some(next) = topology.step_toward(current, target) else {
        events.push(format!(
          "move hauler-{} no available path {} -> {}",
          hauler.id, current, target
        ));
        continue;
      };
      if next == current {
        continue;
      }
      if matches!(next, NodeId::Transit(_)) && !reserved_transit.insert(next) {
        events.push(format!(
          "move hauler-{} blocked by queued transit occupancy at {}",
          hauler.id, next
        ));
        continue;
      }
      self.world.queued_mutations.push(WorldMutation::MoveHauler {
        hauler_id: hauler.id,
        from: current,
        to: next,
        target,
      });
    }
  }

  pub fn step(&mut self) -> TickSnapshot {
    self.world.tick += 1;
    let mut events = Vec::new();
    self.advance_receiver_phases(&mut events);
    self.advance_power(&mut events);
    self.advance_mining(&mut events);
    self.refresh_dispatch_intents();
    self.assign_dispatch(&mut events);
    self.collect(&mut events);
    self.deliver(&mut events);
    self.retrieve_and_deploy(&mut events);
    self.queue_source_lifecycle();
    self.advance_inserters(&mut events);
    self.advance_production(&mut events);
    self.queue_hauler_movements(&mut events);
    self.apply_world_mutations(&mut events);
    self.metrics.ticks = self.world.tick;
    if events.is_empty() {
      self.metrics.idle_ticks += 1;
    }
    self.snapshot(events)
  }

  pub fn metrics(&self) -> RunMetricsSnapshot {
    self.metrics.snapshot()
  }

  pub fn snapshot(&self, events: Vec<String>) -> TickSnapshot {
    TickSnapshot {
      tick: self.world.tick,
      scenario: ScenarioSnapshot {
        id: self.world.scenario.id,
        name: self.world.scenario.name.clone(),
      },
      topology: TopologySnapshot {
        width: self.world.topology.width,
        height: self.world.topology.height,
        nodes: self.world.topology.nodes.clone(),
        blocked: self.world.topology.blocked.clone(),
        obstacles: self.world.topology.obstacles.clone(),
        power_lines: self.world.power_lines.clone(),
      },
      sources: self
        .world
        .sources
        .iter()
        .map(|source| SourceSnapshot {
          node: source.node,
          item: source.item,
          stockpile: source.stockpile.snapshot(),
          mining: source.mining.clone(),
          dispatch: source.dispatch.clone(),
          deployed: source.deployed,
          exhausted: source.exhausted,
          alerts: source.alerts.clone(),
        })
        .collect(),
      haulers: self
        .world
        .haulers
        .iter()
        .map(|hauler| HaulerSnapshot {
          id: hauler.id,
          position: hauler.position,
          position_grid: self.world.topology.position(hauler.position),
          target: hauler.target,
          target_grid: self.world.topology.position(hauler.target),
          cargo: hauler.cargo.snapshot(),
          carry_limit: hauler.carry_limit,
          dispatch: hauler.dispatch.clone(),
          alerts: hauler.alerts.clone(),
        })
        .collect(),
      factories: self
        .world
        .factories
        .iter()
        .map(|factory| FactorySnapshot {
          node: factory.node,
          inventory: factory.production.inventory.snapshot(),
          craft: factory.production.craft_snapshot(),
          dispatch: factory.dispatch.clone(),
          alerts: factory.alerts.clone(),
        })
        .collect(),
      structures: self.world.structures.clone(),
      power: self.world.power.as_ref().map(|power| {
        power.snapshot(self.world.batteries.values().cloned())
      }),
      events,
    }
  }
}

fn adjacent(left: GridPosition, right: GridPosition) -> bool {
  left != right && left.x.abs_diff(right.x) <= 1 && left.y.abs_diff(right.y) <= 1
}

fn two_factories_mut(
  factories: &mut [FactoryNode],
  left: usize,
  right: usize,
) -> (&mut FactoryNode, &mut FactoryNode) {
  assert_ne!(left, right, "factory transfer endpoints must differ");
  if left < right {
    let (before, after) = factories.split_at_mut(right);
    (&mut before[left], &mut after[0])
  } else {
    let (before, after) = factories.split_at_mut(left);
    (&mut after[0], &mut before[right])
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
    ContentDatabase, GridPoint, BUILDING_MATERIALS, BUILDING_MATERIALS_SCENARIO, COAL,
    COPPER_BARS, COPPER_ORE, DEPLOYMENT_DEMO_SCENARIO, DISTRIBUTED_CHAIN_SCENARIO, FRAMES,
    IRON_BARS, IRON_BARS_FLEET_SCENARIO, IRON_BARS_SCENARIO, IRON_ORE, MINING_DRILL, MOTORS,
    PATHFINDING_DEMO_SCENARIO, POWERED_IRONWORKS_SCENARIO, POWER_LINE_SCENARIO,
    PRODUCTION_CHAIN_SCENARIO, STONE,
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
  fn mining_depletes_a_finite_deposit_and_stops() {
    let content = ContentDatabase::starter();
    let mut extractor = MiningExtractor::for_item(&content, IRON_ORE, 4, 10);
    let mut stockpile = Inventory::new(1024, 1024);

    assert_eq!(4, extractor.advance(&content, &mut stockpile));
    assert_eq!(4, extractor.advance(&content, &mut stockpile));
    assert_eq!(2, extractor.advance(&content, &mut stockpile));
    assert_eq!(0, extractor.advance(&content, &mut stockpile));
    assert_eq!(Deposit::Finite(0), extractor.deposit);
    assert_eq!(10, stockpile.count(IRON_ORE));
  }

  #[test]
  fn mining_manifest_items_create_from_nothing() {
    let content = ContentDatabase::starter();
    let mut extractor = MiningExtractor::for_item(&content, STONE, 2, 0);
    let mut stockpile = Inventory::new(3, 3);

    assert_eq!(Deposit::Manifest, extractor.deposit);
    assert_eq!(2, extractor.advance(&content, &mut stockpile));
    assert_eq!(1, extractor.advance(&content, &mut stockpile));
    assert_eq!(0, extractor.advance(&content, &mut stockpile));
    assert_eq!(3, stockpile.count(STONE));
  }

  #[test]
  fn manifest_factories_produce_without_recipe_inputs() {
    let mut content = ContentDatabase::starter();
    content
      .scenarios
      .get_mut(&IRON_BARS_SCENARIO)
      .expect("iron-bars scenario exists")
      .factories[0]
      .product_item = STONE;
    let mut state = GameState::new(content, IRON_BARS_SCENARIO).unwrap();

    let first = state.step();

    assert_eq!(Some(&1), first.factories[0].inventory.items.get(STONE.as_str()));
    assert_eq!(Some(&1), state.metrics().crafted.get(STONE.as_str()));
  }

  #[test]
  fn ingredientless_non_manifest_recipes_remain_invalid() {
    let mut content = ContentDatabase::starter();
    content
      .scenarios
      .get_mut(&IRON_BARS_SCENARIO)
      .expect("iron-bars scenario exists")
      .factories[0]
      .product_item = IRON_ORE;

    assert!(matches!(
      GameState::new(content, IRON_BARS_SCENARIO),
      Err(SimulationError::RecipeMissingIngredients(IRON_ORE))
    ));
  }

  #[test]
  fn source_deposit_depletes_in_the_iron_bars_scenario() {
    let mut state = GameState::starter_iron_bars();
    let snapshots: Vec<_> = (0..6).map(|_| state.step()).collect();

    assert_eq!(Deposit::Finite(6), snapshots[0].sources[0].mining.deposit);
    assert_eq!(Deposit::Finite(0), snapshots[2].sources[0].mining.deposit);
    assert!(snapshots[0]
      .events
      .iter()
      .any(|event| event.starts_with("mine")));
    assert!(snapshots[3..].iter().all(|snapshot| snapshot
      .events
      .iter()
      .all(|event| !event.starts_with("mine"))));
  }

  #[test]
  fn run_metrics_summarize_a_fixed_iron_bars_run() {
    let mut state = GameState::starter_iron_bars();
    for _ in 0..6 {
      state.step();
    }
    let metrics = state.metrics();

    assert_eq!(6, metrics.ticks);
    assert_eq!(
      9,
      metrics.mined.get(IRON_ORE.as_str()).copied().unwrap_or(0)
    );
    assert_eq!(
      20,
      metrics
        .crafted
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert_eq!(3, metrics.dispatches_assigned);
    assert_eq!(9, metrics.units_collected);
    assert_eq!(9, metrics.units_delivered);
    assert_eq!(0, metrics.idle_ticks);
  }

  #[test]
  fn run_metrics_are_deterministic_across_runs() {
    let mut first = GameState::starter_iron_bars();
    let mut second = GameState::starter_iron_bars();
    for _ in 0..12 {
      first.step();
      second.step();
    }
    assert_eq!(first.metrics(), second.metrics());
  }

  #[test]
  fn dispatch_lifecycle_advances_through_collect_and_deliver() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();

    let first = state.step();
    assert!(matches!(
      first.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Collect,
        ..
      })
    ));
    assert_eq!(NodeId::Road, first.haulers[0].position);
    assert_eq!(
      3,
      first.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert_eq!(NodeId::Source(0), first.haulers[0].target);

    let second = state.step();
    assert!(matches!(
      second.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Road, second.haulers[0].position);
    assert_eq!(
      0,
      second.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(!second.factories[0].craft.crafting);
    assert_eq!(
      Some(&10),
      second.factories[0].inventory.items.get(IRON_BARS.as_str())
    );

    let third = state.step();
    assert!(matches!(
      third.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Collect,
        ..
      })
    ));
    assert_eq!(NodeId::Road, third.haulers[0].position);
    assert_eq!(
      3,
      third.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(!third.factories[0].craft.crafting);

    let fourth = state.step();
    assert!(matches!(
      fourth.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Road, fourth.haulers[0].position);
    assert_eq!(NodeId::Factory(0), fourth.haulers[0].target);
  }

  #[test]
  fn repeated_factory_conditions_refresh_one_object_alert() {
    let mut state = GameState::starter_iron_bars();
    let factory = &mut state.world.factories[0];
    factory.production.inventory.force_insert(IRON_ORE, 3);
    factory.production.inventory.force_insert(IRON_BARS, 20);

    let first = state.step();
    assert_eq!(
      Some(&AlertEntry {
        tick: 1,
        message: "product output full".into(),
      }),
      first.factories[0].alerts.latest()
    );

    let second = state.step();
    assert_eq!(1, second.factories[0].alerts.entries.len());
    assert_eq!(
      Some(&AlertEntry {
        tick: 2,
        message: "product output full".into(),
      }),
      second.factories[0].alerts.latest()
    );
  }

  #[test]
  fn receiver_alerts_remain_isolated_to_the_owning_hauler() {
    let mut state = GameState::starter_iron_bars();
    let hauler = &mut state.world.haulers[0];
    hauler.assign(DispatchAssignment::collect(
      IRON_ORE,
      NodeId::Source(0),
      NodeId::Factory(0),
    ));
    hauler.cargo.force_insert(IRON_ORE, 1);

    let snapshot = state.step();
    assert_eq!("collect => deliver", snapshot.haulers[0].alerts.latest().unwrap().message);
    assert!(snapshot.sources.iter().all(|source| source.alerts.entries.is_empty()));
    assert!(snapshot
      .factories
      .iter()
      .all(|factory| factory.alerts.entries.is_empty()));
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
        .factories[0]
        .inventory
        .items
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    }));
  }

  #[test]
  fn adjacent_transfer_stops_before_entering_occupied_buildings() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();

    let first = state.step();
    assert_eq!(NodeId::Road, first.haulers[0].position);
    assert!(first
      .events
      .iter()
      .any(|event| event.contains("dispatch collect")));
    assert!(!first
      .events
      .iter()
      .any(|event| event.contains("dispatch deliver")));

    let second = state.step();
    assert_eq!(NodeId::Road, second.haulers[0].position);
    assert!(second
      .events
      .iter()
      .any(|event| event.contains("dispatch deliver")));
    let factory_position = second
      .topology
      .nodes
      .iter()
      .find(|node| node.id == NodeId::Factory(0))
      .unwrap()
      .position;
    assert!(second
      .topology
      .blocked
      .contains(&factory_position));
  }

  #[test]
  fn collect_and_deliver_require_transfer_range() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();
    state.world.haulers[0].position = NodeId::Factory(0);
    state.world.haulers[0].dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source(0),
      destination: NodeId::Factory(0),
      phase: DispatchPhase::Collect,
      priority: DispatchPriority::NORMAL,
    });

    let collect_snapshot = state.step();
    assert_eq!(
      0,
      collect_snapshot.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(collect_snapshot
      .events
      .iter()
      .all(|event| !event.contains("dispatch collect")));
    assert_eq!(NodeId::Road, collect_snapshot.haulers[0].position);

    state.world.haulers[0].position = NodeId::Source(0);
    state.world.haulers[0]
      .cargo
      .insert_exact(&ContentDatabase::starter(), IRON_ORE, 3)
      .unwrap();
    state.world.haulers[0].dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source(0),
      destination: NodeId::Factory(0),
      phase: DispatchPhase::Deliver,
      priority: DispatchPriority::NORMAL,
    });

    let deliver_snapshot = state.step();
    assert_eq!(
      3,
      deliver_snapshot.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(deliver_snapshot
      .events
      .iter()
      .all(|event| !event.contains("dispatch deliver")));
    assert_eq!(NodeId::Road, deliver_snapshot.haulers[0].position);
  }

  #[test]
  fn factory_advertises_one_intent_per_missing_input() {
    let mut state =
      GameState::new(ContentDatabase::starter(), BUILDING_MATERIALS_SCENARIO).unwrap();

    let first = state.step();
    let intent_items: Vec<&str> = first
      .factories[0]
      .dispatch
      .intents
      .iter()
      .map(|intent| intent.item.as_str())
      .collect();
    assert_eq!(vec![IRON_BARS.as_str(), STONE.as_str()], intent_items);
  }

  #[test]
  fn dispatch_targets_the_nearest_matching_world_object() {
    let mut content = ContentDatabase::starter();
    let scenario = content
      .scenarios
      .get_mut(&IRON_BARS_SCENARIO)
      .expect("iron-bars scenario exists");
    scenario.sources.push(scenario.sources[0].clone());
    scenario.layout.width = 6;
    scenario.layout.height = 3;
    scenario.layout.source_positions =
      vec![GridPoint { x: 0, y: 1 }, GridPoint { x: 4, y: 1 }];
    scenario.layout.road_position = GridPoint { x: 3, y: 0 };
    scenario.layout.factory_positions = vec![GridPoint { x: 5, y: 1 }];

    let mut state = GameState::new(content, IRON_BARS_SCENARIO).unwrap();
    let first = state.step();
    let DispatchReceiverState::Assigned(assignment) = &first.haulers[0].dispatch else {
      panic!("nearest source receives the dispatch");
    };

    assert_eq!(IRON_ORE, assignment.item);
    assert_eq!(NodeId::Source(1), assignment.source);
    assert_eq!(NodeId::Factory(0), assignment.destination);
  }

  fn contested_dispatch_content() -> ContentDatabase {
    let mut content = ContentDatabase::starter();
    let scenario = content
      .scenarios
      .get_mut(&IRON_BARS_SCENARIO)
      .expect("iron-bars scenario exists");
    scenario.factories.push(scenario.factories[0].clone());
    scenario.layout.width = 6;
    scenario.layout.height = 3;
    scenario.layout.source_positions = vec![GridPoint { x: 0, y: 1 }];
    scenario.layout.road_position = GridPoint { x: 2, y: 1 };
    scenario.layout.factory_positions =
      vec![GridPoint { x: 3, y: 0 }, GridPoint { x: 5, y: 1 }];
    content
  }

  #[test]
  fn higher_priority_dispatch_demand_wins_one_hauler_contention() {
    let mut state = GameState::new(contested_dispatch_content(), IRON_BARS_SCENARIO).unwrap();

    assert_eq!(
      DispatchPriority::NORMAL,
      state.dispatch_priority(NodeId::Factory(1), IRON_ORE)
    );
    assert_eq!(
      None,
      state.set_dispatch_priority(
        NodeId::Factory(1),
        IRON_ORE,
        DispatchPriority::HIGH,
      )
    );

    let first = state.step();
    let DispatchReceiverState::Assigned(assignment) = &first.haulers[0].dispatch else {
      panic!("the available hauler receives contested demand");
    };

    assert_eq!(NodeId::Factory(1), assignment.destination);
    assert_eq!(DispatchPriority::HIGH, assignment.priority);
    assert_eq!(
      Some(DispatchPriority::HIGH),
      first.factories[1]
        .dispatch
        .intents
        .iter()
        .find(|intent| intent.verb == DispatchVerb::Deliver && intent.item == IRON_ORE)
        .map(|intent| intent.priority)
    );
  }

  #[test]
  fn equal_priority_dispatch_demand_uses_deterministic_destination_order() {
    let mut first = GameState::new(contested_dispatch_content(), IRON_BARS_SCENARIO).unwrap();
    let mut second = GameState::new(contested_dispatch_content(), IRON_BARS_SCENARIO).unwrap();

    let first_snapshot = first.step();
    let second_snapshot = second.step();
    assert_eq!(first_snapshot, second_snapshot);

    let DispatchReceiverState::Assigned(assignment) = &first_snapshot.haulers[0].dispatch else {
      panic!("the available hauler receives contested demand");
    };
    assert_eq!(NodeId::Factory(0), assignment.destination);
    assert_eq!(DispatchPriority::NORMAL, assignment.priority);
  }

  #[test]
  fn scenario_factory_buffer_is_the_dispatch_threshold() {
    let mut content = ContentDatabase::starter();
    content
      .scenarios
      .get_mut(&IRON_BARS_SCENARIO)
      .expect("iron-bars scenario exists")
      .factories[0]
      .input_buffer = 4;
    let mut state = GameState::new(content, IRON_BARS_SCENARIO).unwrap();
    let factory = &mut state.world.factories[0];

    factory
      .production
      .inventory
      .insert_exact(&state.content, IRON_ORE, 4)
      .expect("configured reservation holds the buffer");
    factory.refresh_dispatch(&state.content);
    assert!(factory
      .dispatch
      .intents
      .iter()
      .all(|intent| intent.item != IRON_ORE));

    factory
      .production
      .inventory
      .remove_exact(IRON_ORE, 1)
      .expect("buffer stock can be consumed");
    factory.refresh_dispatch(&state.content);
    assert!(factory
      .dispatch
      .intents
      .iter()
      .any(|intent| intent.item == IRON_ORE));
  }

  #[test]
  fn deployment_targets_the_nearest_unoccupied_matching_source() {
    let mut content = ContentDatabase::starter();
    let scenario = content
      .scenarios
      .get_mut(&DEPLOYMENT_DEMO_SCENARIO)
      .expect("deployment scenario exists");
    scenario.sources.push(scenario.sources[0].clone());
    scenario.layout.width = 6;
    scenario.layout.height = 3;
    scenario.layout.source_positions =
      vec![GridPoint { x: 0, y: 1 }, GridPoint { x: 4, y: 1 }];
    scenario.layout.road_position = GridPoint { x: 3, y: 0 };
    scenario.layout.factory_positions = vec![GridPoint { x: 5, y: 1 }];

    let mut state = GameState::new(content, DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let first = state.step();
    let DispatchReceiverState::Assigned(assignment) = &first.haulers[0].dispatch else {
      panic!("drill receives the nearest deployment target");
    };

    assert_eq!(MINING_DRILL, assignment.item);
    assert_eq!(NodeId::Source(1), assignment.destination);
  }

  #[test]
  fn building_materials_loop_services_both_source_types() {
    let mut first =
      GameState::new(ContentDatabase::starter(), BUILDING_MATERIALS_SCENARIO).unwrap();
    let mut second =
      GameState::new(ContentDatabase::starter(), BUILDING_MATERIALS_SCENARIO).unwrap();

    let first_run: Vec<_> = (0..20).map(|_| first.step()).collect();
    let second_run: Vec<_> = (0..20).map(|_| second.step()).collect();
    assert_eq!(first_run, second_run);

    let metrics = first.metrics();
    assert!(metrics.mined.get(IRON_BARS.as_str()).copied().unwrap_or(0) > 0);
    assert!(metrics.mined.get(STONE.as_str()).copied().unwrap_or(0) > 0);
    assert!(
      metrics
        .crafted
        .get(BUILDING_MATERIALS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    );
    assert!(first_run.iter().any(|snapshot| {
      snapshot
        .factories[0]
        .inventory
        .items
        .get(BUILDING_MATERIALS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    }));
  }

  #[test]
  fn adjacent_inserters_run_the_multi_factory_drill_chain() {
    let mut first =
      GameState::new(ContentDatabase::starter(), PRODUCTION_CHAIN_SCENARIO).unwrap();
    let mut second =
      GameState::new(ContentDatabase::starter(), PRODUCTION_CHAIN_SCENARIO).unwrap();

    let first_run: Vec<_> = (0..80).map(|_| first.step()).collect();
    let second_run: Vec<_> = (0..80).map(|_| second.step()).collect();
    assert_eq!(first_run, second_run);
    assert_eq!(5, first_run[0].factories.len());
    assert!(first_run.iter().flat_map(|snapshot| &snapshot.events).any(
      |event| event.starts_with("inserter move iron_ore") && event.contains("source-0")
    ));
    assert!(first_run.iter().flat_map(|snapshot| &snapshot.events).any(
      |event| event.starts_with("inserter move frames") && event.contains("factory-2")
    ));
    assert!(
      first_run
        .last()
        .unwrap()
        .factories[4]
        .inventory
        .items
        .get(MINING_DRILL.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    );
  }

  #[test]
  fn haulers_retrieve_factory_output_for_nonadjacent_demand() {
    let mut state =
      GameState::new(ContentDatabase::starter(), DISTRIBUTED_CHAIN_SCENARIO).unwrap();
    let snapshots: Vec<_> = (0..100).map(|_| state.step()).collect();

    assert!(snapshots.iter().flat_map(|snapshot| &snapshot.events).any(
      |event| event.contains("dispatch collect")
        && event.contains("factory-0")
        && event.contains("factory-1")
    ));
    assert!(
      snapshots
        .last()
        .unwrap()
        .factories[1]
        .inventory
        .items
        .get(FRAMES.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    );
    assert!(snapshots.iter().all(|snapshot| {
      snapshot.haulers.iter().all(|hauler| {
        !matches!(hauler.position, NodeId::Factory(_))
          && !matches!(hauler.position, NodeId::Source(_))
      })
    }));
  }

  #[test]
  fn inserters_ignore_nonadjacent_resource_containers() {
    let mut content = ContentDatabase::starter();
    content
      .scenarios
      .get_mut(&PRODUCTION_CHAIN_SCENARIO)
      .unwrap()
      .layout
      .factory_positions[4] = factory_content::GridPoint { x: 0, y: 0 };
    let mut state = GameState::new(content, PRODUCTION_CHAIN_SCENARIO).unwrap();

    for _ in 0..80 {
      state.step();
    }

    assert_eq!(0, state.world.factories[4].production.inventory.count(MINING_DRILL));
    assert!(state.world.factories[2].production.inventory.count(FRAMES) > 0);
    assert!(state.world.factories[3].production.inventory.count(MOTORS) > 0);
  }

  #[test]
  fn fleet_arbitration_assigns_only_needed_haulers() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_FLEET_SCENARIO).unwrap();

    let first = state.step();
    let assigned: Vec<bool> = first
      .haulers
      .iter()
      .map(|hauler| matches!(hauler.dispatch, DispatchReceiverState::Assigned(_)))
      .collect();
    assert_eq!(vec![true, true, false], assigned);
  }

  #[test]
  fn deliveries_never_exceed_the_factory_input_buffer() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_FLEET_SCENARIO).unwrap();

    let mut bars_seen = false;
    for _ in 0..16 {
      let snapshot = state.step();
      let input_stock = snapshot
        .factories[0]
        .inventory
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0);
      assert!(input_stock <= 6, "input stock {input_stock} exceeds buffer");
      bars_seen |= snapshot
        .factories[0]
        .inventory
        .items
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
        > 0;
    }
    assert!(bars_seen);
  }

  #[test]
  fn powered_ironworks_burns_fuel_and_routes_coal_to_the_plant() {
    let mut state = GameState::new(ContentDatabase::starter(), POWERED_IRONWORKS_SCENARIO).unwrap();
    let snapshots: Vec<_> = (0..32).map(|_| state.step()).collect();
    let metrics = state.metrics();

    assert!(metrics.fuel_burned > 0);
    assert!(metrics.energy_generated > 0);
    assert!(metrics.energy_consumed > 0);
    assert!(metrics.mined.get(COAL.as_str()).copied().unwrap_or(0) > 0);
    assert!(
      metrics
        .crafted
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    );
    assert!(snapshots.iter().any(|snapshot| {
      snapshot
        .events
        .iter()
        .any(|event| event.contains("dispatch deliver") && event.contains("power-plant"))
    }));
    assert!(snapshots.iter().all(|snapshot| {
      snapshot
        .power
        .as_ref()
        .is_some_and(|power| power.energy <= power.capacity)
    }));
  }

  #[test]
  fn an_empty_power_grid_blocks_energy_gated_systems() {
    let content = ContentDatabase::starter();
    let mut state = GameState::new(content, POWERED_IRONWORKS_SCENARIO).unwrap();
    let power = state
      .world
      .power
      .as_mut()
      .expect("scenario has a power plant");
    power.fuel.remove_up_to(COAL, u32::MAX);

    let snapshot = state.step();
    let metrics = state.metrics();

    assert!(snapshot
      .sources
      .iter()
      .all(|source| source.stockpile.items.is_empty()));
    assert_eq!(0, metrics.energy_generated);
    assert!(metrics.power_starvations >= 2);
    assert!(snapshot
      .events
      .iter()
      .any(|event| event.starts_with("power starved mining")));
  }

  #[test]
  fn batteries_clamp_capacity_and_reject_overdraw() {
    let owner = BatteryOwner::Node(NodeId::Factory(0));
    let mut battery = Battery::new(owner, 9, 0);

    assert_eq!(5, battery.capacity);
    assert_eq!(5, battery.energy);
    assert_eq!(0, battery.charge(10));
    assert!(!battery.consume(6));
    assert_eq!(5, battery.energy);
    assert!(battery.consume(3));
    assert_eq!(2, battery.energy);
  }

  #[test]
  fn adjacent_batteries_balance_by_capacity_without_losing_energy() {
    let mut state =
      GameState::new(ContentDatabase::starter(), POWERED_IRONWORKS_SCENARIO).unwrap();
    state.world.batteries.clear();
    let first = BatteryOwner::Node(NodeId::Source(0));
    let second = BatteryOwner::Node(NodeId::Source(1));
    state
      .world
      .batteries
      .insert(first, Battery::new(first, 25, 100));
    state
      .world
      .batteries
      .insert(second, Battery::new(second, 75, 200));

    state.balance_batteries(&mut Vec::new());

    assert_eq!(33, state.world.batteries[&first].energy);
    assert_eq!(67, state.world.batteries[&second].energy);
    assert_eq!(
      100,
      state
        .world
        .batteries
        .values()
        .map(|battery| battery.energy)
        .sum::<u32>()
    );
  }

  #[test]
  fn disconnected_batteries_do_not_exchange_energy() {
    let mut state =
      GameState::new(ContentDatabase::starter(), POWERED_IRONWORKS_SCENARIO).unwrap();
    state.world.batteries.clear();
    let source = BatteryOwner::Node(NodeId::Source(0));
    let factory = BatteryOwner::Node(NodeId::Factory(0));
    state
      .world
      .batteries
      .insert(source, Battery::new(source, 25, 100));
    state
      .world
      .batteries
      .insert(factory, Battery::new(factory, 75, 100));

    state.balance_batteries(&mut Vec::new());

    assert_eq!(25, state.world.batteries[&source].energy);
    assert_eq!(75, state.world.batteries[&factory].energy);
  }

  #[test]
  fn powered_snapshots_expose_every_object_battery() {
    let mut state =
      GameState::new(ContentDatabase::starter(), POWERED_IRONWORKS_SCENARIO).unwrap();
    let snapshot = state.step();
    let power = snapshot.power.unwrap();

    assert_eq!(6, power.batteries.len());
    assert_eq!(
      power.energy,
      power
        .batteries
        .iter()
        .map(|battery| battery.energy)
        .sum::<u32>()
    );
    assert!(power.batteries.iter().any(
      |battery| battery.owner == BatteryOwner::Node(NodeId::PowerPlant)
    ));
    assert!(power
      .batteries
      .iter()
      .any(|battery| battery.owner == BatteryOwner::Hauler(0)));
  }

  #[test]
  fn generator_builds_one_greedy_battery_backed_power_line() {
    let mut state = GameState::new(ContentDatabase::starter(), POWER_LINE_SCENARIO).unwrap();

    let first = state.step();
    assert_eq!(
      std::collections::BTreeSet::from([
        GridPosition { x: 1, y: 1 },
        GridPosition { x: 2, y: 1 },
        GridPosition { x: 3, y: 1 },
      ]),
      first.topology.power_lines
    );
    assert!(first
      .events
      .iter()
      .any(|event| event.starts_with("power line built 3 cells")));
    for position in &first.topology.power_lines {
      assert!(first.power.as_ref().unwrap().batteries.iter().any(
        |battery| battery.owner == BatteryOwner::PowerLine(*position)
      ));
    }
    assert!(
      first
        .power
        .as_ref()
        .unwrap()
        .batteries
        .iter()
        .find(|battery| battery.owner == BatteryOwner::Node(NodeId::Factory(0)))
        .unwrap()
        .energy
        > 0
    );

    let second = state.step();
    assert_eq!(first.topology.power_lines, second.topology.power_lines);
    assert!(second
      .events
      .iter()
      .all(|event| !event.starts_with("power line built")));
  }

  #[test]
  fn retrieve_and_deploy_activates_a_source_through_the_mutation_queue() {
    let mut state = GameState::new(ContentDatabase::starter(), DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let snapshots: Vec<_> = (0..8).map(|_| state.step()).collect();

    let deploy_tick = snapshots
      .iter()
      .position(|snapshot| {
        snapshot
          .events
          .iter()
          .any(|event| event.contains("world deploy mining drill"))
      })
      .expect("drill deployment occurs");
    assert!(snapshots[..deploy_tick]
      .iter()
      .all(|snapshot| snapshot.sources[0].stockpile.items.is_empty()));
    assert!(snapshots
      .iter()
      .any(|snapshot| snapshot.events.iter().any(|event| {
        event.contains("dispatch retrieve") && event.contains("factory")
      })));
    assert!(snapshots
      .iter()
      .any(|snapshot| snapshot.events.iter().any(|event| {
        event.contains("dispatch deploy") && event.contains("queued")
      })));
    assert!(snapshots
      .iter()
      .any(|snapshot| snapshot.events.iter().any(|event| {
        event.contains("world deploy mining drill")
      })));
    assert!(snapshots.last().expect("run has snapshots").sources[0].deployed);
    assert!(
      snapshots
        .last()
        .expect("run has snapshots")
        .sources[0]
        .stockpile
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    );
    assert_eq!(1, state.metrics().deployments);
    assert_eq!(
      0,
      snapshots
        .last()
        .expect("run has snapshots")
        .factories[0]
        .inventory
        .items
        .get(MINING_DRILL.as_str())
        .copied()
        .unwrap_or(0)
    );
  }

  #[test]
  fn spawnable_factory_inventory_constructs_a_blocking_world_structure() {
    use factory_content::{BUILDING_DEPLOYMENT_SCENARIO, STORAGE_WAREHOUSE};

    let mut state =
      GameState::new(ContentDatabase::starter(), BUILDING_DEPLOYMENT_SCENARIO).unwrap();
    let initial = state.snapshot(Vec::new());
    let site_position = GridPosition { x: 4, y: 1 };

    assert!(initial
      .topology
      .nodes
      .iter()
      .any(|node| node.id == NodeId::BuildSite(0) && node.position == site_position));
    assert!(!initial.topology.blocked.contains(&site_position));
    assert!(initial.structures.is_empty());

    let snapshots = (0..16).map(|_| state.step()).collect::<Vec<_>>();
    let built = snapshots
      .iter()
      .find(|snapshot| {
        snapshot
          .events
          .iter()
          .any(|event| event == "world spawn storage_warehouse at structure-0")
      })
      .expect("warehouse construction completes");

    assert!(snapshots.iter().any(|snapshot| snapshot.events.iter().any(
      |event| event.contains("dispatch retrieve") && event.contains("factory")
    )));
    assert_eq!(
      vec![StructureSnapshot {
        node: NodeId::Structure(0),
        item: STORAGE_WAREHOUSE,
        alerts: AlertHistory::default(),
      }],
      built.structures
    );
    assert!(built.topology.blocked.contains(&site_position));
    assert!(built
      .topology
      .nodes
      .iter()
      .any(|node| node.id == NodeId::Structure(0) && node.position == site_position));
    assert!(!built
      .topology
      .nodes
      .iter()
      .any(|node| node.id == NodeId::BuildSite(0)));
    assert_eq!(1, state.metrics().deployments);
    assert_eq!(1, state.world.structures.len());
    assert_eq!(
      0,
      state.world.factories[0]
        .production
        .inventory
        .count(STORAGE_WAREHOUSE)
    );
  }

  #[test]
  fn deployment_scenario_is_deterministic() {
    let mut first = GameState::new(ContentDatabase::starter(), DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let mut second = GameState::new(ContentDatabase::starter(), DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let first_run: Vec<_> = (0..12).map(|_| first.step()).collect();
    let second_run: Vec<_> = (0..12).map(|_| second.step()).collect();
    assert_eq!(first_run, second_run);
    assert_eq!(first.metrics(), second.metrics());
  }

  #[test]
  fn depleted_ore_and_empty_drill_teardown_at_world_boundaries() {
    let mut state = GameState::new(ContentDatabase::starter(), DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let snapshots = (0..64).map(|_| state.step()).collect::<Vec<_>>();
    let deposit_delete_tick = snapshots
      .iter()
      .position(|snapshot| {
        snapshot
          .events
          .iter()
          .any(|event| event.contains("delete depleted ore"))
      })
      .expect("depleted ore is deleted");
    let drill_delete_tick = snapshots
      .iter()
      .position(|snapshot| {
        snapshot
          .events
          .iter()
          .any(|event| event.contains("delete depleted mining drill"))
      })
      .expect("empty drill is deleted");
    let final_snapshot = snapshots.last().expect("run has snapshots");

    assert!(deposit_delete_tick < drill_delete_tick);
    assert!(final_snapshot.sources[0].exhausted);
    assert!(!final_snapshot.sources[0].deployed);
    assert!(final_snapshot.sources[0].stockpile.items.is_empty());
    assert_eq!(2, state.metrics().world_deletions);
    assert!(!final_snapshot
      .topology
      .blocked
      .contains(&final_snapshot.topology.nodes[0].position));
  }

  #[test]
  fn pathfinding_scenario_routes_around_the_blocked_cell() {
    let mut state = GameState::new(ContentDatabase::starter(), PATHFINDING_DEMO_SCENARIO).unwrap();
    let first = state.step();
    let second = state.step();
    let third = state.step();
    let fourth = state.step();

    assert_eq!(NodeId::Road, first.haulers[0].position);
    assert_eq!(
      NodeId::Transit(GridPosition { x: 2, y: 1 }),
      second.haulers[0].position
    );
    assert_eq!(
      NodeId::Transit(GridPosition { x: 2, y: 1 }),
      third.haulers[0].position
    );
    assert!(third.events.iter().any(|event| event.contains("deliver 3")));
    assert_ne!(NodeId::Factory(0), fourth.haulers[0].position);
    assert_eq!(
      1,
      first.topology.obstacles.len(),
      "the occupied cell is exposed to projections"
    );
  }

  #[test]
  fn movement_queue_applies_after_deterministic_transit_arbitration() {
    let mut content = ContentDatabase::starter();
    content
      .scenarios
      .get_mut(&PATHFINDING_DEMO_SCENARIO)
      .expect("pathfinding scenario exists")
      .hauler_count = 2;
    let mut state = GameState::new(content, PATHFINDING_DEMO_SCENARIO).unwrap();
    for hauler in &mut state.world.haulers {
      hauler.position = NodeId::Road;
      hauler.assign(DispatchAssignment {
        item: IRON_ORE,
        source: NodeId::Source(0),
        destination: NodeId::Factory(0),
        phase: DispatchPhase::Deliver,
        priority: DispatchPriority::NORMAL,
      });
    }
    let mut events = Vec::new();

    state.queue_hauler_movements(&mut events);

    assert!(state
      .world
      .haulers
      .iter()
      .all(|hauler| hauler.position == NodeId::Road));
    assert_eq!(1, state.world.queued_mutations.len());
    assert!(events
      .iter()
      .any(|event| event.contains("hauler-1 blocked by queued transit occupancy")));

    state.apply_world_mutations(&mut events);

    assert_eq!(
      NodeId::Transit(GridPosition { x: 2, y: 1 }),
      state.world.haulers[0].position
    );
    assert_eq!(NodeId::Road, state.world.haulers[1].position);
    assert!(state.world.queued_mutations.is_empty());
  }
}
