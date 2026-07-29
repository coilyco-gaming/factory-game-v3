use crate::dispatch::{DispatchBoard, DispatchIntent};
use crate::resources::{Inventory, InventorySnapshot};
use crate::NodeId;
use factory_content::{ContentDatabase, ItemId, PowerSpec};
use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryOwner {
  Node(NodeId),
  Hauler(u8),
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
pub struct PowerPlant {
  pub fuel: Inventory,
  pub dispatch: DispatchBoard,
  pub spec: PowerSpec,
}

impl PowerPlant {
  pub fn new(content: &ContentDatabase, spec: PowerSpec, mut fuel: Inventory) -> Self {
    fuel.reserve(spec.fuel_item, spec.fuel_buffer);
    fuel
      .insert_exact(content, spec.fuel_item, spec.initial_fuel)
      .expect("starter power-plant fuel fits its reserved inventory");
    Self {
      fuel,
      dispatch: DispatchBoard::new(),
      spec,
    }
  }

  pub fn refresh_dispatch(&mut self) {
    self.dispatch.intents = (self.fuel.count(self.spec.fuel_item) < self.spec.fuel_buffer)
      .then(|| DispatchIntent::deliver(self.spec.fuel_item, NodeId::Road, NodeId::PowerPlant))
      .into_iter()
      .collect();
  }

  pub fn generate(
    &mut self,
    battery: &mut Battery,
    events: &mut Vec<String>,
  ) -> (u32, u32) {
    if battery.energy >= battery.capacity
      || self.fuel.count(self.spec.fuel_item) < self.spec.burn_rate
    {
      return (0, 0);
    }
    let burned = self
      .fuel
      .remove_up_to(self.spec.fuel_item, self.spec.burn_rate);
    let generated = battery.charge(self.spec.gain_rate);
    events.push(format!(
      "power burn {} {} generated {} grid {}/{}",
      burned, self.spec.fuel_item, generated, battery.energy, battery.capacity
    ));
    (burned, generated)
  }

  pub fn snapshot(&self, batteries: impl Iterator<Item = Battery>) -> PowerSnapshot {
    let batteries = batteries.collect::<Vec<_>>();
    PowerSnapshot {
      fuel_item: self.spec.fuel_item,
      fuel: self.fuel.snapshot(),
      energy: batteries.iter().map(|battery| battery.energy).sum(),
      capacity: batteries.iter().map(|battery| battery.capacity).sum(),
      burn_rate: self.spec.burn_rate,
      gain_rate: self.spec.gain_rate,
      dispatch: self.dispatch.clone(),
      batteries,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PowerSnapshot {
  pub fuel_item: ItemId,
  pub fuel: InventorySnapshot,
  pub energy: u32,
  pub capacity: u32,
  pub burn_rate: u32,
  pub gain_rate: u32,
  pub dispatch: DispatchBoard,
  pub batteries: Vec<Battery>,
}
