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
  ScenarioMissingSources(ScenarioId),
  RecipeMissingIngredients(ItemId),
}

impl fmt::Display for SimulationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownScenario(id) => write!(f, "unknown scenario: {id}"),
      Self::ScenarioMissingSources(id) => {
        write!(f, "scenario {id} must define at least one source")
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

    let mut factory_inventory = Inventory::new(32, 32);
    for input_item in recipe.inputs.keys() {
      factory_inventory.reserve(*input_item, scenario.craft_input_buffer);
    }
    factory_inventory.reserve(recipe.output_item, scenario.craft_output_buffer);
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
        )
      })
      .collect();
    let haulers = (0..scenario.hauler_count)
      .map(|index| {
        Hauler::new(
          index as u8,
          Inventory::new(32, 32),
          NodeId::Source(0),
          scenario.hauler_capacity,
        )
      })
      .collect();

    Ok(Self {
      world: WorldState {
        tick: 0,
        sources,
        haulers,
        factory: FactoryNode::new(production, scenario.craft_input_buffer),
        topology: Topology::for_sources(scenario.sources.len() as u8),
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
    for source in &mut self.world.sources {
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
    for source in &mut self.world.sources {
      source.refresh_dispatch(NodeId::Factory);
    }
    self.world.factory.refresh_dispatch();
  }

  // Demand minus in-flight cargo goes to unassigned haulers in index
  // order (collect phase counts at carry limit): never double-served.
  fn assign_dispatch(&mut self, events: &mut Vec<String>) {
    let items: Vec<ItemId> = self
      .world
      .factory
      .dispatch
      .intents
      .iter()
      .map(|intent| intent.item)
      .collect();
    for item in items {
      self.assign_dispatch_for_item(item, events);
    }
  }

  fn assign_dispatch_for_item(&mut self, item: ItemId, events: &mut Vec<String>) {
    let mut need = self
      .world
      .factory
      .input_buffer
      .saturating_sub(self.world.factory.production.inventory.count(item));
    for hauler in &self.world.haulers {
      if let DispatchReceiverState::Assigned(assignment) = &hauler.dispatch {
        if assignment.item == item {
          let in_flight = match assignment.phase {
            DispatchPhase::Collect => hauler.carry_limit,
            DispatchPhase::Deliver => hauler.cargo.count(item),
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
      if !matches!(hauler.dispatch, DispatchReceiverState::Unassigned) || !hauler.cargo.is_empty()
      {
        continue;
      }
      let source_node = self.world.sources.iter().find_map(|source| {
        source
          .dispatch
          .intents
          .iter()
          .any(|intent| intent.item == item)
          .then_some(source.node)
      });
      let source_node = match source_node {
        Some(node) => node,
        None => break,
      };
      let hauler = &mut self.world.haulers[hauler_index];
      hauler.assign(DispatchAssignment::collect(item, source_node, NodeId::Factory));
      self.metrics.dispatches_assigned += 1;
      need = need.saturating_sub(hauler.carry_limit);
      events.push(format!(
        "dispatch assigned collect {} {} -> {} to hauler-{}",
        item,
        source_node,
        NodeId::Factory,
        hauler.id
      ));
    }
  }

  fn collect(&mut self, events: &mut Vec<String>) {
    for hauler_index in 0..self.world.haulers.len() {
      let (assignment, position) = match &self.world.haulers[hauler_index].dispatch {
        DispatchReceiverState::Assigned(assignment) => {
          (assignment.clone(), self.world.haulers[hauler_index].position)
        }
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
        DispatchReceiverState::Assigned(assignment) => {
          (assignment.clone(), self.world.haulers[hauler_index].position)
        }
        DispatchReceiverState::Unassigned => continue,
      };
      if assignment.phase != DispatchPhase::Deliver || position != NodeId::Factory {
        continue;
      }
      let carried = self.world.haulers[hauler_index].cargo.count(assignment.item);
      let delivered = self.world.haulers[hauler_index].cargo.transfer_up_to(
        &self.content,
        &mut self.world.factory.production.inventory,
        assignment.item,
        carried,
      );
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

  fn advance_production(&mut self, events: &mut Vec<String>) {
    let produced = self
      .world
      .factory
      .production
      .advance(&self.content, events);
    let output_item = self.world.factory.production.recipe.output_item;
    self.metrics.record_crafted(output_item, produced);
  }

  fn move_haulers(&mut self, events: &mut Vec<String>) {
    for hauler in &mut self.world.haulers {
      let target = match &hauler.dispatch {
        DispatchReceiverState::Assigned(assignment) => match assignment.phase {
          DispatchPhase::Collect => assignment.source,
          DispatchPhase::Deliver => assignment.destination,
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
      let next = self.world.topology.step_toward(current, target);
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
    self.advance_mining(&mut events);
    self.refresh_dispatch_intents();
    self.assign_dispatch(&mut events);
    self.collect(&mut events);
    self.deliver(&mut events);
    self.advance_production(&mut events);
    self.move_haulers(&mut events);
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
        nodes: self.world.topology.nodes.clone(),
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
    ContentDatabase, BUILDING_MATERIALS, BUILDING_MATERIALS_SCENARIO, COPPER_BARS, COPPER_ORE,
    IRON_BARS, IRON_BARS_FLEET_SCENARIO, IRON_BARS_SCENARIO, IRON_ORE, STONE,
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
      first.haulers[0].dispatch,
      DispatchReceiverState::Assigned(DispatchAssignment {
        phase: DispatchPhase::Deliver,
        ..
      })
    ));
    assert_eq!(NodeId::Road, first.haulers[0].position);
    assert_eq!(
      3,
      first.haulers[0].cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0)
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
      second.haulers[0].cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0)
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
      third.haulers[0].cargo.items.get(IRON_ORE.as_str()).copied().unwrap_or(0)
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
    assert!(!first.events.iter().any(|event| event.contains("dispatch deliver")));

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
    let mut state =
      GameState::new(ContentDatabase::starter(), IRON_BARS_FLEET_SCENARIO).unwrap();

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
    let mut state =
      GameState::new(ContentDatabase::starter(), IRON_BARS_FLEET_SCENARIO).unwrap();

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
}
