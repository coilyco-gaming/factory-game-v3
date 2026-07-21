mod dispatch;
mod metrics;
mod mining;
mod production;
mod resources;
mod world;

use factory_content::{ContentDatabase, ItemId, ScenarioId, IRON_BARS_SCENARIO};
use std::fmt;

pub use dispatch::{
  DispatchAssignment, DispatchBoard, DispatchIntent, DispatchPhase, DispatchReceiverState,
  DispatchVerb,
};
pub use metrics::{RunMetrics, RunMetricsSnapshot};
pub use mining::{Deposit, MiningExtractor};
pub use production::{CraftSnapshot, FactoryProduction, RecipeRuntime};
pub use resources::{Inventory, InventoryError, InventorySnapshot};
pub use world::{
  FactoryNode, FactorySnapshot, GridPosition, Hauler, HaulerSnapshot, NodeId,
  ScenarioSnapshot, SourceNode, SourceSnapshot, TickSnapshot, Topology,
  TopologyNode, TopologySnapshot, WorldState,
};

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
    let production = FactoryProduction::new(factory_inventory, recipe);

    Ok(Self {
      world: WorldState {
        tick: 0,
        source: SourceNode::new(
          Inventory::new(1024, 1024),
          scenario.source_item,
          MiningExtractor::for_item(
            &content,
            scenario.source_item,
            scenario.mining_speed,
            scenario.source_deposit,
          ),
        ),
        hauler: Hauler::new(
          Inventory::new(32, 32),
          NodeId::Source,
          scenario.hauler_capacity,
          NodeId::Source,
        ),
        factory: FactoryNode::new(production, scenario.craft_input_buffer),
        topology: Topology::starter(),
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
    let source = &mut self.world.source;
    let mined = source.mining.advance(&self.content, &mut source.stockpile);
    self.metrics.record_mined(source.mining.item, mined);
    if mined > 0 {
      match source.mining.deposit {
        Deposit::Finite(remaining) => events.push(format!(
          "mine {} +{} deposit {}",
          source.mining.item, mined, remaining
        )),
        Deposit::Manifest => {
          events.push(format!("mine {} +{} manifest", source.mining.item, mined))
        }
      }
    }
  }

  fn refresh_dispatch_intents(&mut self) {
    self.world.source.refresh_dispatch(NodeId::Factory);
    self.world.factory.refresh_dispatch(NodeId::Source);
  }

  fn assign_dispatch(&mut self, events: &mut Vec<String>) {
    if !matches!(self.world.hauler.dispatch, DispatchReceiverState::Unassigned) {
      return;
    }
    if self.world.hauler.position != NodeId::Source || !self.world.hauler.cargo.is_empty() {
      return;
    }
    let source_intent = match &self.world.source.dispatch.intent {
      Some(intent) => intent,
      None => return,
    };
    let factory_intent = match &self.world.factory.dispatch.intent {
      Some(intent) => intent,
      None => return,
    };
    if source_intent.item != factory_intent.item {
      return;
    }
    let assignment = DispatchAssignment::collect(
      source_intent.item,
      source_intent.from,
      source_intent.to,
    );
    self.world.hauler.assign(assignment);
    self.metrics.dispatches_assigned += 1;
    events.push(format!(
      "dispatch assigned {} {} {} -> {}",
      DispatchVerb::Collect,
      source_intent.item,
      source_intent.from,
      source_intent.to
    ));
  }

  fn collect(&mut self, events: &mut Vec<String>) {
    let position = self.world.hauler.position;
    let assignment = match self.world.hauler.dispatch.clone() {
      DispatchReceiverState::Assigned(assignment) => assignment,
      DispatchReceiverState::Unassigned => return,
    };
    if assignment.phase != DispatchPhase::Collect || position != NodeId::Source {
      return;
    };
    let moved = self.world.source.stockpile.transfer_up_to(
      &self.content,
      &mut self.world.hauler.cargo,
      assignment.item,
      self.world.hauler.carry_limit,
    );
    self.metrics.units_collected += moved;
    if moved > 0 {
      self.world.hauler.dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..assignment
      });
      events.push(format!(
        "dispatch collect {} from {} to {}",
        moved, assignment.source, assignment.destination
      ));
    }
  }

  fn deliver(&mut self, events: &mut Vec<String>) {
    let position = self.world.hauler.position;
    let assignment = match self.world.hauler.dispatch.clone() {
      DispatchReceiverState::Assigned(assignment) => assignment,
      DispatchReceiverState::Unassigned => return,
    };
    if assignment.phase != DispatchPhase::Deliver || position != NodeId::Factory {
      return;
    }
    let destination = assignment.destination;
    let carried = self.world.hauler.cargo.count(assignment.item);
    let delivered = self.world.hauler.cargo.transfer_up_to(
      &self.content,
      &mut self.world.factory.production.inventory,
      assignment.item,
      carried,
    );
    self.metrics.units_delivered += delivered;
    if delivered > 0 {
      self.world.hauler.clear_assignment();
      events.push(format!("dispatch deliver {} to {}", delivered, destination));
    }
  }

  fn advance_production(&mut self, events: &mut Vec<String>) {
    let produced = self
      .world
      .factory
      .production
      .advance(&self.content, events);
    let output_item = self.world.factory.production.recipe.output_item;
    self.metrics.record_crafted(output_item, produced);
  }

  fn move_hauler(&mut self, events: &mut Vec<String>) {
    let target = match &self.world.hauler.dispatch {
      DispatchReceiverState::Assigned(assignment) => match assignment.phase {
        DispatchPhase::Collect => assignment.source,
        DispatchPhase::Deliver => assignment.destination,
      },
      DispatchReceiverState::Unassigned => {
        if self.world.hauler.cargo.is_empty() {
          NodeId::Source
        } else {
          NodeId::Factory
        }
      }
    };
    self.world.hauler.set_target(target);
    let current = self.world.hauler.position;
    let next = self.world.topology.step_toward(current, target);
    self.world.hauler.position = next;
    self.world.hauler.set_route_index(&self.world.topology);
    if next != current {
      events.push(format!(
        "move hauler {} -> {} toward {}",
        current, next, target
      ));
    }
  }

  pub fn step(&mut self) -> TickSnapshot {
    self.world.tick += 1;
    let mut events = Vec::new();
    self.advance_mining(&mut events);
    self.refresh_dispatch_intents();
    self.assign_dispatch(&mut events);
    self.collect(&mut events);
    self.deliver(&mut events);
    self.advance_production(&mut events);
    self.move_hauler(&mut events);
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
        nodes: self.world.topology.nodes,
        route: self.world.topology.route,
      },
      source: SourceSnapshot {
        item: self.world.source.item,
        stockpile: self.world.source.stockpile.snapshot(),
        mining: self.world.source.mining.clone(),
        dispatch: self.world.source.dispatch.clone(),
      },
      hauler: HaulerSnapshot {
        position: self.world.hauler.position,
        position_grid: self.world.topology.position(self.world.hauler.position),
        target: self.world.hauler.target,
        target_grid: self.world.topology.position(self.world.hauler.target),
        route_index: self.world.hauler.route_index,
        cargo: self.world.hauler.cargo.snapshot(),
        carry_limit: self.world.hauler.carry_limit,
        dispatch: self.world.hauler.dispatch.clone(),
      },
      factory: FactorySnapshot {
        inventory: self.world.factory.production.inventory.snapshot(),
        craft: self.world.factory.production.craft_snapshot(),
        dispatch: self.world.factory.dispatch.clone(),
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
    ContentDatabase, COPPER_BARS, COPPER_ORE, IRON_BARS, IRON_BARS_SCENARIO, IRON_ORE, STONE,
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

    assert_eq!(Deposit::Finite(6), snapshots[0].source.mining.deposit);
    assert_eq!(Deposit::Finite(0), snapshots[2].source.mining.deposit);
    assert!(snapshots[0].events.iter().any(|event| event.starts_with("mine")));
    assert!(snapshots[3..]
      .iter()
      .all(|snapshot| snapshot.events.iter().all(|event| !event.starts_with("mine"))));
  }

  #[test]
  fn run_metrics_summarize_a_fixed_iron_bars_run() {
    let mut state = GameState::starter_iron_bars();
    for _ in 0..6 {
      state.step();
    }
    let metrics = state.metrics();

    assert_eq!(6, metrics.ticks);
    assert_eq!(9, metrics.mined.get(IRON_ORE.as_str()).copied().unwrap_or(0));
    assert_eq!(
      10,
      metrics.crafted.get(IRON_BARS.as_str()).copied().unwrap_or(0)
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
      first.hauler.dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Road, first.hauler.position);
    assert_eq!(3, first.hauler.cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0));
    assert_eq!(NodeId::Factory, first.hauler.target);

    let second = state.step();
    assert!(matches!(
      second.hauler.dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Factory, second.hauler.position);
    assert_eq!(3, second.hauler.cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0));
    assert_eq!(2, second.hauler.route_index);
    assert!(!second.factory.craft.crafting);

    let third = state.step();
    assert!(matches!(third.hauler.dispatch, DispatchReceiverState::Unassigned));
    assert_eq!(NodeId::Road, third.hauler.position);
    assert_eq!(0, third.hauler.cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0));
    assert!(third.factory.craft.crafting);
    assert_eq!(NodeId::Source, third.hauler.target);
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
    assert_eq!(NodeId::Road, first.hauler.position);
    assert!(first
      .events
      .iter()
      .any(|event| event.contains("dispatch collect")));
    assert!(!first.events.iter().any(|event| event.contains("dispatch deliver")));

    let second = state.step();
    assert_eq!(NodeId::Factory, second.hauler.position);
    assert!(second
      .events
      .iter()
      .all(|event| !event.contains("dispatch deliver")));

    let third = state.step();
    assert_eq!(NodeId::Road, third.hauler.position);
    assert!(third
      .events
      .iter()
      .any(|event| event.contains("dispatch deliver")));
  }

  #[test]
  fn collect_and_deliver_require_the_correct_node() {
    let mut state = GameState::new(ContentDatabase::starter(), IRON_BARS_SCENARIO).unwrap();
    state.world.hauler.position = NodeId::Road;
    state.world.hauler.dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source,
      destination: NodeId::Factory,
      phase: DispatchPhase::Collect,
    });

    let collect_snapshot = state.step();
    assert_eq!(0, collect_snapshot.hauler.cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0));
    assert!(collect_snapshot
      .events
      .iter()
      .all(|event| !event.contains("dispatch collect")));
    assert_eq!(NodeId::Source, collect_snapshot.hauler.target);

    state.world.hauler.position = NodeId::Road;
    state.world.hauler.cargo.insert_exact(&ContentDatabase::starter(), IRON_ORE, 3).unwrap();
    state.world.hauler.dispatch = DispatchReceiverState::Assigned(DispatchAssignment {
      item: IRON_ORE,
      source: NodeId::Source,
      destination: NodeId::Factory,
      phase: DispatchPhase::Deliver,
    });

    let deliver_snapshot = state.step();
    assert_eq!(
      3,
      deliver_snapshot
        .hauler
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
    assert_eq!(NodeId::Factory, deliver_snapshot.hauler.target);
  }
}
