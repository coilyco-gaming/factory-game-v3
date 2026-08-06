use crate::alerts::AlertHistory;
use crate::dispatch::{DispatchBoard, DispatchIntent};
use crate::resources::{Inventory, InventorySnapshot};
use crate::{GridPosition, NodeId};
use factory_content::{ContentDatabase, GeneratorSpec, ItemId, PowerSpec};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryOwner {
  Node(NodeId),
  Hauler(u8),
  PowerLine(GridPosition),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Battery {
  pub owner: BatteryOwner,
  pub energy: u32,
  pub capacity: u32,
}

impl Battery {
  pub fn new(owner: BatteryOwner, energy: u32, capacity: u32) -> Self {
    let capacity = capacity.max(5);
    Self {
      owner,
      energy: energy.min(capacity),
      capacity,
    }
  }

  pub fn charge(&mut self, amount: u32) -> u32 {
    let accepted = amount.min(self.capacity.saturating_sub(self.energy));
    self.energy += accepted;
    accepted
  }

  pub fn consume(&mut self, amount: u32) -> bool {
    if self.energy < amount {
      return false;
    }
    self.energy -= amount;
    true
  }
}

#[derive(Clone, Debug)]
pub struct PowerGenerator {
  pub node: NodeId,
  pub fuel: Inventory,
  pub dispatch: DispatchBoard,
  pub spec: GeneratorSpec,
  pub alerts: AlertHistory,
}

impl PowerGenerator {
  pub fn new(
    content: &ContentDatabase,
    node: NodeId,
    spec: GeneratorSpec,
    mut fuel: Inventory,
  ) -> Self {
    if let Some(fuel_item) = spec.fuel_item {
      fuel.reserve(fuel_item, spec.fuel_buffer);
      fuel
        .insert_exact(content, fuel_item, spec.initial_fuel)
        .expect("starter generator fuel fits its reserved inventory");
    }
    Self {
      node,
      fuel,
      dispatch: DispatchBoard::new(),
      spec,
      alerts: AlertHistory::default(),
    }
  }

  pub fn refresh_dispatch(&mut self) {
    self.dispatch.intents = self
      .spec
      .fuel_item
      .filter(|_| self.spec.burn_rate > 0)
      .filter(|_| self.spec.gain_rate > 0)
      .filter(|fuel_item| self.fuel.count(*fuel_item) < self.spec.fuel_buffer)
      .map(|fuel_item| DispatchIntent::deliver(fuel_item, NodeId::Road, self.node))
      .into_iter()
      .collect();
  }

  pub fn generate(
    &mut self,
    battery: &mut Battery,
    events: &mut Vec<String>,
  ) -> (u32, u32) {
    if battery.energy >= battery.capacity || self.spec.gain_rate == 0 {
      return (0, 0);
    }
    let burned = if self.spec.burn_rate == 0 {
      0
    } else {
      let Some(fuel_item) = self.spec.fuel_item else {
        return (0, 0);
      };
      if self.fuel.count(fuel_item) < self.spec.burn_rate {
        return (0, 0);
      }
      self.fuel.remove_up_to(fuel_item, self.spec.burn_rate)
    };
    let generated = battery.charge(self.spec.gain_rate);
    events.push(format!(
      "power generate {} burned {} generated {} battery {}/{}",
      self.node, burned, generated, battery.energy, battery.capacity
    ));
    (burned, generated)
  }

  fn snapshot(&self, battery: &Battery) -> GeneratorSnapshot {
    GeneratorSnapshot {
      node: self.node,
      fuel_item: self.spec.fuel_item,
      fuel: self.fuel.snapshot(),
      energy: battery.energy,
      capacity: battery.capacity,
      burn_rate: self.spec.burn_rate,
      gain_rate: self.spec.gain_rate,
      dispatch: self.dispatch.clone(),
      alerts: self.alerts.clone(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct PowerGrid {
  pub generators: Vec<PowerGenerator>,
  pub spec: PowerSpec,
}

impl PowerGrid {
  pub fn new(content: &ContentDatabase, spec: PowerSpec) -> Self {
    let generators = spec
      .generators
      .iter()
      .cloned()
      .enumerate()
      .map(|(index, generator)| {
        PowerGenerator::new(
          content,
          NodeId::Generator(index as u8),
          generator,
          Inventory::new(64, 64),
        )
      })
      .collect();
    Self { generators, spec }
  }

  pub fn refresh_dispatch(&mut self) {
    for generator in &mut self.generators {
      generator.refresh_dispatch();
    }
  }

  pub fn generate(
    &mut self,
    batteries: &mut BTreeMap<BatteryOwner, Battery>,
    events: &mut Vec<String>,
  ) -> (u32, u32) {
    let mut burned = 0_u32;
    let mut generated = 0_u32;
    for generator in &mut self.generators {
      let battery = batteries
        .get_mut(&BatteryOwner::Node(generator.node))
        .expect("powered scenario has every generator battery");
      let (generator_burned, generator_output) = generator.generate(battery, events);
      burned = burned.saturating_add(generator_burned);
      generated = generated.saturating_add(generator_output);
    }
    (burned, generated)
  }

  pub fn snapshot(&self, batteries: impl Iterator<Item = Battery>) -> PowerSnapshot {
    let batteries = batteries.collect::<Vec<_>>();
    let generators = self
      .generators
      .iter()
      .map(|generator| {
        let battery = batteries
          .iter()
          .find(|battery| battery.owner == BatteryOwner::Node(generator.node))
          .expect("snapshot contains every generator battery");
        generator.snapshot(battery)
      })
      .collect();
    PowerSnapshot {
      energy: batteries.iter().map(|battery| battery.energy).sum(),
      capacity: batteries.iter().map(|battery| battery.capacity).sum(),
      generators,
      batteries,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratorSnapshot {
  pub node: NodeId,
  pub fuel_item: Option<ItemId>,
  pub fuel: InventorySnapshot,
  pub energy: u32,
  pub capacity: u32,
  pub burn_rate: u32,
  pub gain_rate: u32,
  pub dispatch: DispatchBoard,
  pub alerts: AlertHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PowerSnapshot {
  pub energy: u32,
  pub capacity: u32,
  pub generators: Vec<GeneratorSnapshot>,
  pub batteries: Vec<Battery>,
}
