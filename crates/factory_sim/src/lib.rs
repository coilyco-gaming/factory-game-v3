mod dispatch;
mod metrics;
mod mining;
mod power;
mod production;
mod resources;
mod world;

use factory_content::{
  ContentDatabase, ItemId, ScenarioId, IRON_BARS_SCENARIO, MINING_DRILL,
};
use std::fmt;

pub use dispatch::{
  DispatchAssignment, DispatchBoard, DispatchIntent, DispatchPhase, DispatchReceiverState,
  DispatchVerb,
};
pub use metrics::{RunMetrics, RunMetricsSnapshot};
pub use mining::{Deposit, MiningExtractor};
pub use power::{PowerPlant, PowerSnapshot};
pub use production::{CraftSnapshot, FactoryProduction, RecipeRuntime};
pub use resources::{Inventory, InventoryError, InventorySnapshot};
pub use world::{
  FactoryNode, FactorySnapshot, GridPosition, Hauler, HaulerSnapshot, NodeId, ScenarioSnapshot,
  SourceNode, SourceSnapshot, TickSnapshot, Topology, TopologyNode, TopologySnapshot,
  WorldMutation, WorldState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimulationError {
  UnknownScenario(ScenarioId),
  ScenarioMissingSources(ScenarioId),
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
  metrics: RunMetrics,
}

impl GameState {
  pub fn new(content: ContentDatabase, scenario_id: ScenarioId) -> Result<Self, SimulationError> {
    let scenario = content
      .scenarios
      .get(&scenario_id)
      .cloned()
      .ok_or(SimulationError::UnknownScenario(scenario_id))?;
    if scenario.sources.is_empty() {
      return Err(SimulationError::ScenarioMissingSources(scenario_id));
    }
    if scenario.sources.len() != scenario.layout.source_positions.len()
      || scenario.power.is_some() != scenario.layout.power_plant_position.is_some()
    {
      return Err(SimulationError::ScenarioLayoutMismatch(scenario_id));
    }
    let product = content.item(scenario.product_item).clone();
    if product.ingredients.is_empty() {
      return Err(SimulationError::RecipeMissingIngredients(product.id));
    }
    let recipe = RecipeRuntime {
      inputs: product.ingredients.clone(),
      output_item: product.id,
      output_quantity: product.craft_output,
      craft_time: product.craft_time.max(1),
    };

    let mut factory_inventory = Inventory::new(1024, 1024);
    for input_item in recipe.inputs.keys() {
      factory_inventory.reserve(*input_item, scenario.craft_input_buffer);
    }
    factory_inventory.reserve(recipe.output_item, scenario.craft_output_buffer);
    for (item, quantity) in &scenario.factory_starting_items {
      factory_inventory.reserve(*item, *quantity);
      factory_inventory
        .insert_exact(&content, *item, *quantity)
        .expect("scenario starting inventory fits the factory");
    }
    let production = FactoryProduction::new(factory_inventory, recipe);

    let sources = scenario
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
    let haulers = (0..scenario.hauler_count)
      .map(|index| {
        Hauler::new(
          index as u8,
          Inventory::new(
            scenario.hauler_weight_capacity,
            scenario.hauler_volume_capacity,
          ),
          NodeId::Source(0),
          scenario.hauler_capacity,
        )
      })
      .collect();

    let power = scenario
      .power
      .clone()
      .map(|spec| PowerPlant::new(&content, spec, Inventory::new(64, 64)));

    Ok(Self {
      world: WorldState {
        tick: 0,
        sources,
        haulers,
        factory: FactoryNode::new(production, scenario.craft_input_buffer),
        power,
        topology: Topology::from_layout(&scenario.layout),
        queued_mutations: Vec::new(),
        scenario,
      },
      content,
      metrics: RunMetrics::default(),
    })
  }

  pub fn starter_iron_bars() -> Self {
    Self::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).expect("starter scenario is valid")
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
      let consumer = format!("mining-{}", self.world.sources[source_index].node);
      if !self.consume_power(mining_cost, &consumer, events) {
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
      }
    }
  }

  fn refresh_dispatch_intents(&mut self) {
    let power_fuel = self.world.power.as_ref().map(|power| power.spec.fuel_item);
    for source in &mut self.world.sources {
      let destination = if Some(source.item) == power_fuel {
        NodeId::PowerPlant
      } else {
        NodeId::Factory
      };
      source.refresh_dispatch(destination);
    }
    self.world.factory.refresh_dispatch();
    for source in self.world.sources.iter().filter(|source| !source.deployed) {
      if self
        .world
        .factory
        .production
        .inventory
        .count(MINING_DRILL)
        > 0
      {
        self
          .world
          .factory
          .dispatch
          .intents
          .push(DispatchIntent::retrieve(
            MINING_DRILL,
            NodeId::Factory,
            source.node,
          ));
      }
    }
    if let Some(power) = &mut self.world.power {
      power.refresh_dispatch();
    }
  }

  // Demand minus in-flight cargo goes to unassigned haulers in index
  // order (collect phase counts at carry limit): never double-served.
  fn assign_dispatch(&mut self, events: &mut Vec<String>) {
    let demands: Vec<(ItemId, NodeId, u32, u32)> = self
      .world
      .factory
      .dispatch
      .intents
      .iter()
      .filter(|intent| intent.verb == DispatchVerb::Deliver)
      .map(|intent| {
        (
          intent.item,
          NodeId::Factory,
          self.world.factory.input_buffer,
          self.world.factory.production.inventory.count(intent.item),
        )
      })
      .collect();
    let mut demands = demands;
    if let Some(power) = &self.world.power {
      demands.extend(power.dispatch.intents.iter().map(|intent| {
        (
          intent.item,
          NodeId::PowerPlant,
          power.spec.fuel_buffer,
          power.fuel.count(intent.item),
        )
      }));
    }
    for (item, destination, buffer, stocked) in demands {
      self.assign_dispatch_for_demand(item, destination, buffer, stocked, events);
    }
    self.assign_deployments(events);
  }

  fn assign_deployments(&mut self, events: &mut Vec<String>) {
    let intents = self
      .world
      .factory
      .dispatch
      .intents
      .iter()
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
      if !self.consume_power(dispatch_cost, "dispatch-deploy", events) {
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
      let source_node = self.world.sources.iter().find_map(|source| {
        source
          .dispatch
          .intents
          .iter()
          .any(|intent| intent.item == item && intent.to == destination)
          .then_some(source.node)
      });
      let source_node = match source_node {
        Some(node) => node,
        None => break,
      };
      let dispatch_cost = self
        .world
        .power
        .as_ref()
        .map_or(0, |power| power.spec.dispatch_cost);
      if !self.consume_power(dispatch_cost, "dispatch", events) {
        break;
      }
      let hauler = &mut self.world.haulers[hauler_index];
      hauler.assign(DispatchAssignment::collect(item, source_node, destination));
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

  fn collect(&mut self, events: &mut Vec<String>) {
    for hauler_index in 0..self.world.haulers.len() {
      let (assignment, position) = match &self.world.haulers[hauler_index].dispatch {
        DispatchReceiverState::Assigned(assignment) => (
          assignment.clone(),
          self.world.haulers[hauler_index].position,
        ),
        DispatchReceiverState::Unassigned => continue,
      };
      if assignment.phase != DispatchPhase::Collect || position != assignment.source {
        continue;
      }
      let source = match self
        .world
        .sources
        .iter_mut()
        .find(|source| source.node == assignment.source)
      {
        Some(source) => source,
        None => continue,
      };
      let carry_limit = self.world.haulers[hauler_index].carry_limit;
      let moved = source.stockpile.transfer_up_to(
        &self.content,
        &mut self.world.haulers[hauler_index].cargo,
        assignment.item,
        carry_limit,
      );
      self.metrics.units_collected += moved;
      if moved > 0 {
        let hauler = &mut self.world.haulers[hauler_index];
        hauler.dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
          phase: DispatchPhase::Deliver,
          ..assignment
        });
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
      if assignment.phase != DispatchPhase::Deliver || position != assignment.destination {
        continue;
      }
      let carried = self.world.haulers[hauler_index]
        .cargo
        .count(assignment.item);
      let delivered = match assignment.destination {
        NodeId::Factory => self.world.haulers[hauler_index].cargo.transfer_up_to(
          &self.content,
          &mut self.world.factory.production.inventory,
          assignment.item,
          carried,
        ),
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
        NodeId::Source(_) | NodeId::Road | NodeId::Transit(_) => 0,
      };
      self.metrics.units_delivered += delivered;
      if delivered > 0 {
        let hauler = &mut self.world.haulers[hauler_index];
        hauler.clear_assignment();
        events.push(format!(
          "dispatch deliver {} to {} by hauler-{}",
          delivered, assignment.destination, hauler.id
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
        DispatchPhase::Retrieve if position == assignment.source => {
          let moved = self.world.factory.production.inventory.transfer_up_to(
            &self.content,
            &mut self.world.haulers[hauler_index].cargo,
            assignment.item,
            1,
          );
          if moved > 0 {
            self.world.haulers[hauler_index].dispatch =
              DispatchReceiverState::Assigned(DispatchAssignment {
                phase: DispatchPhase::Deploy,
                ..assignment
              });
            events.push(format!(
              "dispatch retrieve {} from factory by hauler-{}",
              moved, self.world.haulers[hauler_index].id
            ));
          }
        }
        DispatchPhase::Deploy if position == assignment.destination => {
          if self.world.haulers[hauler_index]
            .cargo
            .remove_exact(assignment.item, 1)
            .is_ok()
          {
            let NodeId::Source(source_index) = assignment.destination else {
              continue;
            };
            self
              .world
              .queued_mutations
              .push(WorldMutation::DeploySource(source_index));
            self.world.haulers[hauler_index].clear_assignment();
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
      }
    }
  }

  fn advance_production(&mut self, events: &mut Vec<String>) {
    let production_cost = self
      .world
      .power
      .as_ref()
      .map_or(0, |power| power.spec.production_cost);
    if self.world.factory.production.wants_power()
      && !self.consume_power(production_cost, "factory", events)
    {
      return;
    }
    let produced = self.world.factory.production.advance(&self.content, events);
    let output_item = self.world.factory.production.recipe.output_item;
    self.metrics.record_crafted(output_item, produced);
  }

  fn advance_power(&mut self, events: &mut Vec<String>) {
    let Some(power) = &mut self.world.power else {
      return;
    };
    let (burned, generated) = power.generate(events);
    self.metrics.fuel_burned += burned;
    self.metrics.energy_generated += generated;
  }

  fn consume_power(&mut self, amount: u32, consumer: &str, events: &mut Vec<String>) -> bool {
    let Some(power) = &mut self.world.power else {
      return true;
    };
    if power.consume(amount, consumer, events) {
      self.metrics.energy_consumed += amount;
      true
    } else {
      self.metrics.power_starvations += 1;
      false
    }
  }

  fn move_haulers(&mut self, events: &mut Vec<String>) {
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
            NodeId::Factory
          }
        }
      };
      hauler.set_target(target);
      let current = hauler.position;
      let Some(next) = self.world.topology.step_toward(current, target) else {
        events.push(format!(
          "move hauler-{} no available path {} -> {}",
          hauler.id, current, target
        ));
        continue;
      };
      hauler.position = next;
      if next != current {
        events.push(format!(
          "move hauler-{} {} -> {} toward {}",
          hauler.id, current, next, target
        ));
      }
    }
  }

  pub fn step(&mut self) -> TickSnapshot {
    self.world.tick += 1;
    let mut events = Vec::new();
    self.advance_power(&mut events);
    self.advance_mining(&mut events);
    self.refresh_dispatch_intents();
    self.assign_dispatch(&mut events);
    self.collect(&mut events);
    self.deliver(&mut events);
    self.retrieve_and_deploy(&mut events);
    self.advance_production(&mut events);
    self.move_haulers(&mut events);
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
        })
        .collect(),
      factory: FactorySnapshot {
        inventory: self.world.factory.production.inventory.snapshot(),
        craft: self.world.factory.production.craft_snapshot(),
        dispatch: self.world.factory.dispatch.clone(),
      },
      power: self.world.power.as_ref().map(PowerPlant::snapshot),
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
    ContentDatabase, BUILDING_MATERIALS, BUILDING_MATERIALS_SCENARIO, COAL, COPPER_BARS,
    COPPER_ORE, DEPLOYMENT_DEMO_SCENARIO, IRON_BARS, IRON_BARS_FLEET_SCENARIO,
    IRON_BARS_SCENARIO, IRON_ORE, MINING_DRILL, PATHFINDING_DEMO_SCENARIO,
    POWERED_IRONWORKS_SCENARIO, STONE,
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
      10,
      metrics
        .crafted
        .get(IRON_BARS.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert_eq!(2, metrics.dispatches_assigned);
    assert_eq!(6, metrics.units_collected);
    assert_eq!(3, metrics.units_delivered);
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
        phase: DispatchPhase::Deliver,
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
    assert_eq!(NodeId::Factory, first.haulers[0].target);

    let second = state.step();
    assert!(matches!(
      second.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Factory, second.haulers[0].position);
    assert_eq!(
      3,
      second.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(!second.factory.craft.crafting);

    let third = state.step();
    assert!(matches!(
      third.haulers[0].dispatch,
      DispatchReceiverState::Unassigned
    ));
    assert_eq!(NodeId::Factory, third.haulers[0].position);
    assert_eq!(
      0,
      third.haulers[0]
        .cargo
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0)
    );
    assert!(third.factory.craft.crafting);

    let fourth = state.step();
    assert!(matches!(
      fourth.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Collect,
        ..
      })
    ));
    assert_eq!(NodeId::Road, fourth.haulers[0].position);
    assert_eq!(NodeId::Source(0), fourth.haulers[0].target);
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

  #[test]
  fn route_traversal_takes_multiple_ticks_before_delivery() {
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
    assert_eq!(NodeId::Factory, second.haulers[0].position);
    assert!(second
      .events
      .iter()
      .all(|event| !event.contains("dispatch deliver")));

    let third = state.step();
    assert!(third
      .events
      .iter()
      .any(|event| event.contains("dispatch deliver")));
  }

  #[test]
  fn collect_and_deliver_require_the_correct_node() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();
    state.world.haulers[0].position = NodeId::Road;
    state.world.haulers[0].dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source(0),
      destination: NodeId::Factory,
      phase: DispatchPhase::Collect,
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
    assert_eq!(NodeId::Source(0), collect_snapshot.haulers[0].target);

    state.world.haulers[0].position = NodeId::Road;
    state.world.haulers[0]
      .cargo
      .insert_exact(&ContentDatabase::starter(), IRON_ORE, 3)
      .unwrap();
    state.world.haulers[0].dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source(0),
      destination: NodeId::Factory,
      phase: DispatchPhase::Deliver,
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
    assert_eq!(NodeId::Factory, deliver_snapshot.haulers[0].target);
  }

  #[test]
  fn factory_advertises_one_intent_per_missing_input() {
    let mut state =
      GameState::new(ContentDatabase::starter(), BUILDING_MATERIALS_SCENARIO).unwrap();

    let first = state.step();
    let intent_items: Vec<&str> = first
      .factory
      .dispatch
      .intents
      .iter()
      .map(|intent| intent.item.as_str())
      .collect();
    assert_eq!(vec![IRON_ORE.as_str(), STONE.as_str()], intent_items);
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
    assert!(metrics.mined.get(IRON_ORE.as_str()).copied().unwrap_or(0) > 0);
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
        .factory
        .inventory
        .items
        .get(BUILDING_MATERIALS.as_str())
        .copied()
        .unwrap_or(0)
        > 0
    }));
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
        .factory
        .inventory
        .items
        .get(IRON_ORE.as_str())
        .copied()
        .unwrap_or(0);
      assert!(input_stock <= 6, "input stock {input_stock} exceeds buffer");
      bars_seen |= snapshot
        .factory
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
  fn retrieve_and_deploy_activates_a_source_through_the_mutation_queue() {
    let mut state = GameState::new(ContentDatabase::starter(), DEPLOYMENT_DEMO_SCENARIO).unwrap();
    let snapshots: Vec<_> = (0..8).map(|_| state.step()).collect();

    assert!(snapshots[..4]
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
        .factory
        .inventory
        .items
        .get(MINING_DRILL.as_str())
        .copied()
        .unwrap_or(0)
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
    assert_eq!(NodeId::Factory, third.haulers[0].position);
    assert!(fourth.events.iter().any(|event| event.contains("deliver 3")));
    assert_eq!(
      1,
      first.topology.obstacles.len(),
      "the occupied cell is exposed to projections"
    );
  }
}
