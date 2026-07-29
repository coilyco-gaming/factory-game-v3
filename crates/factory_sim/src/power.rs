use crate::dispatch::{DispatchBoard, DispatchIntent};
use crate::resources::{Inventory, InventorySnapshot};
use crate::NodeId;
use factory_content::{ContentDatabase, ItemId, PowerSpec};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct PowerPlant {
  pub fuel: Inventory,
  pub dispatch: DispatchBoard,
  pub spec: PowerSpec,
  pub energy: u32,
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
      energy: 0,
    }
  }

  pub fn refresh_dispatch(&mut self) {
    self.dispatch.intents = (self.fuel.count(self.spec.fuel_item) < self.spec.fuel_buffer)
      .then(|| DispatchIntent::deliver(self.spec.fuel_item, NodeId::Road, NodeId::PowerPlant))
      .into_iter()
      .collect();
  }

  pub fn generate(&mut self, events: &mut Vec<String>) -> (u32, u32) {
    if self.energy >= self.spec.grid_capacity
      || self.fuel.count(self.spec.fuel_item) < self.spec.burn_rate
    {
      return (0, 0);
    }
    let burned = self
      .fuel
      .remove_up_to(self.spec.fuel_item, self.spec.burn_rate);
    let generated = self
      .spec
      .gain_rate
      .min(self.spec.grid_capacity.saturating_sub(self.energy));
    self.energy += generated;
    events.push(format!(
      "power burn {} {} generated {} grid {}/{}",
      burned, self.spec.fuel_item, generated, self.energy, self.spec.grid_capacity
    ));
    (burned, generated)
  }

  pub fn consume(&mut self, amount: u32, consumer: &str, events: &mut Vec<String>) -> bool {
    if amount == 0 {
      return true;
    }
    if self.energy < amount {
      events.push(format!(
        "power starved {consumer} need {} grid {}/{}",
        amount, self.energy, self.spec.grid_capacity
      ));
      return false;
    }
    self.energy -= amount;
    events.push(format!(
      "power consume {consumer} {} grid {}/{}",
      amount, self.energy, self.spec.grid_capacity
    ));
    true
  }

  pub fn snapshot(&self) -> PowerSnapshot {
    PowerSnapshot {
      fuel_item: self.spec.fuel_item,
      fuel: self.fuel.snapshot(),
      energy: self.energy,
      capacity: self.spec.grid_capacity,
      burn_rate: self.spec.burn_rate,
      gain_rate: self.spec.gain_rate,
      dispatch: self.dispatch.clone(),
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
}
